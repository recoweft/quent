//! Shared opened-handle, bounded regular-file input. No public error strings.
use std::{io::Read, path::Path};
#[derive(Debug)]
pub(super) enum InputError {
    Invalid,
    Budget,
    Io,
    #[allow(dead_code)]
    Unsupported,
}
pub(super) fn read(path: &Path, max: usize) -> Result<Vec<u8>, InputError> {
    let f = open_input(path)?;
    if !f.metadata().map_err(|_| InputError::Io)?.is_file() {
        return Err(InputError::Invalid);
    }
    let mut b = Vec::new();
    f.take((max + 1) as u64)
        .read_to_end(&mut b)
        .map_err(|_| InputError::Io)?;
    if b.len() > max {
        return Err(InputError::Budget);
    }
    Ok(b)
}
// Check the object actually opened, never a path stat followed by a different open.
// NONBLOCK prevents a writerless FIFO from hanging before the regular-file check.
#[cfg(unix)]
fn open_input(path: &Path) -> Result<std::fs::File, InputError> {
    use rustix::fs::{Mode, OFlags, open};
    open(
        path,
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(std::fs::File::from)
    .map_err(|_| InputError::Io)
}
#[cfg(windows)]
fn open_input(path: &Path) -> Result<std::fs::File, InputError> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    // Open the reparse point itself; do not follow a final symlink. Only a
    // regular file proceeds to Read (pipe/device metadata errors refuse).
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(0x0020_0000) // FILE_FLAG_OPEN_REPARSE_POINT
        .open(path)
        .map_err(|_| InputError::Io)?;
    let metadata = file.metadata().map_err(|_| InputError::Io)?;
    if metadata.file_attributes() & (0x400 | 0x40) != 0 {
        // Reparse points and device attributes are outside this input contract.
        return Err(InputError::Invalid);
    }
    Ok(file)
}
#[cfg(not(any(unix, windows)))]
fn open_input(_: &Path) -> Result<std::fs::File, InputError> {
    Err(InputError::Unsupported)
}
