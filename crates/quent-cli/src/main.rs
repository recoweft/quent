//! Bootstrap refusal surface. No argument/stdin/path parser, DB or detail export.
use std::io::Write;
#[cfg(feature = "manifest")]
mod verify;
fn main() -> std::process::ExitCode {
    // Never format panic payload/location or activate a backtrace on this surface.
    std::panic::set_hook(Box::new(|_| {
        let _ = std::io::stderr().write_all(b"QUENT_PANIC\n");
    }));
    #[cfg(feature = "manifest")]
    if let Some(code) = verify::run() {
        return std::process::ExitCode::from(code);
    }
    #[cfg(not(feature = "manifest"))]
    {
        let mut args = std::env::args_os().skip(1);
        if args.next().as_deref() == Some(std::ffi::OsStr::new("manifest"))
            && args.next().as_deref() == Some(std::ffi::OsStr::new("verify"))
        {
            let mode = args.next();
            let a = args.next();
            let b = args.next();
            let valid = matches!(
                mode.as_deref().and_then(|s| s.to_str()),
                Some("--offline" | "--live")
            ) && a.is_some()
                && b.is_some()
                && args.next().is_none();
            let _ = std::io::stderr().write_all(if valid {
                b"QUENT_VERIFY_UNSUPPORTED\n"
            } else {
                b"QUENT_VERIFY_INVALID\n"
            });
            return std::process::ExitCode::from(if valid { 4 } else { 2 });
        }
    }
    // write_all errors are deliberately discarded: even a closed/full stderr
    // must not trigger a default panic hook with a source path/backtrace.
    let _ = std::io::stderr().write_all(b"QUENT_NOT_IMPLEMENTED\n");
    std::process::ExitCode::FAILURE
}
