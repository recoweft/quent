//! Real fresh production output, workspace pins/toolchain, diagnostic-checked rustc.
#![cfg(feature = "codegen")]
use std::{fs, process::Command};
#[test]
fn generated_consumer_positive_controls_and_checked_type_refusals() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!("quent-codegen-consumer-{}", std::process::id()));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fs::create_dir_all(dir.join("src")).unwrap();
    let _cleanup = Cleanup(dir.clone());
    for d in ["postgres", "mysql"] {
        let root = repo.join("../../fixtures/codegen-v1");
        let artifact = quent::codegen::generate(
            &fs::read(root.join(format!("{d}-source.json"))).unwrap(),
            &fs::read(root.join(format!("{d}-config.json"))).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join(format!("src/{d}.rs")),
            artifact.sensitive_rust_source(),
        )
        .unwrap();
    }
    let root = repo.join("../../fixtures/codegen-v1");
    let mut source: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("postgres-source.json")).unwrap()).unwrap();
    let mut config: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("postgres-config.json")).unwrap()).unwrap();
    for table in source["schema"]["tables"].as_array_mut().unwrap() {
        table["keys"][0]["columns"] = serde_json::json!(["id"]);
    }
    source["policy"]["rules"] = serde_json::json!([]);
    for model in config["models"].as_array_mut().unwrap() {
        for column in model["columns"].as_array_mut().unwrap() {
            column["role"] = serde_json::json!("ordinary");
        }
    }
    let plain = quent::codegen::generate(
        &serde_json::to_vec(&source).unwrap(),
        &serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    fs::write(dir.join("src/plain.rs"), plain.sensitive_rust_source()).unwrap();
    source["schema"]["tables"]
        .as_array_mut()
        .unwrap()
        .truncate(1);
    config["models"].as_array_mut().unwrap().truncate(1);
    let fields: Vec<_> = (0..16).map(|i| format!("f{i}")).collect();
    source["schema"]["tables"][0]["columns"]=serde_json::json!(fields.iter().map(|n|serde_json::json!({"name":n,"native":"i64","nullability":"nonnull","default":{"state":"none"}})).collect::<Vec<_>>());
    source["schema"]["tables"][0]["keys"][0]["columns"] = serde_json::json!(fields);
    config["models"][0]["columns"] = serde_json::json!(
        fields
            .iter()
            .map(|n| serde_json::json!({"name":n,"role":"ordinary","read":true}))
            .collect::<Vec<_>>()
    );
    config["models"][0]["projections"] = serde_json::json!([{"name":"detail","fields":fields}]);
    let wide = quent::codegen::generate(
        &serde_json::to_vec(&source).unwrap(),
        &serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    fs::write(dir.join("src/wide.rs"), wide.sensitive_rust_source()).unwrap();

    fs::write(
        dir.join("Cargo.toml"),
        format!(
            r#"[package]
name="quent-generated-consumer"
version="0.0.0"
edition="2024"
rust-version="1.94"
publish=false
[workspace]
[features]
default=[]
postgres=["quent/postgres"]
mysql=["quent/mysql"]
runtime-tokio=["quent/runtime-tokio"]
[dependencies]
quent={{path={:?},features=["codegen"]}}
quent-core={{path={:?}}}
serde_json="=1.0.154"
"#,
            repo,
            repo.join("../quent-core")
        ),
    )
    .unwrap();
    // Seed exact workspace resolutions; only this standalone root package is new.
    let workspace_lock = fs::read_to_string(repo.join("../../Cargo.lock")).unwrap();
    fs::write(dir.join("Cargo.lock"), &workspace_lock).unwrap();
    let lock_ready = std::cell::Cell::new(false);
    let prelude = "#![allow(dead_code)]\nmod pg {include!(\"postgres.rs\");} mod my {include!(\"mysql.rs\");} mod plain {include!(\"plain.rs\");} mod wide {include!(\"wide.rs\");} use pg::generated_items as a;use pg::other_items as b;\n";
    let run = |body: &str, features: &str| {
        fs::write(dir.join("src/lib.rs"), format!("{prelude}{body}")).unwrap();
        let mut command = Command::new(env!("CARGO"));
        command.arg("check");
        if lock_ready.replace(true) {
            command.arg("--locked");
        }
        command
            .args(["--offline", "--quiet", "--manifest-path"])
            .arg(dir.join("Cargo.toml"))
            .args(["--features", features])
            .env("CARGO_TARGET_DIR", repo.join("../../target/owned-api"))
            .output()
            .unwrap()
    };
    let control = "fn control(c:&quent_core::DatabaseContext,m:&quent::operation_spec::TypedModelContract) {let repo=a::Repository::bind(c,m).unwrap();let k=a::Key::new(1i16,2i64);let _=k.sensitive_components();let p=k.predicate().unwrap().and(a::Model::id().eq(2i64).unwrap()).unwrap();let r:a::Read<a::CompactRow>=repo.select(a::Model::compact(),Some(p),vec![a::Model::id().desc()],10u16).unwrap();let _=r.inspect();let _=r.public_view();let _=a::Model::small().eq(Some(1i16));let _=a::Model::txt().eq(Some(String::new()));let _=my::generated_items::Model::u_big().eq(Some(u64::MAX));let _=plain::generated_items::Key::new(2i64);}";
    for features in [
        "",
        "postgres,runtime-tokio",
        "mysql,runtime-tokio",
        "postgres,mysql,runtime-tokio",
    ] {
        let r = run(control, features);
        assert!(
            r.status.success(),
            "generated positive ({features}): {}",
            String::from_utf8_lossy(&r.stderr)
        );
    }
    let mut negatives = 0;
    let mut check = |body: &str, code: &str, needle: &str| {
        let r = run(body, "postgres,mysql,runtime-tokio");
        let stderr = String::from_utf8_lossy(&r.stderr);
        assert!(!r.status.success(), "negative compiled: {body}");
        assert!(
            stderr.contains(code) && stderr.contains(needle),
            "wrong diagnostic for {body}: {stderr}"
        );
        assert!(
            !stderr.contains("E0432") && !stderr.contains("E0463"),
            "missing import/dependency is not evidence: {stderr}"
        );
        negatives += 1;
    };
    for (body, code, needle) in [
        (
            "fn f(v:plain::other_items::Key){let _:plain::generated_items::Key=v;}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(v:b::Key){let _:a::Key=v;}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(v:b::Column<i64>){let _:a::Column<i64>=v;}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(r:&a::Repository<'_>){r.select(a::Model::compact(),Some(b::Model::id().eq(1).unwrap()),vec![],1);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(r:&a::Repository<'_>){r.select(a::Model::compact(),None,vec![b::Model::id().asc()],1);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(r:&a::Repository<'_>){r.select(b::Model::compact(),None,vec![],1);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(v:b::CompactRow){let _:a::CompactRow=v;}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(v:b::Read<b::CompactRow>){let _:a::Read<a::CompactRow>=v;}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(v:b::Rows<b::CompactRow>){let _:a::Rows<a::CompactRow>=v;}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){a::Model::id().eq(None);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){a::Model::small().eq(1i16);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){a::Model::small().eq(Some(1i64));}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){my::generated_items::Model::u_big().eq(Some(-1i64));}",
            "E0308",
            "mismatched types",
        ),
        ("fn f(){a::Key::new(1i16);}", "E0061", "argument"),
        (
            "fn f(){a::Key::new(1i64,2i16);}",
            "E0308",
            "arguments to this function are incorrect",
        ),
        ("fn f(){a::Model::tenant().eq(11i64);}", "E0599", "tenant"),
        (
            "fn f(v:a::Read<a::CompactRow>){let _=v.compiled;}",
            "E0616",
            "private",
        ),
        (
            "fn f(v:a::Read<a::CompactRow>){let _:quent::operation_spec::CompiledRead=v;}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(v:a::Key){let _:quent_core::DatabaseContext=v.into();}",
            "E0277",
            "From",
        ),
        (
            "fn f(v:&a::Key){serde_json::to_string(v).unwrap();}",
            "E0277",
            "Serialize",
        ),
        (
            "fn f(v:&a::CompactRow){serde_json::to_string(v).unwrap();}",
            "E0277",
            "Serialize",
        ),
        ("fn f(v:&a::Key){format!(\"{}\",v);}", "E0277", "Display"),
        (
            "fn f(){let _:a::Key=serde_json::from_str(\"[1,2]\").unwrap();}",
            "E0277",
            "Deserialize",
        ),
        (
            "fn f(){let _:quent_core::ScopeFacts=serde_json::from_str(\"{}\").unwrap();}",
            "E0277",
            "Deserialize",
        ),
        (
            "fn f(v:a::CompactRow){let _=v.optional;}",
            "E0616",
            "private",
        ),
        ("fn f(v:a::Key){let _=v.0;}", "E0616", "private"),
        (
            "fn f(){let _=a::Predicate{wire:serde_json::json!({}),nodes:1,depth:0};}",
            "E0451",
            "private",
        ),
        (
            "fn f(c:&quent_core::DatabaseContext,m:&quent::operation_spec::TypedModelContract){let _=a::Repository{current:c,mapping:m};}",
            "E0451",
            "private",
        ),
        (
            "fn f(v:a::Read<a::CompactRow>){let _=v.clone();}",
            "E0599",
            "clone",
        ),
        (
            "fn f(v:a::CompactRow){let _=v.sensitive_txt();}",
            "E0599",
            "sensitive_txt",
        ),
        (
            "fn f(v:a::Key){let _:b::Predicate=v.predicate().unwrap();}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(v:&a::Model){serde_json::to_string(v).unwrap();}",
            "E0277",
            "Serialize",
        ),
        (
            "fn f(v:quent::codegen::CodeArtifact){let _=v.source;}",
            "E0616",
            "private",
        ),
        (
            "fn f(){let _:quent::codegen::Summary=serde_json::from_str(\"{}\").unwrap();}",
            "E0277",
            "Deserialize",
        ),
        (
            "fn f(v:quent::codegen::CodeArtifact){let _:quent_core::DatabaseContext=v.into();}",
            "E0277",
            "From",
        ),
        (
            "fn f(v:a::Projection<a::CompactRow>){let _:quent::ValidatedOperation=v.into();}",
            "E0277",
            "From",
        ),
    ] {
        check(body, code, needle);
    }
    let execution = "async fn execute(s:&mut quent::PostgresSession,c:&quent_core::DatabaseContext,m:&quent::operation_spec::TypedModelContract,r:a::Read<a::CompactRow>){let rows=r.execute_postgres(s,c,m).await.unwrap();let row:&a::CompactRow=&rows.sensitive_rows()[0];let _: &Option<i64>=row.sensitive_optional();let _:&i64=row.sensitive_id();}";
    let r = run(execution, "postgres,runtime-tokio");
    assert!(
        r.status.success(),
        "execute positive: {}",
        String::from_utf8_lossy(&r.stderr)
    );
    check(
        "async fn f(s:&mut quent::PostgresSession,c:&quent_core::DatabaseContext,m:&quent::operation_spec::TypedModelContract,r:a::Read<a::CompactRow>){r.execute_postgres(s,c,m).await; r.inspect();}",
        "E0382",
        "moved",
    );
    assert_eq!(negatives, 37);
    fs::write(dir.join("src/lib.rs"),format!("{prelude}#[test] fn maximum_complete_key() {{ let key=wide::generated_items::Key::new(0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15); assert!(key.predicate().is_ok()); }}")).unwrap();
    let control = Command::new(env!("CARGO"))
        .args([
            "test",
            "--lib",
            "--offline",
            "--locked",
            "--quiet",
            "--manifest-path",
        ])
        .arg(dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", repo.join("../../target/owned-api"))
        .output()
        .unwrap();
    assert!(
        control.status.success(),
        "maximum-key positive run: {}",
        String::from_utf8_lossy(&control.stderr)
    );
    let registry_packages = |lock: &str| -> std::collections::BTreeSet<String> {
        lock.split("[[package]]")
            .filter(|p| p.lines().any(|l| l.starts_with("source = ")))
            .map(|p| {
                p.lines()
                    .filter(|l| {
                        l.starts_with("name = ")
                            || l.starts_with("version = ")
                            || l.starts_with("source = ")
                            || l.starts_with("checksum = ")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect()
    };
    let expected = registry_packages(&workspace_lock);
    let actual = registry_packages(&fs::read_to_string(dir.join("Cargo.lock")).unwrap());
    assert!(
        actual.is_subset(&expected),
        "consumer dependencies drifted from workspace lock"
    );
    println!(
        "fresh generated consumer: 37 diagnostic-checked negatives, 6 positive controls; four finite feature combinations"
    );
}
