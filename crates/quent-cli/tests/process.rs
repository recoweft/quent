use std::{
    fs,
    process::{Command, Stdio},
};
#[test]
fn every_current_cli_route_is_fixed_non_success() {
    let cases: &[&[&str]] = &[
        &[],
        &["--help"],
        &["-h"],
        &["--version"],
        &["-V"],
        &["--CANARY_unknown=SQL_tenant_subject"],
        &["CANARY_unknown_command"],
        &["manifest", "{broken CANARY"],
        &["mcp", "--detail", "/CANARY_secret"],
        &["--", "CANARY\n\r\u{1b}"],
    ];
    for args in cases {
        let o = Command::new(env!("CARGO_BIN_EXE_quent-cli"))
            .args(*args)
            .env("RUST_BACKTRACE", "full")
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(1));
        assert_eq!(o.stdout, b"");
        assert_eq!(o.stderr, b"QUENT_NOT_IMPLEMENTED\n");
    }
    let large = "CANARY".repeat(3000);
    let o = Command::new(env!("CARGO_BIN_EXE_quent-cli"))
        .arg(large)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(o.stdout, b"");
    assert_eq!(o.stderr, b"QUENT_NOT_IMPLEMENTED\n");
}
#[test]
fn stdin_invalid_and_oversized_content_is_never_read_or_echoed() {
    let dir = std::env::temp_dir().join(format!("quent-cli-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("CANARY-secret-path");
    for bytes in [
        b"{\xff\x00CANARY SQL tenant subject\n".to_vec(),
        vec![b'X'; 2 * 1024 * 1024],
    ] {
        fs::write(&path, bytes).unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_quent-cli"))
            .arg(&path)
            .stdin(Stdio::from(fs::File::open(&path).unwrap()))
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(1));
        assert!(o.stdout.is_empty());
        assert_eq!(o.stderr, b"QUENT_NOT_IMPLEMENTED\n");
    }
    fs::remove_dir_all(&dir).unwrap();
    assert!(!dir.exists());
}
#[cfg(unix)]
#[test]
fn invalid_os_argument_and_stderr_io_failure_do_not_panic() {
    use std::os::unix::ffi::OsStringExt;
    let o = Command::new(env!("CARGO_BIN_EXE_quent-cli"))
        .arg(std::ffi::OsString::from_vec(b"CANARY\xff".to_vec()))
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(o.stdout.is_empty());
    assert_eq!(o.stderr, b"QUENT_NOT_IMPLEMENTED\n");
    // Read-only stderr descriptor forces EBADF without touching a user file.
    let input = fs::File::open("/dev/null").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_quent-cli"))
        .stderr(Stdio::from(input))
        .env("RUST_BACKTRACE", "full")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(o.stdout.is_empty());
}
