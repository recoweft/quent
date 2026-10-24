#![cfg(feature = "manifest")]
use std::{fs, path::PathBuf, process::Command};
struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "quent-cli-verify-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let p = self.0.join(name);
        fs::write(&p, bytes).unwrap();
        p
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn run(args: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_quent-cli"))
        .args(args)
        .env("PGOPTIONS", "CANARY_AMBIENT_CONFIGURATION")
        .env("PGPASSWORD", "CANARY_AMBIENT_PASSWORD")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map(watch)
        .unwrap()
}
// Every product subprocess has a finite watchdog, including the FIFO regression.
// A timeout is failure after kill + wait, never a skip or a writer-based rescue.
fn watch(mut child: std::process::Child) -> std::process::Output {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("CLI exceeded watchdog; child killed and reaped");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
fn complete_source() -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/manifest-v1/postgres.json"
    ))
    .unwrap();
    value.as_object_mut().unwrap().remove("generated_code");
    value.as_object_mut().unwrap().remove("live_observation");
    serde_json::to_vec(&value).unwrap()
}
fn rejected(o: std::process::Output, code: i32, diagnostic: &[u8]) {
    assert_eq!(o.status.code(), Some(code));
    assert!(o.stdout.is_empty());
    assert_eq!(o.stderr, diagnostic);
}
#[cfg(unix)]
#[test]
fn writerless_fifo_and_special_inputs_refuse_before_read() {
    let f = Files::new();
    let a = f.write("CANARY_source", &complete_source());
    let fifo = f.0.join("CANARY_fifo");
    rustix::fs::mkfifoat(rustix::fs::CWD, &fifo, rustix::fs::Mode::RUSR).unwrap();
    // No writer is ever connected, even on failure.
    for p in [&fifo, &f.0, std::path::Path::new("/dev/null")] {
        rejected(command("--offline", p, &a), 2, b"QUENT_VERIFY_INVALID\n");
        rejected(command("--offline", &a, p), 2, b"QUENT_VERIFY_INVALID\n");
        #[cfg(all(
            feature = "runtime-tokio",
            any(feature = "postgres", feature = "mysql")
        ))]
        rejected(command("--live", &a, p), 2, b"QUENT_VERIFY_INVALID\n");
    }
    assert_eq!(command("--offline", &a, &a).status.code(), Some(0));
    // Final symlinks refuse, including one pointing at a regular file.
    let link = f.0.join("CANARY_link");
    std::os::unix::fs::symlink(&a, &link).unwrap();
    rejected(command("--offline", &link, &a), 8, b"QUENT_VERIFY_IO\n");
}
#[cfg(windows)]
#[test]
fn windows_directory_and_device_inputs_refuse() {
    let f = Files::new();
    let a = f.write("CANARY_source", &complete_source());
    for p in [f.0.as_path(), std::path::Path::new("NUL")] {
        for o in [command("--offline", p, &a), command("--offline", &a, p)] {
            assert!(matches!(o.status.code(), Some(2 | 8)));
            assert!(o.stdout.is_empty());
            assert!(o.stderr == b"QUENT_VERIFY_INVALID\n" || o.stderr == b"QUENT_VERIFY_IO\n");
        }
        #[cfg(all(
            feature = "runtime-tokio",
            any(feature = "postgres", feature = "mysql")
        ))]
        {
            let o = command("--live", &a, p);
            assert!(matches!(o.status.code(), Some(2 | 8)));
            assert!(o.stdout.is_empty());
            assert!(o.stderr == b"QUENT_VERIFY_INVALID\n" || o.stderr == b"QUENT_VERIFY_IO\n");
        }
    }
}
#[test]
fn exact_file_byte_limits_are_preserved() {
    let f = Files::new();
    let mut bytes = complete_source();
    bytes.resize(65_536, b' ');
    let a = f.write("CANARY_limit", &bytes);
    assert_eq!(command("--offline", &a, &a).status.code(), Some(0));
    bytes.push(b' ');
    let huge = f.write("CANARY_over_limit", &bytes);
    rejected(command("--offline", &huge, &a), 2, b"QUENT_VERIFY_BUDGET\n");
    rejected(command("--offline", &a, &huge), 2, b"QUENT_VERIFY_BUDGET\n");
}
#[cfg(unix)]
#[test]
fn path_replacement_never_turns_fifo_or_symlink_into_a_read() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let f = Files::new();
    let a = f.write("CANARY_stable", &complete_source());
    let changing = f.write("CANARY_changing", &complete_source());
    let stop = Arc::new(AtomicBool::new(false));
    std::thread::scope(|scope| {
        let stop_writer = stop.clone();
        let changing_ref = &changing;
        let stable = &a;
        let dir = &f.0;
        let writer = scope.spawn(move || {
            let temp = dir.join("CANARY_replace");
            while !stop_writer.load(Ordering::Relaxed) {
                fs::write(&temp, complete_source()).unwrap();
                fs::rename(&temp, changing_ref).unwrap();
                rustix::fs::mkfifoat(rustix::fs::CWD, &temp, rustix::fs::Mode::RUSR).unwrap();
                fs::rename(&temp, changing_ref).unwrap();
                std::os::unix::fs::symlink(stable, &temp).unwrap();
                fs::rename(&temp, changing_ref).unwrap();
            }
        });
        // Ensure a failed CLI assertion still stops/joins the mutator before cleanup.
        let result = std::panic::catch_unwind(|| {
            for _ in 0..32 {
                let o = command("--offline", &changing, &a);
                match o.status.code() {
                    Some(0) => assert!(o.stderr.is_empty()),
                    Some(2) => rejected(o, 2, b"QUENT_VERIFY_INVALID\n"),
                    Some(8) => rejected(o, 8, b"QUENT_VERIFY_IO\n"),
                    _ => panic!("unexpected replacement result"),
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
fn command(mode: &str, a: &std::path::Path, b: &std::path::Path) -> std::process::Output {
    run(&[
        "manifest".as_ref(),
        "verify".as_ref(),
        mode.as_ref(),
        a.as_os_str(),
        b.as_os_str(),
    ])
}
#[test]
fn offline_and_invalid_inputs_are_bounded_sourcefree() {
    let f = Files::new();
    let source = include_bytes!("../../../fixtures/manifest-v1/postgres.json");
    let unknown = f.write("CANARY_unknown_source", source);
    let o = command("--offline", &unknown, &unknown);
    assert_eq!(o.status.code(), Some(3));
    let mut complete: serde_json::Value = serde_json::from_slice(source).unwrap();
    complete.as_object_mut().unwrap().remove("generated_code");
    complete.as_object_mut().unwrap().remove("live_observation");
    let a = f.write("CANARY_manifest", &serde_json::to_vec(&complete).unwrap());
    let output = command("--offline", &a, &a);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout,b"{\"kind\":\"quent_verify_offline\",\"version\":1,\"state\":\"OfflineConsistent\",\"live\":\"Unverified\"}\n");
    assert!(output.stderr.is_empty());
    let invalid = f.write("CANARY_invalid", b"{broken CANARY_credential");
    let huge = f.write("CANARY_huge", &vec![b'x'; 65_537]);
    for p in [&invalid, &huge] {
        let o = command("--offline", p, &a);
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&o.stderr).contains("CANARY"));
    }
    let o = run(&[
        "manifest".as_ref(),
        "verify".as_ref(),
        "--offline".as_ref(),
        a.as_os_str(),
        a.as_os_str(),
        "--CANARY_unknown".as_ref(),
    ]);
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stdout.is_empty());
    #[cfg(not(all(
        feature = "runtime-tokio",
        any(feature = "postgres", feature = "mysql")
    )))]
    {
        let o = command("--live", &a, &f.0.join("does_not_exist"));
        assert_eq!(o.status.code(), Some(4));
        assert_eq!(o.stderr, b"QUENT_VERIFY_UNSUPPORTED\n");
    }
}
#[cfg(all(unix, feature = "runtime-tokio", feature = "postgres"))]
#[test]
fn live_config_rejection_has_zero_connect_with_detecting_control() {
    use serde_json::json;
    let f = Files::new();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("127.0.0.1:{port}");
    let mut source: serde_json::Value = serde_json::from_slice(&complete_source()).unwrap();
    source.as_object_mut().unwrap().remove("policy");
    source["schema"]["target"]["address"] = json!(address);
    let a = f.write("CANARY_source", &serde_json::to_vec(&source).unwrap());
    let config = json!({"transport":"tcp_plaintext","dialect":"postgres","host":"127.0.0.1","port":port,"user":"CANARY_user","password":"CANARY_password","database":"db","address":address,"catalog":"catalog","namespace":"public","tables":["items"],"generation":1,"timeout_ms":100,"ttl_ns":30000000000u64});
    let mut bytes = serde_json::to_vec(&config).unwrap();
    bytes.resize(8192, b' ');
    let cfg = f.write("CANARY_config", &bytes);
    // Positive control: the same real CLI accepts an exactly 8KiB regular config,
    // reaches this listener, and fails the incomplete PostgreSQL handshake safely.
    rejected(command("--live", &a, &cfg), 6, b"QUENT_VERIFY_STALE\n");
    let (connection, _) = listener.accept().expect("control must detect real connect");
    drop(connection);
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    bytes.push(b' ');
    let huge = f.write("CANARY_huge_config", &bytes);
    rejected(command("--live", &a, &huge), 2, b"QUENT_VERIFY_BUDGET\n");
    let fifo = f.0.join("CANARY_config_fifo");
    rustix::fs::mkfifoat(rustix::fs::CWD, &fifo, rustix::fs::Mode::RUSR).unwrap();
    rejected(command("--live", &a, &fifo), 2, b"QUENT_VERIFY_INVALID\n");
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
#[cfg(all(feature = "runtime-tokio", feature = "postgres", feature = "mysql"))]
#[test]
fn live_cli_both_catalogs_and_safe_exits() {
    use serde_json::json;
    use sqlx::{ConnectOptions, Connection};
    assert_eq!(
        std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),
        Ok("quent_dispatch_test")
    );
    let f = Files::new();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    for pg in [true, false] {
        let dsn = std::env::var(if pg {
            "QUENT_TEST_POSTGRES_DSN"
        } else {
            "QUENT_TEST_MYSQL_DSN"
        })
        .unwrap();
        let (host, port) = if pg {
            let o: sqlx::postgres::PgConnectOptions = dsn.parse().unwrap();
            (o.get_host().to_owned(), o.get_port())
        } else {
            let o: sqlx::mysql::MySqlConnectOptions = dsn.parse().unwrap();
            (o.get_host().to_owned(), o.get_port())
        };
        let dialect = if pg { "postgres" } else { "mysql" };
        let ns = if pg { "public" } else { "quent_dispatch_test" };
        let address = format!("{host}:{port}");
        let source = json!({"kind":"quent_manifest_source","version":1,"schema":{"kind":"quent_schema","version":1,"target":{"database":"probe","address":address,"catalog":"quent_dispatch_test","dialect":dialect},"coverage":"complete","tables":[{"path":[ns,"cli_manifest_probe"],"semantics":"ordinary","columns_coverage":"complete","keys_coverage":"complete","columns":[{"name":"CANARY_column","native":"i64","nullability":"nonnull","default":{"state":"literal","value":{"type":"i64","value":0}}}],"keys":[]}]}});
        let mut source = source;
        // MySQL preserves the identifier, PostgreSQL's fixture uses quoted spelling.
        let source_path = f.write("CANARY_source", &serde_json::to_vec(&source).unwrap());
        let cfg = json!({"transport":"tcp_plaintext","dialect":dialect,"host":host,"port":port,"user":"quent_test","password":"","database":"probe","address":address,"catalog":"quent_dispatch_test","namespace":ns,"tables":["cli_manifest_probe"],"generation":1,"timeout_ms":5000,"ttl_ns":30000000000u64});
        let cfg_path = f.write("CANARY_connection", &serde_json::to_vec(&cfg).unwrap());
        rt.block_on(async {
            let ddl = if pg {
                "CREATE TABLE cli_manifest_probe(\"CANARY_column\" bigint NOT NULL DEFAULT 0)"
            } else {
                "CREATE TABLE cli_manifest_probe(CANARY_column bigint NOT NULL DEFAULT 0)"
            };
            if pg {
                let o: sqlx::postgres::PgConnectOptions = dsn.parse().unwrap();
                let mut c = sqlx::PgConnection::connect_with(&o.disable_statement_logging())
                    .await
                    .unwrap();
                sqlx::query(ddl).execute(&mut c).await.unwrap();
            } else {
                let o: sqlx::mysql::MySqlConnectOptions = dsn.parse().unwrap();
                let mut c = sqlx::MySqlConnection::connect_with(&o.disable_statement_logging())
                    .await
                    .unwrap();
                sqlx::query(ddl).execute(&mut c).await.unwrap();
            }
        });
        let o = command("--live", &source_path, &cfg_path);
        assert_eq!(
            o.status.code(),
            Some(0),
            "{dialect}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        assert!(o.stderr.is_empty());
        let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(v["schema"], "Matched");
        assert_eq!(v["policy"], "Unverified");
        assert!(!String::from_utf8_lossy(&o.stdout).contains("CANARY"));
        let saved = source.clone();
        source["schema"]["coverage"] = json!("unknown");
        f.write("CANARY_source", &serde_json::to_vec(&source).unwrap());
        let o = command("--live", &source_path, &cfg_path);
        assert_eq!(o.status.code(), Some(3));
        assert!(!String::from_utf8_lossy(&o.stdout).contains("CANARY"));
        source["schema"]["coverage"] = json!("complete");
        source["schema"]["tables"][0]["semantics"] = json!("unsupported");
        f.write("CANARY_source", &serde_json::to_vec(&source).unwrap());
        let o = command("--live", &source_path, &cfg_path);
        assert_eq!(o.status.code(), Some(4));
        source = saved;
        f.write("CANARY_source", &serde_json::to_vec(&source).unwrap());
        let mut short = cfg.clone();
        short["ttl_ns"] = json!(1);
        f.write("CANARY_connection", &serde_json::to_vec(&short).unwrap());
        let o = command("--live", &source_path, &cfg_path);
        assert_eq!(o.status.code(), Some(6));
        assert_eq!(o.stderr, b"QUENT_VERIFY_STALE\n");
        let mut bad = cfg.clone();
        bad["user"] = json!("CANARY_DRIVER_USER");
        bad["password"] = json!("CANARY_CREDENTIAL");
        f.write("CANARY_connection", &serde_json::to_vec(&bad).unwrap());
        let o = command("--live", &source_path, &cfg_path);
        assert_eq!(o.status.code(), Some(8));
        assert_eq!(o.stderr, b"QUENT_VERIFY_CONNECT\n");
        assert!(o.stdout.is_empty());
        f.write("CANARY_connection", &vec![b'x'; 8193]);
        let o = command("--live", &source_path, &cfg_path);
        assert_eq!(o.status.code(), Some(2));
        assert_eq!(o.stderr, b"QUENT_VERIFY_BUDGET\n");
        f.write("CANARY_connection", &serde_json::to_vec(&cfg).unwrap());
        source["schema"]["tables"][0]["columns"][0]["native"] = json!("i32");
        source["schema"]["tables"][0]["columns"][0]["default"]["value"]["type"] = json!("i32");
        f.write("CANARY_source", &serde_json::to_vec(&source).unwrap());
        let o = command("--live", &source_path, &cfg_path);
        assert_eq!(o.status.code(), Some(5));
        assert!(!String::from_utf8_lossy(&o.stdout).contains("CANARY"));
        source["schema"]["target"]["address"] = json!("wrong:1");
        f.write("CANARY_source", &serde_json::to_vec(&source).unwrap());
        let o = command("--live", &source_path, &cfg_path);
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.is_empty());
        assert_eq!(o.stderr, b"QUENT_VERIFY_TARGET\n");
        rt.block_on(async {
            if pg {
                let o: sqlx::postgres::PgConnectOptions = dsn.parse().unwrap();
                let mut c = sqlx::PgConnection::connect_with(&o.disable_statement_logging())
                    .await
                    .unwrap();
                sqlx::query("DROP TABLE cli_manifest_probe")
                    .execute(&mut c)
                    .await
                    .unwrap();
            } else {
                let o: sqlx::mysql::MySqlConnectOptions = dsn.parse().unwrap();
                let mut c = sqlx::MySqlConnection::connect_with(&o.disable_statement_logging())
                    .await
                    .unwrap();
                sqlx::query("DROP TABLE cli_manifest_probe")
                    .execute(&mut c)
                    .await
                    .unwrap();
            }
        });
    }
}
