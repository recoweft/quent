//! Application-only, explicit local opt-in + absolute new path. No wire parser,
//! execution authority, stdout fallback or generic sensitive serializer API.
use quent_core::SensitiveSource;
use std::{fmt, path::Path};
mod encoding;
#[cfg(target_os = "linux")]
mod protection;

/// Product surfaces are finite. Selecting LocalApplication is an application
/// assertion, not authentication of the caller/process. CI detection still applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    LocalApplication,
    Ci,
    Mcp,
    Remote,
}
/// Deliberate sensitive-output acknowledgement; no boolean/JSON override.
#[derive(Debug)]
pub struct LocalOptIn(());
impl LocalOptIn {
    pub fn acknowledge_sensitive_local_file() -> Self {
        Self(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailError {
    SurfaceDenied,
    EnvironmentDenied,
    UnsupportedOs,
    PathRejected,
    CreateRefused,
    ProtectionFailed,
    Budget,
    WriteFailed,
}
impl DetailError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::SurfaceDenied => "QUENT_DETAIL_SURFACE_DENIED",
            Self::EnvironmentDenied => "QUENT_DETAIL_ENVIRONMENT_DENIED",
            Self::UnsupportedOs => "QUENT_DETAIL_UNSUPPORTED_OS",
            Self::PathRejected => "QUENT_DETAIL_PATH_REJECTED",
            Self::CreateRefused => "QUENT_DETAIL_CREATE_REFUSED",
            Self::ProtectionFailed => "QUENT_DETAIL_PROTECTION_FAILED",
            Self::Budget => "QUENT_DETAIL_BUDGET",
            Self::WriteFailed => "QUENT_DETAIL_WRITE_FAILED",
        }
    }
}
impl fmt::Display for DetailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for DetailError {}
/// A receipt carries neither path, source bytes nor a capability.
#[derive(Debug, PartialEq, Eq)]
pub struct CreatedLocalDetail(());
// Presence (including empty/false/non-Unicode) denies. No environment allow key.
const DENIED_ENV: &[&str] = &[
    "CI",
    "CONTINUOUS_INTEGRATION",
    "GITHUB_ACTIONS",
    "GITLAB_CI",
    "TF_BUILD",
    "JENKINS_URL",
    "JENKINS_HOME",
    "BUILD_BUILDID",
    "TEAMCITY_VERSION",
    "BUILDKITE",
    "CIRCLECI",
    "TRAVIS",
    "APPVEYOR",
    "CODEBUILD_BUILD_ID",
    "BITBUCKET_BUILD_NUMBER",
    "DRONE",
    "SEMAPHORE",
    "WOODPECKER",
    "QUENT_MCP",
    "QUENT_REMOTE",
];
/// Exports only the exact finalized record borrowed from Plan/Inspection or
/// PolicyPreparation::sensitive_source().final_source(). No context/session/handle.
/// Unsupported OS/filesystem protection fails before serialization or writing.
pub fn create_new(
    _opt_in: LocalOptIn,
    surface: Surface,
    path: &Path,
    source: SensitiveSource<'_>,
) -> Result<CreatedLocalDetail, DetailError> {
    if surface != Surface::LocalApplication {
        return Err(DetailError::SurfaceDenied);
    }
    if DENIED_ENV.iter().any(|k| std::env::var_os(k).is_some()) {
        return Err(DetailError::EnvironmentDenied);
    }
    write_source(path, source)
}
#[cfg(target_os = "linux")]
fn write_source(
    path: &Path,
    source: SensitiveSource<'_>,
) -> Result<CreatedLocalDetail, DetailError> {
    let mut file = protection::create(path)?;
    let bytes = encoding::encode(source)?;
    use std::io::Write;
    file.write_all(bytes.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|_| DetailError::WriteFailed)?;
    Ok(CreatedLocalDetail(()))
}
#[cfg(not(target_os = "linux"))]
fn write_source(
    _path: &Path,
    _source: SensitiveSource<'_>,
) -> Result<CreatedLocalDetail, DetailError> {
    Err(DetailError::UnsupportedOs)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> quent_core::Plan {
        use quent_core::*;
        let t = Table::new(vec![Ident::new("canary").unwrap()], None).unwrap();
        Draft::delete(t)
            .freeze()
            .finalize(Dialect::Postgres)
            .unwrap()
    }
    #[cfg(not(target_os = "linux"))]
    #[test]
    fn native_unsupported_backend_never_touches_filesystem() {
        // Private platform boundary, isolated synthetic source; public CI guard
        // remains enabled and is tested independently. No public bypass.
        let dir = std::env::temp_dir().join(format!("quent-unsupported-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("new");
        assert_eq!(
            write_source(&path, source().sensitive_source()).unwrap_err(),
            DetailError::UnsupportedOs
        );
        assert!(!path.exists());
        std::fs::remove_dir(&dir).unwrap();
        assert!(!dir.exists());
    }
    #[test]
    fn environment_child() {
        let Some(path) = std::env::var_os("QUENT_DETAIL_TEST_CHILD_PATH") else {
            return;
        };
        let path = std::path::PathBuf::from(path);
        let e = create_new(
            LocalOptIn::acknowledge_sensitive_local_file(),
            Surface::LocalApplication,
            &path,
            source().sensitive_source(),
        )
        .unwrap_err();
        assert_eq!(e, DetailError::EnvironmentDenied);
        assert!(!path.exists());
    }
    #[test]
    fn every_known_environment_key_denies_even_false_or_empty() {
        let path = std::env::temp_dir().join(format!("quent-denial-{}", std::process::id()));
        assert!(!path.exists());
        for key in DENIED_ENV {
            for value in ["", "false"] {
                let mut c = std::process::Command::new(std::env::current_exe().unwrap());
                c.args(["--exact", "local_details::tests::environment_child"]);
                for k in DENIED_ENV {
                    c.env_remove(k);
                }
                let o = c
                    .env(key, value)
                    .env("QUENT_DETAIL_TEST_CHILD_PATH", &path)
                    .output()
                    .unwrap();
                assert!(o.status.success(), "environment denial child failed");
                assert!(!path.exists());
            }
        }
    }
}
