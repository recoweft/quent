//! Diagnostic-checked compile failures against real public APIs, with a compiling
//! control in the same dependency/feature environment. No missing API is evidence.
#![cfg(all(feature = "postgres", feature = "runtime-tokio"))]
use std::{fs, process::Command};
#[test]
fn owned_handle_compile_contracts() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!("quent-owned-api-{}", std::process::id()));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fs::create_dir_all(dir.join("src")).unwrap();
    let _cleanup = Cleanup(dir.clone());
    let manifest = format!(
        r#"[package]
name = "quent-owned-api-check"
version = "0.0.0"
edition = "2024"
publish = false
[workspace]
[dependencies]
quent = {{ path = {:?}, features = ["postgres", "runtime-tokio"] }}
quent-core = {{ path = {:?} }}
serde_json = "=1.0.154"
sqlx = {{ version = "=0.9.0", default-features = false, features = ["postgres", "runtime-tokio"] }}
"#,
        repo,
        repo.join("../quent-core")
    );
    fs::write(dir.join("Cargo.toml"), manifest).unwrap();
    let prelude = "#![allow(dead_code)]\nuse quent::{PostgresSession,ValidatedOperation,ValidatedAtomicOperation,ExpectedRows,BorrowedPostgresTransaction};\nuse quent_core::{DatabaseContext,Operation};\n";
    let run = |source: &str| {
        fs::write(dir.join("src/lib.rs"), format!("{prelude}{source}")).unwrap();
        Command::new(env!("CARGO"))
            .args(["check", "--offline", "--quiet", "--manifest-path"])
            .arg(dir.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", repo.join("../../target/owned-api"))
            .output()
            .unwrap()
    };
    let output = run(
        "async fn control(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation) { let h=s.validate(c,o,std::time::Duration::from_secs(1)).unwrap(); let _=s.execute_validated(c,o,h).await; }",
    );
    assert!(
        output.status.success(),
        "control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = run(
        "async fn control(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,raw:&mut sqlx::PgConnection) { let e=ExpectedRows::exactly(1).unwrap(); let h=s.validate_atomic(c,o,e,std::time::Duration::from_secs(1)).unwrap(); let _=s.execute_atomic(c,o,e,h).await; use sqlx::Connection; let mut tx=raw.begin().await.unwrap(); let mut b=BorrowedPostgresTransaction::new(&mut tx,c.clone()).unwrap(); let h=b.validate(c,o,std::time::Duration::from_secs(1)).unwrap(); let _=b.execute_validated(c,o,h).await; drop(b);tx.rollback().await.unwrap(); }",
    );
    assert!(
        output.status.success(),
        "atomic/borrowed positive control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for (name, body, code) in [
        (
            "borrowed_atomic_execution",
            "fn f(b:&mut BorrowedPostgresTransaction<'_>,c:&DatabaseContext,o:&Operation,h:ValidatedAtomicOperation){let _=b.execute_atomic(c,o,ExpectedRows::exactly(1).unwrap(),h);}",
            "E0599",
        ),
        (
            "atomic_clone",
            "fn f(h:ValidatedAtomicOperation){let _=h.clone();}",
            "E0599",
        ),
        (
            "atomic_copy",
            "fn f<T:Copy>(){} fn g(){f::<ValidatedAtomicOperation>();}",
            "E0277",
        ),
        (
            "atomic_serialize",
            "fn f(h:&ValidatedAtomicOperation){let _=serde_json::to_string(h);}",
            "E0277",
        ),
        (
            "atomic_deserialize",
            "fn f(){let _:ValidatedAtomicOperation=serde_json::from_str(\"{}\").unwrap();}",
            "E0277",
        ),
        (
            "atomic_reuse",
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedAtomicOperation){let e=ExpectedRows::exactly(1).unwrap();let _=s.execute_atomic(c,o,e,h).await;let _=s.execute_atomic(c,o,e,h).await;}",
            "E0382",
        ),
        (
            "atomic_unpolled",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedAtomicOperation){let e=ExpectedRows::exactly(1).unwrap();drop(s.execute_atomic(c,o,e,h));let _=s.execute_atomic(c,o,e,h);}",
            "E0382",
        ),
        (
            "atomic_failure",
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedAtomicOperation){let e=ExpectedRows::exactly(1).unwrap();if s.execute_atomic(c,o,e,h).await.is_err(){let _=s.execute_atomic(c,o,e,h).await;}}",
            "E0382",
        ),
        (
            "atomic_session_borrow",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedAtomicOperation,next:PostgresSession){let future=s.execute_atomic(c,o,ExpectedRows::exactly(1).unwrap(),h);*s=next;drop(future);}",
            "E0506",
        ),
        (
            "atomic_context_borrow",
            "fn f(s:&mut PostgresSession,c:&mut DatabaseContext,o:&Operation,h:ValidatedAtomicOperation,next:DatabaseContext){let future=s.execute_atomic(c,o,ExpectedRows::exactly(1).unwrap(),h);*c=next;drop(future);}",
            "E0506",
        ),
        (
            "atomic_operation_borrow",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&mut Operation,h:ValidatedAtomicOperation,next:Operation){let future=s.execute_atomic(c,o,ExpectedRows::exactly(1).unwrap(),h);*o=next;drop(future);}",
            "E0506",
        ),
        (
            "atomic_handle_boundary",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedOperation){let _=s.execute_atomic(c,o,ExpectedRows::exactly(1).unwrap(),h);}",
            "E0308",
        ),
        (
            "borrowed_commit_borrow",
            "async fn f(tx:&mut sqlx::Transaction<'_,sqlx::Postgres>,c:DatabaseContext,o:&Operation){let mut b=BorrowedPostgresTransaction::new(tx,c).unwrap();let _=tx.commit().await;let _=b.execute(oops(),o.clone()).await;}fn oops()->&'static DatabaseContext{unimplemented!()}",
            "E0505",
        ),
        (
            "borrowed_lifetime",
            "fn f<'a>(tx:&mut sqlx::Transaction<'_,sqlx::Postgres>,c:DatabaseContext)->BorrowedPostgresTransaction<'a>{BorrowedPostgresTransaction::new(tx,c).unwrap()}",
            "E0621",
        ),
        (
            "clone",
            "fn f(h:ValidatedOperation){let _=h.clone();}",
            "E0599",
        ),
        (
            "copy",
            "fn f<T:Copy>(){} fn g(){f::<ValidatedOperation>();}",
            "E0277",
        ),
        (
            "serialize",
            "fn f(h:&ValidatedOperation){let _=serde_json::to_string(h);}",
            "E0277",
        ),
        (
            "deserialize",
            "fn f(){let _:ValidatedOperation=serde_json::from_str(\"{}\").unwrap();}",
            "E0277",
        ),
        (
            "reuse",
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedOperation){let _=s.execute_validated(c,o,h).await;let _=s.execute_validated(c,o,h).await;}",
            "E0382",
        ),
        (
            "unpolled",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedOperation){let future=s.execute_validated(c,o,h);drop(future);let _=s.execute_validated(c,o,h);}",
            "E0382",
        ),
        (
            "failed",
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedOperation){if s.execute_validated(c,o,h).await.is_err(){let _=s.execute_validated(c,o,h).await;}}",
            "E0382",
        ),
        (
            "debug",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedOperation){let view=format!(\"{h:?}\");let _=s.execute_validated(c,o,view);}",
            "E0308",
        ),
        (
            "plan",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,p:quent_core::Plan){let _=s.execute_validated(c,o,p);}",
            "E0308",
        ),
        (
            "source",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,p:quent_core::SensitiveSource<'_>){let _=s.execute_validated(c,o,p);}",
            "E0308",
        ),
        (
            "restore",
            "fn f(){let _:ValidatedOperation=\"{}\".parse().unwrap();}",
            "E0277",
        ),
        (
            "context_borrow",
            "fn f(s:&mut PostgresSession,c:&mut DatabaseContext,o:&Operation,h:ValidatedOperation,next:DatabaseContext){let future=s.execute_validated(c,o,h);*c=next;drop(future);}",
            "E0506",
        ),
        (
            "operation_borrow",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&mut Operation,h:ValidatedOperation,next:Operation){let future=s.execute_validated(c,o,h);*o=next;drop(future);}",
            "E0506",
        ),
        (
            "session_borrow",
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:ValidatedOperation,next:PostgresSession){let future=s.execute_validated(c,o,h);*s=next;drop(future);}",
            "E0506",
        ),
        (
            "private",
            "fn f(h:ValidatedOperation){let _=h.prepared;}",
            "E0616",
        ),
    ] {
        let output = run(body);
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && diagnostic.contains(&format!("error[{code}]")),
            "{name}: {diagnostic}"
        );
        assert!(
            !diagnostic.contains("E0432") && !diagnostic.contains("E0433"),
            "{name}: missing API/dependency: {diagnostic}"
        );
    }
}
