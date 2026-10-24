use super::DetailError;
use rustix::fs::{self, FileType, Mode, OFlags};
use std::{
    fs::File,
    path::{Component, Path},
};

/// File-descriptor traversal refuses every symlink and '..'. The last component
/// is opened with O_CREAT|O_EXCL, never check-then-open. Only local ext4/tmpfs
/// semantics are supported; filesystem IDs alone do not authenticate a host.
pub(super) fn create(path: &Path) -> Result<File, DetailError> {
    use std::os::unix::ffi::OsStrExt;
    if !path.is_absolute()
        || path.as_os_str().as_bytes().ends_with(b"/")
        || path.as_os_str().len() > 4096
    {
        return Err(DetailError::PathRejected);
    }
    let mut parts = path.components().peekable();
    if parts.next() != Some(Component::RootDir) {
        return Err(DetailError::PathRejected);
    }
    let dirflags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut dir =
        fs::open("/", dirflags, Mode::empty()).map_err(|_| DetailError::ProtectionFailed)?;
    while let Some(part) = parts.next() {
        let Component::Normal(name) = part else {
            return Err(DetailError::PathRejected);
        };
        if parts.peek().is_some() {
            dir = fs::openat(&dir, name, dirflags, Mode::empty())
                .map_err(|_| DetailError::PathRejected)?;
            continue;
        }
        let uid = rustix::process::geteuid().as_raw();
        let parent = fs::fstat(&dir).map_err(|_| DetailError::ProtectionFailed)?;
        // The destination directory must be controlled by the effective owner.
        if parent.st_uid != uid || parent.st_mode & 0o022 != 0 {
            return Err(DetailError::ProtectionFailed);
        }
        let filesystem = fs::fstatfs(&dir).map_err(|_| DetailError::ProtectionFailed)?;
        // Linux UAPI EXT4_SUPER_MAGIC / TMPFS_MAGIC; other mounts fail closed.
        if !matches!(filesystem.f_type as u64, 0xef53 | 0x01021994) {
            return Err(DetailError::ProtectionFailed);
        }
        let fd = fs::openat(
            &dir,
            name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        )
        .map_err(|_| DetailError::CreateRefused)?;
        verify(&fd, uid)?;
        return Ok(File::from(fd));
    }
    Err(DetailError::PathRejected)
}
fn verify(fd: &impl std::os::fd::AsFd, uid: u32) -> Result<(), DetailError> {
    let stat = fs::fstat(fd).map_err(|_| DetailError::ProtectionFailed)?;
    if stat.st_uid != uid
        || stat.st_mode & 0o7777 != 0o600
        || stat.st_nlink != 1
        || !FileType::from_raw_mode(stat.st_mode).is_file()
        || stat.st_size != 0
    {
        return Err(DetailError::ProtectionFailed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_creation_protection_primitives_in_isolation() {
        // Native CI can exercise the private OS primitive with synthetic bytes.
        // This is not a bypass on the product exporter or a CI source export.
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        let dir = std::env::temp_dir().join(format!("quent-primitive-{}", std::process::id()));
        std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
        let path = dir.join("new");
        let f = create(&path).unwrap();
        let stat = fs::fstat(&f).unwrap();
        assert_eq!(stat.st_mode & 0o7777, 0o600);
        assert_eq!(stat.st_uid, rustix::process::geteuid().as_raw());
        assert!(matches!(create(&path), Err(DetailError::CreateRefused)));
        // Actual bad protection and actual owner-comparison failures are refused.
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(verify(&f, stat.st_uid), Err(DetailError::ProtectionFailed));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            verify(&f, stat.st_uid.wrapping_add(1)),
            Err(DetailError::ProtectionFailed)
        );
        assert_eq!(f.metadata().unwrap().len(), 0);
        drop(f);
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(!dir.exists());
    }
    #[test]
    fn restrictive_umask_child() {
        let Some(path) = std::env::var_os("QUENT_DETAIL_UMASK_CHILD_PATH") else {
            return;
        };
        // Only this isolated child changes umask, not the concurrent test process.
        rustix::process::umask(Mode::from_raw_mode(0o777));
        let path = std::path::PathBuf::from(path);
        assert!(matches!(create(&path), Err(DetailError::ProtectionFailed)));
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);
    }
    #[test]
    fn actual_os_protection_failure_precedes_every_sensitive_write() {
        use std::os::unix::fs::DirBuilderExt;
        let dir = std::env::temp_dir().join(format!("quent-umask-{}", std::process::id()));
        std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
        let path = dir.join("new");
        let o = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "local_details::protection::tests::restrictive_umask_child",
            ])
            .env("QUENT_DETAIL_UMASK_CHILD_PATH", &path)
            .output()
            .unwrap();
        assert!(o.status.success(), "protection failure child failed");
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(!dir.exists());
    }
}
