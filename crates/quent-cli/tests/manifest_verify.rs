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
        .output()
        .unwrap()
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
