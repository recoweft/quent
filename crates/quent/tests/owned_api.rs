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
quent = {{ path = {:?}, features = ["postgres", "runtime-tokio", "local-details", "output-tracing", "manifest", "operation-spec", "review", "source-review", "review-ci"] }}
quent-core = {{ path = {:?} }}
serde_json = "=1.0.154"
sqlx = {{ version = "=0.9.0", default-features = false, features = ["postgres", "runtime-tokio"] }}
"#,
        repo,
        repo.join("../quent-core")
    );
    fs::write(dir.join("Cargo.toml"), manifest).unwrap();
    let prelude = "#![allow(dead_code)]\nuse quent::{PostgresSession,ValidatedOperation,ValidatedAtomicOperation,ExpectedRows,BorrowedPostgresTransaction};\nuse quent_core::{DatabaseContext,Operation};\n";
    let runs = std::cell::Cell::new(0);
    let controls = std::cell::Cell::new(0);
    let run = |source: &str| {
        runs.set(runs.get() + 1);
        fs::write(dir.join("src/lib.rs"), format!("{prelude}{source}")).unwrap();
        let result = Command::new(env!("CARGO"))
            .args(["check", "--offline", "--quiet", "--manifest-path"])
            .arg(dir.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", repo.join("../../target/owned-api"))
            .output()
            .unwrap();
        if result.status.success() {
            controls.set(controls.get() + 1);
        }
        result
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
    let output = run(
        "fn control(p:&quent_core::Plan,i:&quent_core::Inspection<'_>,p2:&quent_core::PolicyPreparation,e:quent_core::PlanError,x:quent::ExecutionError,r:&quent::ExecutionResult){let v=p.public_view();let _:quent_core::PlanView=i.public_view();let _=p2.public_view();let _:serde_json::Value=serde_json::from_str(&v.to_json()).unwrap();let _=e.public_view().to_string();let _=x.public_view();let _=r.public_view();}",
    );
    assert!(
        output.status.success(),
        "view control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = run(
        "fn control(p:&quent_core::Plan,i:&quent_core::Inspection<'_>,q:&quent_core::PolicyPreparation,path:&std::path::Path){use quent::local_details::*;let _=create_new(LocalOptIn::acknowledge_sensitive_local_file(),Surface::LocalApplication,path,p.sensitive_source());let _=create_new(LocalOptIn::acknowledge_sensitive_local_file(),Surface::LocalApplication,path,i.sensitive_source());let _=create_new(LocalOptIn::acknowledge_sensitive_local_file(),Surface::LocalApplication,path,q.sensitive_source().final_source());quent::output_tracing::emit(&p.public_view());}",
    );
    assert!(
        output.status.success(),
        "detail/trace control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = run(
        "fn control(bytes:&[u8]) { use quent::manifest::*; let s=ManifestSource::parse(bytes).unwrap(); let _:ManifestSummary=s.public_view(); let source=s.sensitive_source(); let _=source.canonical_source(); let _=source.fingerprint(ArtifactKind::Schema); let _=s.compare_offline(&s).public_view().to_json(); }",
    );
    assert!(
        output.status.success(),
        "manifest control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = run(
        "async fn control(r:&mut quent::manifest::live::PostgresReader,c:&DatabaseContext,s:&quent::manifest::live::Scope,m:&quent::manifest::ManifestSource,b:quent::manifest::live::Budget) {let o=r.observe(c,s,m,b,std::time::Duration::from_secs(1)).await.unwrap();let _=r.compare(c,s,m,&o).to_json();}",
    );
    assert!(
        output.status.success(),
        "live reader control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for carrier in [
        "quent::manifest::live::Observation",
        "quent::manifest::live::VerifySummary",
        "quent::manifest::live::Scope",
    ] {
        for authority in [
            "quent_core::ApplicationContext",
            "quent_core::SchemaSnapshot",
            "quent_core::ScopeFacts",
            "DatabaseContext",
            "ValidatedOperation",
            "ValidatedAtomicOperation",
        ] {
            let r = run(&format!("fn f(v:{carrier}){{let _:{authority}=v.into();}}"));
            assert!(!r.status.success());
            assert!(String::from_utf8_lossy(&r.stderr).contains("E0277"));
        }
    }
    let r = run(
        "fn f(){let _:quent::manifest::live::Observation=serde_json::from_str(\"{}\").unwrap();}",
    );
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("E0277"));
    let r = run("fn f(o:&quent::manifest::live::Observation){serde_json::to_string(o).unwrap();}");
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("E0277"));
    for body in [
        "fn f(o:&quent::manifest::live::Observation){let _=&o.observed;}",
        "fn f(r:&mut quent::manifest::live::PostgresReader){let _=&mut r.connection;}",
    ] {
        let r = run(body);
        assert!(!r.status.success());
        assert!(String::from_utf8_lossy(&r.stderr).contains("error[E0616]"));
    }
    for carrier in [
        "quent::manifest::ManifestSource",
        "quent::manifest::SensitiveManifestSource<'_>",
        "quent::manifest::ManifestSummary",
        "quent::manifest::Fingerprint",
        "quent::manifest::OfflineComparison",
    ] {
        for authority in [
            "quent_core::ApplicationContext",
            "quent_core::SchemaSnapshot",
            "DatabaseContext",
            "ValidatedOperation",
            "ValidatedAtomicOperation",
            "quent::manifest::live::Observation",
        ] {
            let r = run(&format!("fn f(v:{carrier}){{let _:{authority}=v.into();}}"));
            let d = String::from_utf8_lossy(&r.stderr);
            assert!(
                !r.status.success() && d.contains("error[E0277]"),
                "{carrier} -> {authority}: {d}"
            );
            assert!(!d.contains("E0432") && !d.contains("E0433"));
        }
    }
    for body in [
        "fn f(){let _:quent::manifest::ManifestSource=serde_json::from_str(\"{}\").unwrap();}",
        "fn f(){let _:quent::manifest::ManifestSummary=serde_json::from_str(\"{}\").unwrap();}",
        "fn f(s:quent::manifest::ManifestSource){quent::output_tracing::emit(&s);}",
    ] {
        let r = run(body);
        let d = String::from_utf8_lossy(&r.stderr);
        assert!(!r.status.success() && d.contains("error[E0277]"), "{d}");
    }
    let r = run(
        "fn f(s:quent::manifest::ManifestSource,path:&std::path::Path){use quent::local_details::*;let _=create_new(LocalOptIn::acknowledge_sensitive_local_file(),Surface::LocalApplication,path,s.sensitive_source());}",
    );
    assert!(!r.status.success() && String::from_utf8_lossy(&r.stderr).contains("error[E0308]"));
    for (body, code) in [
        ("fn f(s:String){quent::output_tracing::emit(&s);}", "E0277"),
        (
            "fn f(e:&dyn std::error::Error){quent::output_tracing::emit(&e);}",
            "E0277",
        ),
        (
            "fn f(h:ValidatedOperation){quent::output_tracing::emit(&h);}",
            "E0277",
        ),
        (
            "fn f(p:&quent_core::Plan,path:&std::path::Path){quent::local_details::create_new(true,quent::local_details::Surface::LocalApplication,path,p.sensitive_source());}",
            "E0308",
        ),
        (
            "fn f(p:quent_core::PlanView,path:&std::path::Path){use quent::local_details::*;create_new(LocalOptIn::acknowledge_sensitive_local_file(),Surface::LocalApplication,path,p);}",
            "E0308",
        ),
    ] {
        let r = run(body);
        let d = String::from_utf8_lossy(&r.stderr);
        assert!(
            !r.status.success() && d.contains(&format!("error[{code}]")),
            "{d}"
        );
    }
    for authority in [
        "Operation",
        "DatabaseContext",
        "quent_core::PolicyPreparation",
        "ValidatedOperation",
        "ValidatedAtomicOperation",
        "String",
    ] {
        let r = run(&format!(
            "fn f(v:quent::local_details::CreatedLocalDetail){{let _:{authority}=v.into();}}"
        ));
        let d = String::from_utf8_lossy(&r.stderr);
        assert!(!r.status.success() && d.contains("error[E0277]"), "{d}");
    }
    // Cross every output representation with every authority carrier. E0277 is
    // the missing conversion trait, not a missing type, import or method.
    for output in [
        "quent_core::PlanView",
        "quent_core::DiagnosticView",
        "String",
        "serde_json::Value",
        "quent::ResultView",
    ] {
        for authority in [
            "Operation",
            "quent_core::PolicyPreparation",
            "DatabaseContext",
            "ValidatedOperation",
            "ValidatedAtomicOperation",
        ] {
            let result = run(&format!("fn f(v:{output}){{let _:{authority}=v.into();}}"));
            let diagnostic = String::from_utf8_lossy(&result.stderr);
            assert!(
                !result.status.success() && diagnostic.contains("error[E0277]"),
                "{output} -> {authority}: {diagnostic}"
            );
            assert!(!diagnostic.contains("E0432") && !diagnostic.contains("E0433"));
        }
    }
    for body in [
        "fn f(s:&mut PostgresSession,c:&DatabaseContext,v:quent_core::PlanView){let _=s.execute(c,v);}",
        "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,v:quent_core::PlanView){let _=s.execute_validated(c,o,v);}",
        "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,v:quent_core::DiagnosticView){let _=s.execute_atomic(c,o,ExpectedRows::exactly(1).unwrap(),v);}",
    ] {
        let result = run(body);
        let diagnostic = String::from_utf8_lossy(&result.stderr);
        assert!(
            !result.status.success() && diagnostic.contains("error[E0308]"),
            "{diagnostic}"
        );
    }
    for body in [
        "fn f(){let _:quent_core::PlanView=serde_json::from_str(\"{}\").unwrap();}",
        "fn f(){let _:quent_core::DiagnosticView=serde_json::from_str(\"{}\").unwrap();}",
        "fn f(e:&dyn std::error::Error){let _:quent_core::DiagnosticView=e.into();}",
    ] {
        let result = run(body);
        let diagnostic = String::from_utf8_lossy(&result.stderr);
        assert!(
            !result.status.success() && diagnostic.contains("error[E0277]"),
            "{diagnostic}"
        );
    }
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
    let output = run(
        "fn control(bytes:&[u8],c:&DatabaseContext) {use quent::operation_spec::*;let s=SpecSource::parse(bytes).unwrap();let _=s.public_view().to_json();let _=s.sensitive_source();let m=ModelContract::application_asserted(\"items\",vec![(\"id\".into(),ColumnRole::Ordinary,true)],c).unwrap();let r=s.check_roles(m,c).unwrap();let _=r.matches_snapshot(c);let _=r.public_view().to_json();}",
    );
    assert!(
        output.status.success(),
        "spec API positive control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for source in ["SpecSource", "RoleCheckedSource", "SpecSummary"] {
        for target in [
            "quent_core::Operation",
            "quent_core::DatabaseContext",
            "quent_core::PolicyPreparation",
            "quent::ValidatedOperation",
            "quent::ValidatedAtomicOperation",
            "quent::manifest::live::Observation",
        ] {
            let body =
                format!("fn f(s:quent::operation_spec::{source}) {{let _:{target}=s.into();}}");
            let output = run(&body);
            let diagnostic = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success() && diagnostic.contains("error[E0277]"),
                "{source} to {target}: {diagnostic}"
            );
            assert!(!diagnostic.contains("E0432") && !diagnostic.contains("E0433"));
        }
    }
    for carrier in [
        "SpecSource",
        "RoleCheckedSource",
        "ModelContract",
        "SpecSummary",
        "SpecDiagnostic",
    ] {
        let output = run(&format!(
            "fn f() {{let _:quent::operation_spec::{carrier}=serde_json::from_str(\"{{}}\").unwrap();}}"
        ));
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && diagnostic.contains("error[E0277]"),
            "spec deserialize {carrier}: {diagnostic}"
        );
    }
    for carrier in ["SpecSource", "RoleCheckedSource", "ModelContract"] {
        let output = run(&format!(
            "fn f(s:&quent::operation_spec::{carrier}) {{let _=serde_json::to_string(s);}}"
        ));
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && diagnostic.contains("error[E0277]"),
            "spec serialize {carrier}: {diagnostic}"
        );
    }
    let output = run(
        "async fn control(s:&mut PostgresSession,c:&DatabaseContext,m:&quent::operation_spec::TypedModelContract,bytes:&[u8]) {use quent::operation_spec::*;let r=SpecSource::parse(bytes).unwrap().compile(m,c).unwrap();let _=r.inspect();let _=r.public_view().to_json();let _=s.execute_compiled_read(c,m,r).await;}",
    );
    assert!(
        output.status.success(),
        "typed positive control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for carrier in [
        "CompiledRead",
        "TypedModelContract",
        "TypedColumn",
        "TypedDiagnostic",
    ] {
        let output = run(&format!(
            "fn f() {{let _:quent::operation_spec::{carrier}=serde_json::from_str(\"{{}}\").unwrap();}}"
        ));
        assert!(
            !output.status.success()
                && String::from_utf8_lossy(&output.stderr).contains("error[E0277]")
        );
    }
    for carrier in ["CompiledRead", "TypedModelContract", "TypedColumn"] {
        let output = run(&format!(
            "fn f(s:&quent::operation_spec::{carrier}) {{let _=serde_json::to_string(s);}}"
        ));
        assert!(
            !output.status.success()
                && String::from_utf8_lossy(&output.stderr).contains("error[E0277]")
        );
    }
    for target in [
        "quent_core::Operation",
        "quent_core::DatabaseContext",
        "quent_core::PolicyPreparation",
        "quent::ValidatedOperation",
        "quent::ValidatedAtomicOperation",
    ] {
        let output = run(&format!(
            "fn f(s:quent::operation_spec::CompiledRead) {{let _:{target}=s.into();}}"
        ));
        assert!(
            !output.status.success()
                && String::from_utf8_lossy(&output.stderr).contains("error[E0277]")
        );
    }
    for (body, code) in [
        (
            "fn f(r:quent::operation_spec::CompiledRead){let _=r.clone();}",
            "E0599",
        ),
        (
            "fn take(_:quent::operation_spec::CompiledRead){} fn f(r:quent::operation_spec::CompiledRead){take(r);take(r);}",
            "E0382",
        ),
        (
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,m:&quent::operation_spec::TypedModelContract,r:quent_core::PlanView){let _=s.execute_compiled_read(c,m,r).await;}",
            "E0308",
        ),
        (
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,m:&quent::operation_spec::TypedModelContract,r:quent_core::Operation){let _=s.execute_compiled_read(c,m,r).await;}",
            "E0308",
        ),
        (
            "fn f(r:&mut quent::operation_spec::CompiledRead){r.prepared=todo!();}",
            "E0616",
        ),
    ] {
        let output = run(body);
        assert!(
            !output.status.success() && String::from_utf8_lossy(&output.stderr).contains(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = run(
        "fn control(r:quent::review::Review) { let _=serde_json::to_string(&r).unwrap(); let _=r.to_json(); let _=r.to_pretty(); let _=r.exit_code(); }",
    );
    assert!(
        output.status.success(),
        "review control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for target in [
        "quent::review::Review",
        "quent_core::Operation",
        "quent_core::PolicyPreparation",
        "quent_core::DatabaseContext",
        "quent::ValidatedOperation",
        "quent::ValidatedAtomicOperation",
        "quent::manifest::live::Observation",
        "quent::operation_spec::CompiledRead",
        "quent::operation_spec::TypedModelContract",
    ] {
        let source = if target == "quent::review::Review" {
            format!("fn f() {{let _:{target}=serde_json::from_str(\"{{}}\").unwrap();}}")
        } else {
            format!("fn f(r:quent::review::Review) {{let _:{target}=r.into();}}")
        };
        let output = run(&source);
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0277]"),
            "review authority: {d}"
        );
        assert!(!d.contains("E0432") && !d.contains("E0433"));
    }
    let source_control = format!(
        "mod delete {{ {} }} mod filter {{ {} }} fn control(r:quent::source_review::Review) {{ let _=serde_json::to_string(&r).unwrap(); let _=r.to_json(); let _=r.to_pretty(); let _=r.exit_code(); let _=quent::source_review::review(b\"fn f(){{}}\").unwrap(); }}",
        include_str!("../../../fixtures/source-review-v1/delete.rs"),
        include_str!("../../../fixtures/source-review-v1/filter.rs")
    );
    let output = run(&source_control);
    assert!(
        output.status.success(),
        "source control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for target in [
        "quent::source_review::Review",
        "quent_core::Operation",
        "quent_core::ScopeFacts",
        "quent_core::PolicyPreparation",
        "quent_core::DatabaseContext",
        "quent::ValidatedOperation",
        "quent::ValidatedAtomicOperation",
        "quent::manifest::live::Observation",
        "quent::operation_spec::CompiledRead",
        "quent::operation_spec::TypedModelContract",
    ] {
        let source = if target == "quent::source_review::Review" {
            format!("fn f(){{let _:{target}=serde_json::from_str(\"{{}}\").unwrap();}}")
        } else {
            format!("fn f(r:quent::source_review::Review){{let _:{target}=r.into();}}")
        };
        let output = run(&source);
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0277]"),
            "source authority: {d}"
        );
        assert!(!d.contains("E0432") && !d.contains("E0433"));
    }
    let output = run(
        "fn control(r:quent::review_ci::Report){let _=r.to_json();let _=r.to_pretty();let _=r.to_sarif();let _=r.exit_code();let _=quent::review_ci::Time::unix_seconds(1).unwrap();}",
    );
    assert!(
        output.status.success(),
        "CI control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for target in [
        "quent::review_ci::Report",
        "quent::review_ci::InputSet",
        "quent::review_ci::Baseline",
        "quent::review_ci::Time",
        "quent::review_ci::Diagnostic",
    ] {
        let output = run(&format!(
            "fn f(){{let _:{target}=serde_json::from_str(\"{{}}\").unwrap();}}"
        ));
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0277]") && !d.contains("E0433"),
            "CI restoration: {d}"
        );
    }
    for target in [
        "quent_core::Operation",
        "quent_core::ScopeFacts",
        "quent_core::PolicyPreparation",
        "quent_core::DatabaseContext",
        "quent::ValidatedOperation",
        "quent::ValidatedAtomicOperation",
        "quent::manifest::live::Observation",
        "quent::operation_spec::CompiledRead",
        "quent::operation_spec::TypedModelContract",
    ] {
        let output = run(&format!(
            "fn f(r:quent::review_ci::Report){{let _:{target}=r.into();}}"
        ));
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0277]") && !d.contains("E0433"),
            "CI authority: {d}"
        );
    }
    let output = run(
        "fn control(c:&quent::authorization::AuthorizationContext,o:&Operation){use quent::authorization::*; let candidate=c.candidate(o,AuthorizationScope::MultiRowUpdate,std::time::Duration::from_secs(1)).unwrap(); let f=acquire_authorization(candidate,c); drop(f); } fn reply(r:quent::authorization::AuthorizationRequest<'_>)->quent::authorization::AuthorizationDecision{r.respond(quent::authorization::AdapterResponse::AllowCandidate)}",
    );
    assert!(
        output.status.success(),
        "authorization control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for ty in [
        "AuthorizationCandidate",
        "AuthorizationDecision",
        "AuthorizationAcquisition",
        "AuthorizationRequest<'static>",
    ] {
        for (source, code) in [
            (
                format!("fn f(x:quent::authorization::{ty}){{let _=x.clone();}}"),
                "E0599",
            ),
            (
                format!("fn copy<T:Copy>(){{}} fn f(){{copy::<quent::authorization::{ty}>();}}"),
                "E0277",
            ),
            (
                format!("fn f(x:&quent::authorization::{ty}){{let _=serde_json::to_string(x);}}"),
                "E0277",
            ),
            (
                format!(
                    "fn f(){{let _:quent::authorization::{ty}=serde_json::from_str(\"{{}}\").unwrap();}}"
                ),
                "E0277",
            ),
            (
                format!("fn f(){{let _:quent::authorization::{ty}=\"{{}}\".parse().unwrap();}}"),
                "E0277",
            ),
            (
                format!("fn f(){{let _:quent::authorization::{ty}=true.into();}}"),
                "E0277",
            ),
        ] {
            let output = run(&source);
            let d = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success()
                    && d.contains(&format!("error[{code}]"))
                    && !d.contains("E0433"),
                "authorization negative: {d}"
            );
        }
    }
    for source in [
        "fn f(x:quent::authorization::AuthorizationCandidate){x.expires=std::time::Instant::now();}",
        "fn f(x:quent::authorization::AuthorizationDecision){let _=x.response;}",
    ] {
        let output = run(source);
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0616]"),
            "private binding: {d}"
        );
    }
    for source in [
        "fn f(c:&quent::authorization::AuthorizationContext,x:quent::authorization::AuthorizationCandidate){let f=quent::authorization::acquire_authorization(x,c);drop(f);let _=quent::authorization::acquire_authorization(x,c);}",
        "async fn f(c:&quent::authorization::AuthorizationContext,x:quent::authorization::AuthorizationCandidate){let _=quent::authorization::acquire_authorization(x,c).await;let _=quent::authorization::acquire_authorization(x,c).await;}",
        "fn f(r:quent::authorization::AuthorizationRequest<'_>){use quent::authorization::*;let _=r.respond(AdapterResponse::AllowCandidate);let _=r.respond(AdapterResponse::AllowCandidate);}",
    ] {
        let output = run(source);
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0382]"),
            "linear consumption: {d}"
        );
    }
    for ty in [
        "AuthorizationCandidate",
        "AuthorizationDecision",
        "AuthorizationAcquisition",
    ] {
        let output = run(&format!(
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,x:quent::authorization::{ty}){{let _=s.execute_validated(c,o,x).await;}}"
        ));
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0308]"),
            "not execution: {d}"
        );
    }
    for source in [
        "fn f(){let _=quent::authorization::AuthorizationDecision{invocation:std::sync::Arc::new(()),response:quent::authorization::AdapterResponse::AllowCandidate};}",
        "fn f(p:quent_core::PolicyPreparation,c:quent::authorization::AuthorizationContext){let _=quent::authorization::AuthorizationCandidate{prepared:p,context:c,scope:quent::authorization::AuthorizationScope::MultiRowUpdate,expires:std::time::Instant::now(),runtime:None};}",
        "fn f(c:quent::authorization::AuthorizationCandidate){let _=quent::authorization::AuthorizationAcquisition{_candidate:c};}",
    ] {
        let output = run(source);
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0451]"),
            "constructor private: {d}"
        );
    }
    for ty in [
        "AuthorizationCandidate",
        "AuthorizationDecision",
        "AuthorizationAcquisition",
    ] {
        let output = run(&format!(
            "fn f(v:quent_core::PlanView){{let _:quent::authorization::{ty}=v.into();}}"
        ));
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0277]"),
            "public summary is not authorization: {d}"
        );
    }
    let output = run(
        "fn f(r:quent::authorization::AuthorizationRequest<'_>){let _=r.respond(std::io::Error::other(\"CANARY_nested_error\"));}",
    );
    let d = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && d.contains("error[E0308]"),
        "arbitrary provider error: {d}"
    );
    let output = run(
        "fn f(mut c:quent::authorization::AuthorizationContext,x:quent::authorization::AuthorizationCandidate){let future=quent::authorization::acquire_authorization(x,&c);c=c.clone();drop(future);}",
    );
    let d = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && d.contains("error[E0506]"),
        "current snapshot frozen during await: {d}"
    );
    let output = run(
        "async fn control(s:&mut PostgresSession,a:&quent::authorization::AuthorizationContext,o:&Operation){let r=quent::AuthorizationExecutionRequest::new();let c=s.authorization_candidate(a,&r,o,quent::authorization::AuthorizationScope::MultiRowUpdate,std::time::Duration::from_secs(1)).unwrap();let h=s.acquire_authorized(a,&r,c).await.unwrap();let _=s.execute_authorized(a,&r,o,h).await;let c=s.authorization_candidate(a,&r,o,quent::authorization::AuthorizationScope::MultiRowUpdate,std::time::Duration::from_secs(1)).unwrap();let h=s.acquire_authorized(a,&r,c).await.unwrap();let _=s.execute_authorized_atomic(a,&r,o,ExpectedRows::exactly(1).unwrap(),h).await;}",
    );
    assert!(
        output.status.success(),
        "authorized control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for ty in [
        "AuthorizedOperation",
        "ExecutableAuthorizationCandidate",
        "AuthorizationExecutionRequest",
    ] {
        for (body, diagnostic) in [
            (format!("fn f(x:quent::{ty}){{let _=x.clone();}}"), "E0599"),
            (
                format!("fn f<T:Copy>(){{}} fn g(){{f::<quent::{ty}>();}}"),
                "E0277",
            ),
            (
                format!("fn f(x:&quent::{ty}){{let _=serde_json::to_string(x);}}"),
                "E0277",
            ),
            (
                format!("fn f(){{let _:quent::{ty}=serde_json::from_str(\"{{}}\").unwrap();}}"),
                "E0277",
            ),
            (
                format!("fn f(x:quent_core::PlanView){{let _:quent::{ty}=x.into();}}"),
                "E0277",
            ),
        ] {
            let output = run(&body);
            let d = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success() && d.contains(&format!("error[{diagnostic}]")),
                "authorized boundary: {d}"
            );
        }
    }
    for (body, diagnostic) in [
        (
            "fn f(x:quent::AuthorizedOperation){let _=x.operation;}",
            "E0616",
        ),
        (
            "fn f(x:quent::ExecutableAuthorizationCandidate){let _=x.candidate;}",
            "E0616",
        ),
        (
            "async fn f(s:&mut PostgresSession,a:&quent::authorization::AuthorizationContext,r:&quent::AuthorizationExecutionRequest,o:&Operation,h:quent::AuthorizedOperation){let f=s.execute_authorized(a,r,o,h);drop(f);let _=s.execute_authorized(a,r,o,h);}",
            "E0382",
        ),
        (
            "async fn f(s:&mut PostgresSession,a:&quent::authorization::AuthorizationContext,r:&quent::AuthorizationExecutionRequest,o:&Operation,h:quent::AuthorizedOperation){s.execute_authorized(a,r,o,h).await;let _=s.execute_authorized(a,r,o,h);}",
            "E0382",
        ),
        (
            "async fn f(s:&mut PostgresSession,a:&quent::authorization::AuthorizationContext,r:&quent::AuthorizationExecutionRequest,o:&Operation,h:quent::authorization::AuthorizationAcquisition){s.execute_authorized(a,r,o,h).await;}",
            "E0308",
        ),
        (
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:quent::AuthorizedOperation){s.execute_validated(c,o,h).await;}",
            "E0308",
        ),
        (
            "async fn f(s:&mut PostgresSession,a:&quent::authorization::AuthorizationContext,r:&quent::AuthorizationExecutionRequest,h:quent::authorization::AuthorizationCandidate){s.acquire_authorized(a,r,h).await;}",
            "E0308",
        ),
        (
            "fn f(x:quent::ValidatedOperation){let _=quent::AuthorizedOperation{operation:x};}",
            "E0639",
        ),
    ] {
        let output = run(body);
        let d = String::from_utf8_lossy(&output.stderr);
        // Field-incomplete literals may report E0063 before privacy checking.
        let matches = d.contains(&format!("error[{diagnostic}]"))
            || (diagnostic == "E0639"
                && (d.contains("error[E0063]") || d.contains("due to private fields")));
        assert!(
            !output.status.success() && matches,
            "authorized ownership/constructor: {d}"
        );
    }
    let output = run(
        "async fn control(b:&mut BorrowedPostgresTransaction<'_>,a:&quent::authorization::AuthorizationContext,r:&quent::AuthorizationExecutionRequest,o:&Operation){let c=b.authorization_candidate(a,r,o,quent::authorization::AuthorizationScope::MultiRowUpdate,std::time::Duration::from_secs(1)).unwrap();let h=b.acquire_authorized(a,r,c).await.unwrap();let _=b.execute_authorized(a,r,o,h).await;}",
    );
    assert!(
        output.status.success(),
        "borrowed authorized control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for (body, diagnostic) in [
        (
            "fn f(h:quent::AuthorizedOperation){let _=h.claim(true);}",
            "E0599",
        ),
        (
            "fn f(){let _:quent::AuthorizedOperation=\"{}\".parse().unwrap();}",
            "E0277",
        ),
        (
            "fn f(){let _:quent::AuthorizedOperation=true.into();}",
            "E0277",
        ),
        (
            "fn f(x:quent::AuthorizationExecutionRequest){let _=quent::AuthorizationExecutionRequest{identity:std::sync::Arc::new(()),requires_live_freshness:false};}",
            "E0451",
        ),
        (
            "async fn f(s:&mut PostgresSession,a:&quent::authorization::AuthorizationContext,r:&quent::AuthorizationExecutionRequest,h:quent::ExecutableAuthorizationCandidate){let future=s.acquire_authorized(a,r,h);drop(future);let _=s.acquire_authorized(a,r,h);}",
            "E0382",
        ),
        (
            "async fn f(s:&mut PostgresSession,a:&quent::authorization::AuthorizationContext,r:&quent::AuthorizationExecutionRequest,o:&Operation,h:quent::AuthorizedOperation){let future=s.execute_authorized_atomic(a,r,o,ExpectedRows::exactly(1).unwrap(),h);drop(future);let _=s.execute_authorized_atomic(a,r,o,ExpectedRows::exactly(1).unwrap(),h);}",
            "E0382",
        ),
    ] {
        let output = run(body);
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains(&format!("error[{diagnostic}]")),
            "authorized restoration/cancel: {d}"
        );
    }
    let output = run(
        "async fn control(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,n:&quent::CommandNamespace,l:&quent::CommandLedger){let id=quent::CommandId::new([1;16]);let h=s.validate_command(c,o,n,id.clone(),l,std::time::Duration::from_secs(1)).unwrap();let _=s.execute_command(c,o,h).await;let h=s.validate_command_expected(c,o,n,id,l,quent::ExpectedRows::exactly(1).unwrap(),std::time::Duration::from_secs(1)).unwrap();let _=s.lookup_command(c,o,h).await;}",
    );
    assert!(
        output.status.success(),
        "command control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = run(
        "fn control(p:&quent_core::PolicyPreparation,r:&quent::CommandResult){let c=p.sensitive_source().canonical_command_input(p.sensitive_source().final_source().sensitive_table()).unwrap();let _=c.sensitive_bytes();let _=r.sensitive_result().affected_rows();let _=r.kind();let _=r.public_view().to_json();let _=quent::CommandNamespace::new(vec![0,255]);let id=quent::CommandId::new([0;16]);let _=id.clone();}",
    );
    assert!(
        output.status.success(),
        "command sensitive/identity control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for carrier in [
        "quent::ValidatedCommand",
        "quent::CommandResult",
        "quent::CommandId",
        "quent::CommandNamespace",
        "quent::CommandLedger",
        "quent_core::CommandCanonical",
    ] {
        for body in [
            format!("fn f(v:&{carrier}){{serde_json::to_string(v).unwrap();}}"),
            format!("fn f(){{let _:{carrier}=serde_json::from_str(\"{{}}\").unwrap();}}"),
        ] {
            let output = run(&body);
            let d = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success() && d.contains("error[E0277]"),
                "command serde: {d}"
            );
        }
    }
    for (body, diagnostic) in [
        ("fn f(h:quent::ValidatedCommand){let _=h.clone();}", "E0599"),
        (
            "fn copy<T:Copy>(){} fn f(){copy::<quent::ValidatedCommand>();}",
            "E0277",
        ),
        (
            "fn f(h:&quent::ValidatedCommand){let _=&h.operation;}",
            "E0616",
        ),
        ("fn f(h:&quent::ValidatedCommand){let _=&h.key;}", "E0616"),
        (
            "fn f(h:&mut quent::ValidatedCommand){h.expected=quent::ExpectedRows::exactly(0).unwrap();}",
            "E0616",
        ),
        (
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:quent::ValidatedCommand){let _=s.execute_command(c,o,h).await;let _=s.execute_command(c,o,h).await;}",
            "E0382",
        ),
        (
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:quent::ValidatedCommand){let future=s.execute_command(c,o,h);drop(future);let _=s.execute_command(c,o,h);}",
            "E0382",
        ),
        (
            "fn f(v:quent_core::PlanView){let _:quent::ValidatedCommand=v.into();}",
            "E0277",
        ),
        (
            "fn f(v:Vec<u8>){let _:quent::ValidatedCommand=v.into();}",
            "E0277",
        ),
        (
            "fn f(v:quent::CommandResult){let _:quent::ValidatedCommand=v.into();}",
            "E0277",
        ),
        (
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,o:&Operation,h:quent::ValidatedCommand){let future=s.lookup_command(c,o,h);drop(future);let _=s.lookup_command(c,o,h);}",
            "E0382",
        ),
    ] {
        let output = run(body);
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains(&format!("error[{diagnostic}]")),
            "command ownership/restoration: {d}"
        );
    }
    let output = run(
        "async fn control(s:&mut PostgresSession,c:&DatabaseContext,b:&quent_core::CommandBundle,n:&quent::CommandNamespace,l:&quent::CommandLedger){let h=s.validate_command_bundle(c,b,n,quent::CommandId::new([1;16]),l,std::time::Duration::from_secs(1)).unwrap();let _=s.execute_command_bundle(c,b,h).await;let h=s.validate_command_bundle(c,b,n,quent::CommandId::new([1;16]),l,std::time::Duration::from_secs(1)).unwrap();let _=s.lookup_command_bundle(c,b,h).await;}",
    );
    assert!(
        output.status.success(),
        "bundle control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = run(
        "fn control(b:quent_core::CommandBundle,p:quent_core::CommandBundlePreparation,c:&DatabaseContext,r:&quent::CommandResult){let _=b.sensitive_business();let _=b.same_bounded_input(&b);let _=p.sensitive_records();let _=r.sensitive_companion_results();let _=c.prepare().prepare_command_bundle(&b,b\"ns\",&[1;16]);}",
    );
    assert!(
        output.status.success(),
        "bundle pure control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for ty in [
        "quent_core::CommandBundle",
        "quent_core::CommandCompanion",
        "quent_core::CommandBundlePreparation",
        "quent::ValidatedCommandBundle",
    ] {
        for bound in ["Clone", "Copy"] {
            let output = run(&format!(
                "fn require<T:{bound}>(){{}} fn f(){{require::<{ty}>();}}"
            ));
            let d = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success() && d.contains("error[E0277]"),
                "bundle ownership {d}"
            );
        }
        let output = run(&format!("fn f(v:&{ty}){{let _=serde_json::to_string(v);}}"));
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0277]"),
            "bundle serialization {d}"
        );
        let output = run(&format!(
            "fn f(){{let _:{ty}=serde_json::from_str(\"{{}}\").unwrap();}}"
        ));
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains("error[E0277]"),
            "bundle restoration {d}"
        );
    }
    for (body, diagnostic) in [
        (
            "async fn f(s:&mut PostgresSession,c:&DatabaseContext,b:&quent_core::CommandBundle,h:quent::ValidatedCommandBundle){let _=s.execute_command_bundle(c,b,h).await;let _=s.execute_command_bundle(c,b,h).await;}",
            "E0382",
        ),
        (
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,b:&quent_core::CommandBundle,h:quent::ValidatedCommandBundle){let f=s.execute_command_bundle(c,b,h);drop(f);let _=s.execute_command_bundle(c,b,h);}",
            "E0382",
        ),
        (
            "fn f(s:&mut PostgresSession,c:&DatabaseContext,b:&quent_core::CommandBundle,h:quent::ValidatedCommandBundle){let f=s.lookup_command_bundle(c,b,h);drop(f);let _=s.lookup_command_bundle(c,b,h);}",
            "E0382",
        ),
        (
            "fn f(h:&quent::ValidatedCommandBundle){let _=&h.inner;}",
            "E0616",
        ),
        (
            "fn f(b:&quent_core::CommandBundle){let _=&b.audit;}",
            "E0616",
        ),
        (
            "fn f(h:quent::ValidatedCommandBundle)->ValidatedOperation{h}",
            "E0308",
        ),
        (
            "fn f(h:quent_core::CommandBundlePreparation)->ValidatedOperation{h}",
            "E0308",
        ),
        (
            "fn f(h:quent::CommandResult)->quent::ValidatedCommandBundle{h}",
            "E0308",
        ),
    ] {
        let output = run(body);
        let d = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && d.contains(&format!("error[{diagnostic}]")),
            "bundle boundary {d}"
        );
    }
    assert_eq!(
        controls.get(),
        18,
        "compiling positive controls (14 retained)"
    );
    assert_eq!(
        runs.get() - controls.get(),
        319,
        "diagnostic-checked negatives (272 retained)"
    );
}
