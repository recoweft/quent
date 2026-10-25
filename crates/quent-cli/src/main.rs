//! Explicit bounded workflows and fixed refusal for all other routes.
use std::io::Write;
#[cfg(any(feature = "manifest", feature = "review", feature = "source-review"))]
mod input;
mod review;
#[cfg(feature = "manifest")]
mod verify;
fn main() -> std::process::ExitCode {
    install_panic_hook();
    if let Some(code) = review::run() {
        return std::process::ExitCode::from(code);
    }
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

fn install_panic_hook() {
    // Never format payload, location or backtrace, even with inherited tracing env.
    std::panic::set_hook(Box::new(|_| {
        let _ = std::io::stderr().write_all(b"QUENT_PANIC\n");
    }));
}
#[cfg(test)]
mod tests {
    #[test]
    fn panic_fixture() {
        if std::env::var_os("QUENT_TEST_PANIC").is_some() {
            super::install_panic_hook();
            panic!("CANARY_SQL_name_value_reason_path_credential");
        }
    }
    #[test]
    fn fixed_panic_hook_with_backtrace_and_tracing_environment() {
        use std::{
            process::{Command, Stdio},
            time::{Duration, Instant},
        };
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "tests::panic_fixture", "--nocapture"])
            .env("QUENT_TEST_PANIC", "1")
            .env("RUST_BACKTRACE", "full")
            .env("RUST_LOG", "trace")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("panic fixture watchdog killed and reaped child");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let o = child.wait_with_output().unwrap();
        assert!(!o.status.success());
        assert_eq!(o.stderr, b"QUENT_PANIC\n");
        assert!(!String::from_utf8_lossy(&o.stdout).contains("CANARY"));
    }
}
