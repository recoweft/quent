//! Explicit regular-file collection only; grammar/features precede all reads.
use std::{ffi::OsStr, io::Write};
fn fail(code: u8, text: &'static str) -> u8 {
    let _ = std::io::stderr().write_all(text.as_bytes());
    code
}
pub(super) fn run() -> Option<u8> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(OsStr::new("review-ci")) {
        return None;
    }
    let format = args.next();
    let paths = [args.next(), args.next(), args.next()];
    if args.next().is_some()
        || !matches!(
            format.as_deref().and_then(OsStr::to_str),
            Some("--json" | "--pretty" | "--sarif")
        )
        || paths
            .iter()
            .any(|p| p.as_deref().is_none_or(|p| p.is_empty() || p == "-"))
    {
        return Some(fail(2, "QUENT_REVIEW_INVALID\n"));
    }
    if paths
        .iter()
        .any(|p| p.as_ref().expect("checked path").as_encoded_bytes().len() > 4096)
    {
        return Some(fail(2, "QUENT_REVIEW_BUDGET\n"));
    }
    #[cfg(not(feature = "review-ci"))]
    {
        Some(fail(4, "QUENT_REVIEW_UNSUPPORTED\n"))
    }
    #[cfg(feature = "review-ci")]
    {
        Some(execute(format.as_deref().expect("checked format"), paths))
    }
}
#[cfg(feature = "review-ci")]
fn execute(format: &OsStr, paths: [Option<std::ffi::OsString>; 3]) -> u8 {
    use quent::review_ci::{Baseline, InputSet, Time, compare};
    let mut bytes = Vec::new();
    for p in paths {
        let p = p.expect("checked path");
        match crate::input::read(std::path::Path::new(&p), 65_536) {
            Ok(b) => bytes.push(b),
            Err(e) => {
                return match e {
                    crate::input::InputError::Invalid => fail(2, "QUENT_REVIEW_INVALID\n"),
                    crate::input::InputError::Budget => fail(2, "QUENT_REVIEW_BUDGET\n"),
                    crate::input::InputError::Io => fail(8, "QUENT_REVIEW_IO\n"),
                    crate::input::InputError::Unsupported => fail(4, "QUENT_REVIEW_UNSUPPORTED\n"),
                };
            }
        }
    }
    let result = (|| {
        let head = InputSet::parse(&bytes[0])?;
        let base = InputSet::parse(&bytes[1])?;
        let baseline = Baseline::parse(&bytes[2])?;
        // Separate CLI boundary. Neither JSON nor environment supplies the time.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|d| Time::unix_seconds(d.as_secs()).ok());
        compare(head, base, &baseline, now)
    })();
    match result {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "{}", e.code());
            e.exit_code()
        }
        Ok(r) => {
            let text = if format == "--sarif" {
                r.to_sarif()
            } else if format == "--pretty" {
                r.to_pretty()
            } else {
                r.to_json()
            };
            if writeln!(std::io::stdout(), "{text}").is_err() {
                fail(8, "QUENT_REVIEW_IO\n")
            } else {
                r.exit_code()
            }
        }
    }
}
