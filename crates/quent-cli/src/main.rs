//! Bootstrap refusal surface. No argument/stdin/path parser, DB or detail export.
use std::io::Write;
fn main() -> std::process::ExitCode {
    // Never format panic payload/location or activate a backtrace on this surface.
    std::panic::set_hook(Box::new(|_| {
        let _ = std::io::stderr().write_all(b"QUENT_PANIC\n");
    }));
    // write_all errors are deliberately discarded: even a closed/full stderr
    // must not trigger a default panic hook with a source path/backtrace.
    let _ = std::io::stderr().write_all(b"QUENT_NOT_IMPLEMENTED\n");
    std::process::ExitCode::FAILURE
}
