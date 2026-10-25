#[cfg(any(unix, feature = "review"))]
use std::path::Path;
use std::{
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};
struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "quent-review-{}-{}",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    #[cfg(feature = "review")]
    fn file(&self, name: &str, b: &[u8]) -> PathBuf {
        let p = self.0.join(name);
        fs::write(&p, b).unwrap();
        p
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn run(args: &[&OsStr]) -> Output {
    run_at(args, "CANARY_credential")
}
fn run_at(args: &[&OsStr], dsn: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_quent-cli"))
        .args(args)
        .env("RUST_BACKTRACE", "full")
        .env("RUST_LOG", "trace")
        .env("DATABASE_URL", dsn)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("CLI watchdog killed and reaped child");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn refused(o: Output, c: i32, e: &[u8]) {
    assert_eq!(o.status.code(), Some(c));
    assert!(o.stdout.is_empty());
    assert_eq!(o.stderr, e);
}
#[test]
fn finite_grammar_and_default_refusal_precede_file_or_stdin_io() {
    let f = Files::new();
    let path = f.0.join("CANARY_absent");
    #[cfg(unix)]
    rustix_fifo_when_available(&path);
    for args in [
        vec!["review"],
        vec!["review", "plan"],
        vec!["review", "plan", "--json"],
        vec!["review", "unknown", "--json"],
        vec!["review", "plan", "--wrong"],
        vec!["review", "plan", "--json", "-"],
        vec!["review", "plan", "--json", "CANARY", "extra"],
    ] {
        let mut a: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
        if a.len() == 3 && (args[1] == "unknown" || args[2] == "--wrong") {
            a.push(path.as_os_str());
        }
        refused(run(&a), 2, b"QUENT_REVIEW_INVALID\n");
    }
    refused(
        run(&[
            OsStr::new("review"),
            OsStr::new("rust"),
            OsStr::new("--json"),
            path.as_os_str(),
        ]),
        4,
        b"QUENT_REVIEW_UNSUPPORTED\n",
    );
    #[cfg(not(feature = "review"))]
    for k in ["plan", "spec", "manifest"] {
        refused(
            run(&[
                OsStr::new("review"),
                OsStr::new(k),
                OsStr::new("--json"),
                path.as_os_str(),
            ]),
            4,
            b"QUENT_REVIEW_UNSUPPORTED\n",
        );
    }
}
#[cfg(unix)]
fn rustix_fifo_when_available(p: &Path) {
    #[cfg(any(feature = "manifest", feature = "review"))]
    rustix::fs::mkfifoat(rustix::fs::CWD, p, rustix::fs::Mode::RUSR).unwrap();
    #[cfg(not(any(feature = "manifest", feature = "review")))]
    let _ = p;
}
#[cfg(feature = "review")]
fn command(k: &str, fmt: &str, p: &Path) -> Output {
    run(&[
        OsStr::new("review"),
        OsStr::new(k),
        OsStr::new(fmt),
        p.as_os_str(),
    ])
}
#[cfg(feature = "review")]
#[test]
fn product_pretty_and_json_match_independent_outputs_and_fixed_exits() {
    let f = Files::new();
    for (k, src, expected, pretty) in [
        (
            "plan",
            include_bytes!("../../../fixtures/review-v1/select.json").as_slice(),
            include_bytes!("../../../fixtures/review-v1/select.expected.json").as_slice(),
            include_bytes!("../../../fixtures/review-v1/select.pretty.expected.json").as_slice(),
        ),
        (
            "spec",
            include_bytes!("../../../fixtures/review-v1/spec.json").as_slice(),
            include_bytes!("../../../fixtures/review-v1/spec.expected.json").as_slice(),
            include_bytes!("../../../fixtures/review-v1/spec.pretty.expected.json").as_slice(),
        ),
        (
            "manifest",
            include_bytes!("../../../fixtures/manifest-v1/postgres.json").as_slice(),
            include_bytes!("../../../fixtures/review-v1/manifest.expected.json").as_slice(),
            include_bytes!("../../../fixtures/review-v1/manifest.pretty.expected.json").as_slice(),
        ),
    ] {
        let p = f.file("CANARY_private_path", src);
        let expected_bytes = expected;
        let expected: serde_json::Value = serde_json::from_slice(expected).unwrap();
        for format in ["--json", "--pretty"] {
            let o = command(k, format, &p);
            assert_eq!(o.status.code(), Some(3));
            assert!(o.stderr.is_empty());
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&o.stdout).unwrap(),
                expected
            );
            assert_eq!(
                o.stdout,
                if format == "--json" {
                    expected_bytes
                } else {
                    pretty
                }
            );
            assert_eq!(o.stdout.last(), Some(&b'\n'));
            assert_eq!(o.stdout, command(k, format, &p).stdout);
            for canary in ["private_", "1234567", "CANARY", "SELECT"] {
                assert!(!String::from_utf8_lossy(&o.stdout).contains(canary));
            }
        }
    }
    let mut v: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/review-v1/select.json")).unwrap();
    v["operation"]["family"] = serde_json::json!("delete");
    v["operation"]["projection"] = serde_json::json!([]);
    v["operation"].as_object_mut().unwrap().remove("filter");
    v["operation"].as_object_mut().unwrap().remove("limit");
    let p = f.file("CANARY_block", &serde_json::to_vec(&v).unwrap());
    let o = command("plan", "--json", &p);
    assert_eq!(o.status.code(), Some(5));
    assert!(o.stderr.is_empty());
    let j: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(
        j["inspection"]["findings"],
        serde_json::json!(["missing_where"])
    );
}
#[cfg(feature = "review")]
#[test]
fn invalid_utf8_bytes_and_all_target_budgets_have_fixed_streams() {
    let f = Files::new();
    for (k, src) in [
        (
            "plan",
            include_bytes!("../../../fixtures/review-v1/select.json").as_slice(),
        ),
        (
            "spec",
            include_bytes!("../../../fixtures/review-v1/spec.json").as_slice(),
        ),
        (
            "manifest",
            include_bytes!("../../../fixtures/manifest-v1/postgres.json").as_slice(),
        ),
    ] {
        let mut b = src.to_vec();
        b.resize(65536, b' ');
        let p = f.file("CANARY_boundary", &b);
        assert_eq!(command(k, "--json", &p).status.code(), Some(3));
        b.push(b' ');
        fs::write(&p, &b).unwrap();
        refused(command(k, "--pretty", &p), 2, b"QUENT_REVIEW_BUDGET\n");
        fs::write(&p, b"CANARY\xff").unwrap();
        refused(command(k, "--json", &p), 2, b"QUENT_REVIEW_INVALID\n");
    }
    let p = f.file(
        "CANARY_claim",
        br#"{"kind":"CANARY_credential","kind":"other"}"#,
    );
    for k in ["plan", "spec", "manifest"] {
        refused(command(k, "--json", &p), 2, b"QUENT_REVIEW_INVALID\n");
    }
}
#[cfg(all(feature = "review", unix))]
#[test]
fn writerless_fifo_directory_device_and_final_symlink_never_read() {
    let f = Files::new();
    let fifo = f.0.join("CANARY_fifo");
    rustix_fifo_when_available(&fifo);
    let regular = f.file(
        "CANARY_regular",
        include_bytes!("../../../fixtures/review-v1/select.json"),
    );
    let link = f.0.join("CANARY_link");
    std::os::unix::fs::symlink(&regular, &link).unwrap();
    for k in ["plan", "spec", "manifest"] {
        for p in [&fifo, f.0.as_path(), Path::new("/dev/null")] {
            refused(command(k, "--json", p), 2, b"QUENT_REVIEW_INVALID\n");
        }
        refused(command(k, "--pretty", &link), 8, b"QUENT_REVIEW_IO\n");
    }
}
#[cfg(all(feature = "review", unix))]
#[test]
fn path_replacement_keeps_actual_opened_object_safety() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let f = Files::new();
    let b = include_bytes!("../../../fixtures/review-v1/select.json");
    let stable = f.file("CANARY_stable", b);
    let changing = f.file("CANARY_changing", b);
    let stop = Arc::new(AtomicBool::new(false));
    std::thread::scope(|scope| {
        let stop_writer = stop.clone();
        let changing_ref = &changing;
        let stable_ref = &stable;
        let dir = &f.0;
        let writer = scope.spawn(move || {
            let temp = dir.join("replace");
            while !stop_writer.load(Ordering::Relaxed) {
                fs::write(&temp, b).unwrap();
                fs::rename(&temp, changing_ref).unwrap();
                rustix::fs::mkfifoat(rustix::fs::CWD, &temp, rustix::fs::Mode::RUSR).unwrap();
                fs::rename(&temp, changing_ref).unwrap();
                std::os::unix::fs::symlink(stable_ref, &temp).unwrap();
                fs::rename(&temp, changing_ref).unwrap();
            }
        });
        let result = std::panic::catch_unwind(|| {
            for _ in 0..32 {
                let o = command("plan", "--json", &changing);
                match o.status.code() {
                    Some(3) => {
                        assert!(o.stderr.is_empty());
                        let j: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
                        assert_eq!(j["source"], "plan_source");
                    }
                    Some(2) => refused(o, 2, b"QUENT_REVIEW_INVALID\n"),
                    Some(8) => refused(o, 8, b"QUENT_REVIEW_IO\n"),
                    _ => panic!("unexpected result"),
                }
            }
        });
        stop.store(true, Ordering::Relaxed);
        writer.join().unwrap();
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    });
}
#[cfg(all(feature = "review", windows))]
#[test]
fn windows_regular_directory_device_and_final_reparse_refuse() {
    let f = Files::new();
    let p = f.file(
        "CANARY_regular",
        include_bytes!("../../../fixtures/review-v1/select.json"),
    );
    assert_eq!(command("plan", "--json", &p).status.code(), Some(3));
    let link = f.0.join("CANARY_reparse");
    std::os::windows::fs::symlink_file(&p, &link).unwrap();
    for p in [f.0.as_path(), Path::new("NUL"), link.as_path()] {
        let o = command("plan", "--json", p);
        assert!(matches!(o.status.code(), Some(2 | 8)));
        assert!(o.stdout.is_empty());
        assert!(o.stderr == b"QUENT_REVIEW_INVALID\n" || o.stderr == b"QUENT_REVIEW_IO\n");
    }
}

