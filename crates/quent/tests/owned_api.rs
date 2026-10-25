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
quent = {{ path = {:?}, features = ["postgres", "runtime-tokio", "local-details", "output-tracing", "manifest", "operation-spec", "review"] }}
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
    assert_eq!(
        controls.get(),
        9,
        "compiling positive controls (8 retained)"
    );
    assert_eq!(
        runs.get() - controls.get(),
        179,
        "diagnostic-checked negatives (170 retained)"
    );
}
