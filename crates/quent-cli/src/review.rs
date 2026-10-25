//! Exact positional grammar is validated before any filesystem operation.
use std::{ffi::OsStr, io::Write};
fn fail(code: u8, text: &'static str) -> u8 {
    let _ = std::io::stderr().write_all(text.as_bytes());
    code
}
pub(super) fn run() -> Option<u8> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(OsStr::new("review")) {
        return None;
    }
    let kind = args.next();
    let format = args.next();
    let path = args.next();
    if args.next().is_some()
        || !matches!(
            kind.as_deref().and_then(OsStr::to_str),
            Some("plan" | "spec" | "manifest" | "rust")
        )
        || !matches!(
            format.as_deref().and_then(OsStr::to_str),
            Some("--pretty" | "--json")
        )
        || path.as_deref().is_none_or(|p| p.is_empty() || p == "-")
    {
        return Some(fail(2, "QUENT_REVIEW_INVALID\n"));
    }
    let path = path.expect("checked positional path");
    if path.as_encoded_bytes().len() > 4096 {
        return Some(fail(2, "QUENT_REVIEW_BUDGET\n"));
    }
    // Rust is an explicitly unsupported target, never a source parser or file open.
    if kind.as_deref() == Some(OsStr::new("rust")) {
        return Some(fail(4, "QUENT_REVIEW_UNSUPPORTED\n"));
    }
    #[cfg(not(feature = "review"))]
    {
        Some(fail(4, "QUENT_REVIEW_UNSUPPORTED\n"))
    }
    #[cfg(feature = "review")]
    {
        use crate::input::{InputError, read};
        let bytes = match read(std::path::Path::new(&path), 65_536) {
            Ok(b) => b,
            Err(e) => {
                return Some(match e {
                    InputError::Invalid => fail(2, "QUENT_REVIEW_INVALID\n"),
                    InputError::Budget => fail(2, "QUENT_REVIEW_BUDGET\n"),
                    InputError::Io => fail(8, "QUENT_REVIEW_IO\n"),
                    InputError::Unsupported => fail(4, "QUENT_REVIEW_UNSUPPORTED\n"),
                });
            }
        };
        let result = match kind.as_deref().and_then(OsStr::to_str) {
            Some("plan") => quent::review::plan_source(&bytes),
            Some("spec") => quent::review::operation_spec(&bytes),
            Some("manifest") => quent::review::manifest(&bytes),
            _ => unreachable!("checked target"),
        };
        Some(match result {
            Err(e) => {
                let _ = writeln!(std::io::stderr(), "{}", e.code());
                e.exit_code()
            }
            Ok(r) => {
                let output = if format.as_deref() == Some(OsStr::new("--json")) {
                    r.to_json()
                } else {
                    r.to_pretty()
                };
                if writeln!(std::io::stdout(), "{output}").is_err() {
                    fail(8, "QUENT_REVIEW_IO\n")
                } else {
                    r.exit_code()
                }
            }
        })
    }
}