#[cfg(feature = "review")]
#[test]
fn review_zero_connect_has_detecting_positive_control() {
    use std::net::{TcpListener, TcpStream};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let control = TcpStream::connect(addr).unwrap();
    assert!(listener.accept().is_ok());
    drop(control);
    let dsn = format!("postgres://CANARY_user:CANARY_password@{addr}/CANARY_database");
    let f = Files::new();
    for (kind, source) in [
        (
            "plan",
            include_bytes!("../../../fixtures/review-v1/select.json").as_slice(),
        ),
        (
            "spec",
            include_bytes!("../../../fixtures/review-v1/spec.json").as_slice(),
        ),
        (
            "manifest",
            include_bytes!("../../../fixtures/manifest-v1/postgres.json").as_slice(),
        ),
    ] {
        let path = f.file("CANARY_source", source);
        let args = [
            OsStr::new("review"),
            OsStr::new(kind),
            OsStr::new("--json"),
            path.as_os_str(),
        ];
        let o = run_at(&args, &dsn);
        assert_eq!(o.status.code(), Some(3));
        assert!(o.stderr.is_empty());
        let mut invalid = args.to_vec();
        invalid.push(OsStr::new("extra"));
        refused(run_at(&invalid, &dsn), 2, b"QUENT_REVIEW_INVALID\n");
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
}

#[cfg(all(feature = "review", windows))]
#[test]
fn windows_path_replacement_keeps_opened_handle_safety() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };
    let f = Files::new();
    let bytes = include_bytes!("../../../fixtures/review-v1/select.json");
    let stable = f.file("CANARY_stable", bytes);
    let changing = f.file("CANARY_changing", bytes);
    let stop = Arc::new(AtomicBool::new(false));
    let replacements = Arc::new(AtomicUsize::new(0));
    std::thread::scope(|scope| {
        let stop_writer = stop.clone();
        let changes = replacements.clone();
        let changing_ref = &changing;
        let stable_ref = &stable;
        let dir = &f.0;
        let writer = scope.spawn(move || {
            let temp = dir.join("CANARY_replace");
            while !stop_writer.load(Ordering::Relaxed) {
                fs::write(&temp, bytes).unwrap();
                fs::rename(&temp, changing_ref).unwrap();
                changes.fetch_add(1, Ordering::Relaxed);
                std::os::windows::fs::symlink_file(stable_ref, &temp).unwrap();
                fs::rename(&temp, changing_ref).unwrap();
                changes.fetch_add(1, Ordering::Relaxed);
            }
        });
        let result = std::panic::catch_unwind(|| {
            for _ in 0..32 {
                let o = command("plan", "--json", &changing);
                match o.status.code() {
                    Some(3) => {
                        assert!(o.stderr.is_empty());
                        let actual: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
                        let expected: serde_json::Value = serde_json::from_slice(include_bytes!(
                            "../../../fixtures/review-v1/select.expected.json"
                        ))
                        .unwrap();
                        assert_eq!(actual, expected);
                    }
                    Some(2) => refused(o, 2, b"QUENT_REVIEW_INVALID\n"),
                    Some(8) => refused(o, 8, b"QUENT_REVIEW_IO\n"),
                    _ => panic!("unexpected replacement result"),
                }
            }
        });
        stop.store(true, Ordering::Relaxed);
        writer.join().unwrap();
        assert!(replacements.load(Ordering::Relaxed) > 0);
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    });
}
