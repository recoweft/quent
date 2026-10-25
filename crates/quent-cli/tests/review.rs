#[cfg(any(unix, feature = "review", feature = "source-review"))]
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
    #[cfg(any(feature = "review", feature = "source-review"))]
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
    run_in(args, dsn, None)
}
fn run_in(args: &[&OsStr], dsn: &str, directory: Option<&std::path::Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_quent-cli"));
    if let Some(d) = directory {
        command.current_dir(d);
    }
    let mut child = command
        // product executable, never the supplied Rust file
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
    #[cfg(not(feature = "source-review"))]
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
    #[cfg(any(feature = "manifest", feature = "review", feature = "source-review"))]
    rustix::fs::mkfifoat(rustix::fs::CWD, p, rustix::fs::Mode::RUSR).unwrap();
    #[cfg(not(any(feature = "manifest", feature = "review", feature = "source-review")))]
    let _ = p;
}
#[cfg(any(feature = "review", feature = "source-review"))]
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

#[cfg(feature = "source-review")]
#[test]
fn source_product_all_stdout_bytes_stderr_exit_and_grammar() {
    let f = Files::new();
    for (source, compact, pretty) in [
        (
            include_bytes!("../../../fixtures/source-review-v1/delete.rs").as_slice(),
            include_bytes!("../../../fixtures/source-review-v1/delete.expected.json").as_slice(),
            include_bytes!("../../../fixtures/source-review-v1/delete.pretty.expected.json")
                .as_slice(),
        ),
        (
            include_bytes!("../../../fixtures/source-review-v1/filter.rs").as_slice(),
            include_bytes!("../../../fixtures/source-review-v1/filter.expected.json").as_slice(),
            include_bytes!("../../../fixtures/source-review-v1/filter.pretty.expected.json")
                .as_slice(),
        ),
    ] {
        let p = f.file("CANARY_source", source);
        for (fmt, expected) in [("--json", compact), ("--pretty", pretty)] {
            let o = command("rust", fmt, &p);
            assert_eq!(o.status.code(), Some(3));
            assert!(o.stderr.is_empty());
            assert_eq!(o.stdout, expected);
        }
    }
    let p = f.file("CANARY_invalid", b"fn CANARY(){panic!(\"CANARY\");}");
    let o = command("rust", "--json", &p);
    assert_eq!(o.status.code(), Some(4));
    assert!(o.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&o.stdout).contains("CANARY"));
    fs::write(&p, b"fn CANARY(){").unwrap();
    refused(command("rust", "--json", &p), 2, b"QUENT_REVIEW_INVALID\n");
    for args in [
        vec!["review", "rust", "--wrong"],
        vec!["review", "rust", "--json", "-"],
        vec!["review", "rust", "--json", "CANARY", "extra"],
    ] {
        let a: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
        refused(run(&a), 2, b"QUENT_REVIEW_INVALID\n");
    }
}
#[cfg(feature = "source-review")]
#[test]
fn source_subprocess_preparser_stack_and_lexical_adversaries() {
    let f = Files::new();
    let mut cases = vec![
        format!("fn f(){{{}x;}}", "!".repeat(400)),
        format!("type T={}&u8;", "&".repeat(400)),
        format!("type T={}u8{};", "X<".repeat(100), ">".repeat(100)),
        format!("fn f(){{{}x{};}}", "(".repeat(100), ")".repeat(100)),
        format!("m!({});", "a ".repeat(600)),
        format!("/*{}*/", "/*".repeat(100)),
        format!("fn f(){{let x=r#\"{}\"#;}}", " ".repeat(4096)),
        " ".repeat(65536),
        " ".repeat(65537),
        "a".repeat(513),
    ]
    .into_iter()
    .map(|s| (s, 2))
    .collect::<Vec<_>>();
    // Near-envelope recursion goes INTO the production parser, not just refusal.
    for n in [8, 16, 32] {
        cases.push((format!("fn f(){{{}x;}}", "!".repeat(n)), 4));
    }
    cases.push((
        format!("type T={}u8{};", "X<".repeat(16), ">".repeat(16)),
        4,
    ));
    cases.push((format!("type T={}u8;", "&".repeat(32)), 4));
    cases.push((format!("fn f(){{x{};}}", " as T".repeat(80)), 4));
    for (s, expected_exit) in cases {
        let p = f.file("CANARY_adversary", s.as_bytes());
        let o = command("rust", "--json", &p);
        if expected_exit == 2 {
            refused(o, 2, b"QUENT_REVIEW_BUDGET\n");
        } else {
            assert_eq!(o.status.code(), Some(expected_exit));
            assert!(o.stderr.is_empty());
        }
    }
    let p = f.file("CANARY_utf8", b"\xff");
    refused(
        command("rust", "--pretty", &p),
        2,
        b"QUENT_REVIEW_INVALID\n",
    );
    let mut exact = vec![b' '; 4096];
    exact[..13].copy_from_slice(b"fn main() {} ");
    fs::write(&p, &exact).unwrap();
    assert_eq!(command("rust", "--json", &p).status.code(), Some(4));
    exact.push(b' ');
    fs::write(&p, &exact).unwrap();
    refused(command("rust", "--json", &p), 2, b"QUENT_REVIEW_BUDGET\n");
}
#[cfg(all(feature = "source-review", unix))]
#[test]
fn source_fifo_symlink_device_and_grammar_precedence() {
    let f = Files::new();
    let fifo = f.0.join("CANARY_fifo");
    rustix_fifo_when_available(&fifo);
    for p in [&fifo, f.0.as_path(), Path::new("/dev/null")] {
        refused(command("rust", "--json", p), 2, b"QUENT_REVIEW_INVALID\n");
        refused(command("rust", "--wrong", p), 2, b"QUENT_REVIEW_INVALID\n");
    }
    let p = f.file("CANARY_regular", b"fn f(){}");
    let link = f.0.join("CANARY_link");
    std::os::unix::fs::symlink(p, &link).unwrap();
    refused(command("rust", "--json", &link), 8, b"QUENT_REVIEW_IO\n");
}
#[cfg(feature = "source-review")]
#[test]
fn source_zero_connect_with_actual_detecting_control() {
    use std::net::{TcpListener, TcpStream};
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.set_nonblocking(true).unwrap();
    let addr = l.local_addr().unwrap();
    let c = TcpStream::connect(addr).unwrap();
    assert!(l.accept().is_ok());
    drop(c);
    let dsn = format!("postgres://CANARY:CANARY@{addr}/CANARY");
    let f = Files::new();
    let sentinel = f.0.join("CANARY");
    fs::write(&sentinel, b"detecting_control").unwrap();
    assert!(sentinel.is_file());
    fs::remove_file(&sentinel).unwrap();
    for source in [
        "fn f(){quent_core::Draft::delete(t);}",
        "fn f(){std::process::abort();}",
        "fn CANARY(){",
        "fn f(){std::fs::write(\"CANARY\",\"CANARY\");}",
    ] {
        let p = f.file("CANARY_source", source.as_bytes());
        let o = run_in(
            &[
                OsStr::new("review"),
                OsStr::new("rust"),
                OsStr::new("--json"),
                p.as_os_str(),
            ],
            &dsn,
            Some(&f.0),
        );
        assert!(matches!(o.status.code(), Some(2..=4)));
        assert!(!String::from_utf8_lossy(&o.stdout).contains("CANARY"));
        assert_eq!(
            l.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    assert!(!f.0.join("CANARY").exists());
}

#[cfg(all(feature = "source-review", unix))]
#[test]
fn source_path_replacement_keeps_actual_opened_object_safety() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let f = Files::new();
    let b = include_bytes!("../../../fixtures/source-review-v1/delete.rs");
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
                let o = command("rust", "--json", &changing);
                match o.status.code() {
                    Some(3) => {
                        assert!(o.stderr.is_empty());
                        let j: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
                        let expected: serde_json::Value = serde_json::from_slice(include_bytes!(
                            "../../../fixtures/source-review-v1/delete.expected.json"
                        ))
                        .unwrap();
                        assert_eq!(j, expected);
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
#[cfg(all(feature = "source-review", windows))]
#[test]
fn source_windows_regular_directory_device_and_final_reparse_refuse() {
    let f = Files::new();
    let p = f.file(
        "CANARY_regular",
        include_bytes!("../../../fixtures/source-review-v1/delete.rs"),
    );
    assert_eq!(command("rust", "--json", &p).status.code(), Some(3));
    let link = f.0.join("CANARY_reparse");
    std::os::windows::fs::symlink_file(&p, &link).unwrap();
    for p in [f.0.as_path(), Path::new("NUL"), link.as_path()] {
        let o = command("rust", "--json", p);
        assert!(matches!(o.status.code(), Some(2 | 8)));
        assert!(o.stdout.is_empty());
        assert!(o.stderr == b"QUENT_REVIEW_INVALID\n" || o.stderr == b"QUENT_REVIEW_IO\n");
    }
}

#[cfg(all(feature = "source-review", windows))]
#[test]
fn source_windows_path_replacement_keeps_opened_handle_safety() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };
    let f = Files::new();
    let bytes = include_bytes!("../../../fixtures/source-review-v1/delete.rs");
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
                let o = command("rust", "--json", &changing);
                match o.status.code() {
                    Some(3) => {
                        assert!(o.stderr.is_empty());
                        let actual: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
                        let expected: serde_json::Value = serde_json::from_slice(include_bytes!(
                            "../../../fixtures/source-review-v1/delete.expected.json"
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

#[cfg(feature = "review-ci")]
fn ci_command(format: &str, paths: [&Path; 3]) -> Output {
    run(&[
        OsStr::new("review-ci"),
        OsStr::new(format),
        paths[0].as_os_str(),
        paths[1].as_os_str(),
        paths[2].as_os_str(),
    ])
}
#[test]
fn ci_grammar_features_precede_all_three_reads() {
    let f = Files::new();
    let p = f.0.join("CANARY_writerless");
    #[cfg(unix)]
    rustix_fifo_when_available(&p);
    let s = p.as_os_str();
    for args in [
        vec![OsStr::new("review-ci")],
        vec![OsStr::new("review-ci"), OsStr::new("--unknown"), s, s, s],
        vec![OsStr::new("review-ci"), OsStr::new("--json"), s, s],
        vec![OsStr::new("review-ci"), OsStr::new("--json"), s, s, s, s],
    ] {
        refused(run(&args), 2, b"QUENT_REVIEW_INVALID\n");
    }
    for position in 0..3 {
        let mut paths = [s, s, s];
        paths[position] = OsStr::new("-");
        refused(
            run(&[
                OsStr::new("review-ci"),
                OsStr::new("--json"),
                paths[0],
                paths[1],
                paths[2],
            ]),
            2,
            b"QUENT_REVIEW_INVALID\n",
        );
    }
    #[cfg(not(feature = "review-ci"))]
    for fmt in ["--json", "--pretty", "--sarif"] {
        refused(
            run(&[OsStr::new("review-ci"), OsStr::new(fmt), s, s, s]),
            4,
            b"QUENT_REVIEW_UNSUPPORTED\n",
        );
    }
}
#[cfg(feature = "review-ci")]
fn ci_files(f: &Files) -> [PathBuf; 3] {
    [
        f.file(
            "CANARY_head",
            include_bytes!("../../../fixtures/review-ci-v1/empty.json"),
        ),
        f.file(
            "CANARY_base",
            include_bytes!("../../../fixtures/review-ci-v1/empty.json"),
        ),
        f.file(
            "CANARY_baseline",
            include_bytes!("../../../fixtures/review-ci-v1/baseline.json"),
        ),
    ]
}
#[cfg(feature = "review-ci")]
#[test]
fn ci_independent_sarif_json_pretty_goldens_and_exits() {
    let f = Files::new();
    let paths = ci_files(&f);
    for (fmt, expected) in [
        (
            "--json",
            include_bytes!("../../../fixtures/review-ci-v1/empty.expected.json").as_slice(),
        ),
        (
            "--pretty",
            include_bytes!("../../../fixtures/review-ci-v1/empty.pretty.expected.json").as_slice(),
        ),
        (
            "--sarif",
            include_bytes!("../../../fixtures/review-ci-v1/empty.sarif.expected.json").as_slice(),
        ),
    ] {
        let o = ci_command(fmt, [&paths[0], &paths[1], &paths[2]]);
        assert_eq!(o.status.code(), Some(3));
        assert!(o.stderr.is_empty());
        assert_eq!(o.stdout, expected);
    }
}
#[cfg(feature = "review-ci")]
#[test]
fn ci_all_inputs_bytes_utf8_closed_wire_and_resource_refusals() {
    let f = Files::new();
    let paths = ci_files(&f);
    for n in 0..3 {
        let original = fs::read(&paths[n]).unwrap();
        let mut exact = original.clone();
        exact.resize(65536, b' ');
        fs::write(&paths[n], &exact).unwrap();
        assert_eq!(
            ci_command("--json", [&paths[0], &paths[1], &paths[2]])
                .status
                .code(),
            Some(3)
        );
        exact.push(b' ');
        fs::write(&paths[n], exact).unwrap();
        refused(
            ci_command("--sarif", [&paths[0], &paths[1], &paths[2]]),
            2,
            b"QUENT_REVIEW_BUDGET\n",
        );
        for b in [
            vec![255],
            br#"{"kind":1,"k\u0069nd":2}"#.to_vec(),
            b"null".to_vec(),
            format!("{}0{}", "[".repeat(100), "]".repeat(100)).into_bytes(),
            br#"{"kind":"quent_review_ci","version":1}"#.to_vec(),
            b"{} {}".to_vec(),
        ] {
            fs::write(&paths[n], b).unwrap();
            let o = ci_command("--pretty", [&paths[0], &paths[1], &paths[2]]);
            assert!(!o.status.success());
            assert!(o.stdout.is_empty());
            assert!(
                o.stderr == b"QUENT_REVIEW_INVALID\n" || o.stderr == b"QUENT_REVIEW_UNSUPPORTED\n"
            );
        }
        fs::write(&paths[n], original).unwrap();
    }
}
#[cfg(feature = "review-ci")]
#[test]
fn ci_suppressed_claim_cannot_hide_block_unknown_or_removed_gate() {
    use serde_json::json;
    let f = Files::new();
    let paths = ci_files(&f);
    let mut source: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/review-v1/select.json")).unwrap();
    source["operation"]["family"] = json!("delete");
    source["operation"]["projection"] = json!([]);
    source["operation"]
        .as_object_mut()
        .unwrap()
        .remove("filter");
    source["operation"].as_object_mut().unwrap().remove("limit");
    source["claims"] = json!({"sql":"https://CANARY.invalid/\u{1b}[31m [CANARY](file:///secret)","coverage":"complete","verdict":"accepted","freshness":"fresh","fingerprint":"CANARY_hash"});
    let input = json!({"target":"CANARY_target","context":"CANARY_context","scope":"CANARY_scope","format":"plan","source":source.to_string()});
    let head = json!({"kind":"quent_review_inputs","version":1,"analysis_version":1,"selection":"changed_only","inputs":[input]});
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let baseline = json!({"kind":"quent_review_baseline","version":1,"analysis_version":1,"entries":[{"code":"untrusted_claims_present","input":input,"reason":"CANARY_reason","owner":"CANARY_owner","expires":now+600}]});
    fs::write(&paths[0], head.to_string()).unwrap();
    fs::write(&paths[2], baseline.to_string()).unwrap();
    for fmt in ["--json", "--pretty", "--sarif"] {
        let o = ci_command(fmt, [&paths[0], &paths[1], &paths[2]]);
        assert_eq!(o.status.code(), Some(5));
        assert!(o.stderr.is_empty());
        let text = String::from_utf8(o.stdout).unwrap();
        for canary in [
            "CANARY",
            "https://",
            "file://",
            "fingerprint",
            "owner",
            "reason",
        ] {
            assert!(!text.contains(canary));
        }
        assert!(text.contains("missing_where"));
        assert!(text.contains("review_incomplete"));
        if fmt != "--sarif" {
            let v: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(v["hidden_count"], 1);
            assert_eq!(v["head_results"][0]["risk"], "blocked");
            assert_eq!(v["git_head"], "unverified");
        }
    }
    // Removed display cannot certify a missing head input set.
    fs::write(&paths[1], head.to_string()).unwrap();
    fs::write(
        &paths[0],
        include_bytes!("../../../fixtures/review-ci-v1/empty.json"),
    )
    .unwrap();
    let o = ci_command("--json", [&paths[0], &paths[1], &paths[2]]);
    assert_eq!(o.status.code(), Some(3));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["explicit_set"], "missing");
    assert_eq!(v["diagnostics"][0]["diff"], "removed");
}
#[cfg(all(feature = "review-ci", unix))]
#[test]
fn ci_all_three_inputs_fifo_device_directory_symlink_refuse() {
    let f = Files::new();
    let paths = ci_files(&f);
    let fifo = f.0.join("CANARY_fifo");
    rustix_fifo_when_available(&fifo);
    let link = f.0.join("CANARY_link");
    std::os::unix::fs::symlink(&paths[0], &link).unwrap();
    for n in 0..3 {
        for bad in [&fifo, f.0.as_path(), Path::new("/dev/null")] {
            let mut p = [paths[0].as_path(), paths[1].as_path(), paths[2].as_path()];
            p[n] = bad;
            refused(ci_command("--json", p), 2, b"QUENT_REVIEW_INVALID\n");
        }
        let mut p = [paths[0].as_path(), paths[1].as_path(), paths[2].as_path()];
        p[n] = &link;
        refused(ci_command("--sarif", p), 8, b"QUENT_REVIEW_IO\n");
    }
}
#[cfg(all(feature = "review-ci", windows))]
#[test]
fn ci_all_three_inputs_windows_device_directory_reparse_refuse() {
    let f = Files::new();
    let paths = ci_files(&f);
    let link = f.0.join("CANARY_link");
    std::os::windows::fs::symlink_file(&paths[0], &link).unwrap();
    for n in 0..3 {
        for bad in [f.0.as_path(), Path::new("NUL"), link.as_path()] {
            let mut p = [paths[0].as_path(), paths[1].as_path(), paths[2].as_path()];
            p[n] = bad;
            let o = ci_command("--json", p);
            assert!(matches!(o.status.code(), Some(2 | 8)));
            assert!(o.stdout.is_empty());
            assert!(o.stderr == b"QUENT_REVIEW_INVALID\n" || o.stderr == b"QUENT_REVIEW_IO\n");
        }
    }
}
#[cfg(feature = "review-ci")]
#[test]
fn ci_all_three_replacement_races_preserve_opened_file_and_fixed_streams() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };
    let f = Files::new();
    let paths = ci_files(&f);
    for n in 0..3 {
        let content = fs::read(&paths[n]).unwrap();
        let stable = f.file("CANARY_stable_ci", &content);
        let changing = f.file("CANARY_changing_ci", &content);
        let stop = Arc::new(AtomicBool::new(false));
        let changes = Arc::new(AtomicUsize::new(0));
        std::thread::scope(|scope| {
            let stop_writer = stop.clone();
            let counter = changes.clone();
            let changing_ref = &changing;
            let stable_ref = &stable;
            let dir = &f.0;
            let content = &content;
            let writer = scope.spawn(move || {
                let tmp = dir.join("CANARY_replace_ci");
                while !stop_writer.load(Ordering::Relaxed) {
                    fs::write(&tmp, content).unwrap();
                    fs::rename(&tmp, changing_ref).unwrap();
                    counter.fetch_add(1, Ordering::Relaxed);
                    #[cfg(unix)]
                    {
                        rustix::fs::mkfifoat(rustix::fs::CWD, &tmp, rustix::fs::Mode::RUSR)
                            .unwrap();
                        fs::rename(&tmp, changing_ref).unwrap();
                    }
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(stable_ref, &tmp).unwrap();
                    #[cfg(windows)]
                    std::os::windows::fs::symlink_file(stable_ref, &tmp).unwrap();
                    fs::rename(&tmp, changing_ref).unwrap();
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            });
            let result = std::panic::catch_unwind(|| {
                for _ in 0..24 {
                    let mut p = [paths[0].as_path(), paths[1].as_path(), paths[2].as_path()];
                    p[n] = &changing;
                    let o = ci_command("--json", p);
                    match o.status.code() {
                        Some(3) => {
                            assert!(o.stderr.is_empty());
                            assert_eq!(
                                o.stdout,
                                include_bytes!(
                                    "../../../fixtures/review-ci-v1/empty.expected.json"
                                )
                            );
                        }
                        Some(2) => refused(o, 2, b"QUENT_REVIEW_INVALID\n"),
                        Some(8) => refused(o, 8, b"QUENT_REVIEW_IO\n"),
                        _ => panic!("unexpected CI race result"),
                    }
                }
            });
            stop.store(true, Ordering::Relaxed);
            writer.join().unwrap();
            assert!(changes.load(Ordering::Relaxed) > 0);
            if let Err(e) = result {
                std::panic::resume_unwind(e);
            }
        });
    }
}
#[cfg(feature = "review-ci")]
#[test]
fn ci_zero_network_with_real_detecting_control() {
    use std::net::{TcpListener, TcpStream};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let c = TcpStream::connect(addr).unwrap();
    listener.accept().unwrap();
    drop(c);
    let f = Files::new();
    let p = ci_files(&f);
    let args = [
        OsStr::new("review-ci"),
        OsStr::new("--sarif"),
        p[0].as_os_str(),
        p[1].as_os_str(),
        p[2].as_os_str(),
    ];
    let o = run_at(&args, &format!("postgres://CANARY:CANARY@{addr}/CANARY"));
    assert_eq!(o.status.code(), Some(3));
    let mut invalid = args.to_vec();
    invalid.push(OsStr::new("extra"));
    refused(run_at(&invalid, "CANARY"), 2, b"QUENT_REVIEW_INVALID\n");
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
