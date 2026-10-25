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
    for d in ["postgres", "mysql"] {
        let root = repo.join("../../fixtures/codegen-v2");
        let artifact = quent::codegen::generate(
            &fs::read(root.join(format!("{d}-source.json"))).unwrap(),
            &fs::read(root.join(format!("{d}-config.json"))).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join(format!("src/v2_{d}.rs")),
            artifact.sensitive_rust_source(),
        )
        .unwrap();
    }
    let root2 = repo.join("../../fixtures/codegen-v2");
    let mut patch_source: serde_json::Value =
        serde_json::from_slice(&fs::read(root2.join("mysql-source.json")).unwrap()).unwrap();
    let mut patch_config: serde_json::Value =
        serde_json::from_slice(&fs::read(root2.join("mysql-config.json")).unwrap()).unwrap();
    patch_source["policy"]["rules"] = serde_json::json!([]);
    for model in patch_config["models"].as_array_mut().unwrap() {
        for column in model["columns"].as_array_mut().unwrap() {
            column["role"] = serde_json::json!("ordinary");
        }
    }
    for table in patch_source["schema"]["tables"].as_array_mut().unwrap() {
        for column in table["columns"].as_array_mut().unwrap() {
            column["nullability"] = serde_json::json!("nonnull");
        }
    }
    let plain_patch = quent::codegen::generate(
        &serde_json::to_vec(&patch_source).unwrap(),
        &serde_json::to_vec(&patch_config).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("src/patch_plain.rs"),
        plain_patch.sensitive_rust_source(),
    )
    .unwrap();
    let mut reduced = patch_config.clone();
    for model in reduced["models"].as_array_mut().unwrap() {
        for col in model["columns"].as_array_mut().unwrap() {
            if col["name"] == "txt" {
                col["read"] = serde_json::json!(false);
            }
        }
        for projection in model["projections"].as_array_mut().unwrap() {
            projection["fields"]
                .as_array_mut()
                .unwrap()
                .retain(|v| v != "txt");
        }
    }
    let reduced_patch = quent::codegen::generate(
        &serde_json::to_vec(&patch_source).unwrap(),
        &serde_json::to_vec(&reduced).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("src/patch_reduced.rs"),
        reduced_patch.sensitive_rust_source(),
    )
    .unwrap();
    patch_source["schema"]["tables"]
        .as_array_mut()
        .unwrap()
        .truncate(1);
    patch_config["models"].as_array_mut().unwrap().truncate(1);
    let mut columns = vec![
        serde_json::json!({"name":"id","native":"i64","nullability":"nonnull","default":{"state":"none"}}),
    ];
    columns.extend((0..18).map(|i|serde_json::json!({"name":format!("text{i}"),"native":"text","nullability":"nonnull","default":{"state":"none"}})));
    patch_source["schema"]["tables"][0]["columns"] = serde_json::json!(columns);
    patch_source["schema"]["tables"][0]["keys"] =
        serde_json::json!([{"kind":"primary","coverage":"complete","columns":["id"]}]);
    patch_config["models"][0]["columns"] = serde_json::json!(
        columns
            .iter()
            .map(|c| serde_json::json!({"name":c["name"],"role":"ordinary","read":true}))
            .collect::<Vec<_>>()
    );
    patch_config["models"][0]["projections"] =
        serde_json::json!([{"name":"detail","fields":["id"]}]);
    let budget_patch = quent::codegen::generate(
        &serde_json::to_vec(&patch_source).unwrap(),
        &serde_json::to_vec(&patch_config).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("src/patch_budget.rs"),
        budget_patch.sensitive_rust_source(),
    )
    .unwrap();
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
    let prelude = "#![allow(dead_code)]\nmod reduced {include!(\"patch_reduced.rs\");} mod patchplain {include!(\"patch_plain.rs\");} mod patchbudget {include!(\"patch_budget.rs\");} mod v2 {include!(\"v2_postgres.rs\");} mod v2my {include!(\"v2_mysql.rs\");} mod pg {include!(\"postgres.rs\");} mod my {include!(\"mysql.rs\");} mod plain {include!(\"plain.rs\");} mod wide {include!(\"wide.rs\");} use pg::generated_items as a;use pg::other_items as b;\n";
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
    let control = "fn nonnull() {let _=patchplain::generated_items::Patch::new().set_big(i64::MAX).unwrap().set_flag(true).unwrap().set_txt(String::new()).unwrap();} fn patch(r:&v2::generated_items::Repository<'_>) {use v2::generated_items::*;let p=Patch::new().set_big(i64::MIN).unwrap().set_null_flag().unwrap().set_txt(String::new()).unwrap();let c=r.compile_patch(Key::new(i16::MIN,i64::MAX),p).unwrap();let _=c.inspect();let _=c.public_view();let _=v2my::generated_items::Patch::new().set_u_big(u64::MAX).unwrap();} fn control(c:&quent_core::DatabaseContext,m:&quent::operation_spec::TypedModelContract) {let repo=a::Repository::bind(c,m).unwrap();let k=a::Key::new(1i16,2i64);let _=k.sensitive_components();let p=k.predicate().unwrap().and(a::Model::id().eq(2i64).unwrap()).unwrap();let r:a::Read<a::CompactRow>=repo.select(a::Model::compact(),Some(p),vec![a::Model::id().desc()],10u16).unwrap();let _=r.inspect();let _=r.public_view();let _=a::Model::small().eq(Some(1i16));let _=a::Model::txt().eq(Some(String::new()));let _=my::generated_items::Model::u_big().eq(Some(u64::MAX));let _=plain::generated_items::Key::new(2i64);}";
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
    for (body, code, needle) in [
        (
            "fn f(v:v2::other_items::Patch){let _:v2::generated_items::Patch=v;}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(r:&v2::generated_items::Repository<'_>,p:v2::generated_items::Patch){r.compile_patch(v2::other_items::Key::new(1,2),p);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){v2::generated_items::Patch::new().set_tenant(11);}",
            "E0599",
            "set_tenant",
        ),
        (
            "fn f(){v2::generated_items::Patch::new().set_small(1i16);}",
            "E0599",
            "set_small",
        ),
        (
            "fn f(){v2::generated_items::Patch::new().set_normal(1i32);}",
            "E0599",
            "set_normal",
        ),
        (
            "fn f(){v2::generated_items::Patch::new().set_id(1i64);}",
            "E0599",
            "set_id",
        ),
        (
            "fn f(){v2::generated_items::Patch::new().set_big(None);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){v2::generated_items::Patch::new().set_big(Some(1i64));}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){v2my::generated_items::Patch::new().set_u_big(-1i64);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(p:v2::generated_items::Patch){let _=p.big;}",
            "E0616",
            "private",
        ),
        (
            "fn f(p:v2::generated_items::CompiledPatch){let _=p.prepared;}",
            "E0616",
            "private",
        ),
        (
            "fn f(p:&v2::generated_items::Patch){serde_json::to_string(p).unwrap();}",
            "E0277",
            "Serialize",
        ),
        (
            "fn f(){let _:v2::generated_items::Patch=serde_json::from_str(\"{}\").unwrap();}",
            "E0277",
            "Deserialize",
        ),
        (
            "fn f(p:v2::generated_items::Patch){let _:quent_core::Operation=p.into();}",
            "E0277",
            "From",
        ),
        (
            "fn f(p:v2::generated_items::CompiledPatch){let _:quent::ValidatedOperation=p.into();}",
            "E0277",
            "From",
        ),
        (
            "fn f(p:&v2::generated_items::Patch){format!(\"{}\",p);}",
            "E0277",
            "Display",
        ),
        (
            "fn f(){let _:quent::codegen::Regeneration=serde_json::from_str(\"{}\").unwrap();}",
            "E0277",
            "Deserialize",
        ),
    ] {
        check(body, code, needle);
    }
    for (body, code, needle) in [
        (
            "fn f(){patchplain::generated_items::Patch::new().set_null_big();}",
            "E0599",
            "set_null_big",
        ),
        (
            "fn f(){patchplain::generated_items::Patch::new().set_null_flag();}",
            "E0599",
            "set_null_flag",
        ),
        (
            "fn f(){patchplain::generated_items::Patch::new().set_null_txt();}",
            "E0599",
            "set_null_txt",
        ),
        (
            "fn f(){patchplain::generated_items::Patch::new().set_big(None);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(p:Box<dyn std::any::Any>,r:&v2::generated_items::Repository<'_>){r.compile_patch(v2::generated_items::Key::new(1,2),p);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(p:v2::generated_items::CompiledPatch){p.execute();}",
            "E0599",
            "execute",
        ),
        (
            "fn f(){let _:quent::codegen::PatchSummary=serde_json::from_str(\"{}\").unwrap();}",
            "E0277",
            "Deserialize",
        ),
    ] {
        check(body, code, needle);
    }
    for column in [
        "small", "normal", "optional", "tiny", "u_tiny", "u_small", "u_normal", "u_big",
    ] {
        let body =
            format!("fn f(){{patchplain::generated_items::Patch::new().set_null_{column}();}}");
        check(&body, "E0599", &format!("set_null_{column}"));
    }
    for (body, code, needle) in [
        (
            "fn f(){patchplain::generated_items::Patch::new().set_small(1i32);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){patchplain::generated_items::Patch::new().set_flag(0u8);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){patchplain::generated_items::Patch::new().set_txt(vec![1u8]);}",
            "E0308",
            "mismatched types",
        ),
        (
            "fn f(){let _=v2::generated_items::Patch{..v2::generated_items::Patch::new()};}",
            "E0451",
            "private",
        ),
        (
            "fn f(){let _:v2::generated_items::CompiledPatch=serde_json::from_str(\"{}\").unwrap();}",
            "E0277",
            "Deserialize",
        ),
        (
            "fn f(p:&v2::generated_items::CompiledPatch){serde_json::to_string(p).unwrap();}",
            "E0277",
            "Serialize",
        ),
    ] {
        check(body, code, needle);
    }
    check(
        "fn f(){reduced::generated_items::Patch::new().set_txt(String::new());}",
        "E0599",
        "set_txt",
    );
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
    assert_eq!(negatives, 76);
    fs::write(dir.join("src/lib.rs"),format!("{prelude}#[test] fn maximum_complete_key() {{ let key=wide::generated_items::Key::new(0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15); assert!(key.predicate().is_ok()); }}")).unwrap();
    let mut runtime_source = fs::read_to_string(dir.join("src/lib.rs")).unwrap();
    runtime_source.push_str(
        "\n#[test] fn payload_budget() {let p=patchbudget::generated_items::Patch::new();",
    );
    for i in 0..16 {
        runtime_source.push_str(&format!(
            "let p=p.set_text{i}(\"x\".repeat(4096)).unwrap();"
        ));
    }
    runtime_source.push_str(
        "assert_eq!(p.set_text16(\"x\".into()).unwrap_err(),quent::codegen::PatchError::Budget);}",
    );
    runtime_source.push_str(r#"
fn budget_context()->(quent_core::DatabaseContext,quent::operation_spec::TypedModelContract) {
    use quent_core::*;use quent::operation_spec::*;
    let id=|s:&str|Ident::new(s).unwrap();
    let binding=DatabaseBinding::application_supplied(DatabaseIdentity::new("dedicated_test").unwrap(),TargetAddress::new("dedicated_loopback").unwrap(),id("quent_dispatch_test"),Dialect::Mysql);
    let g=Generation::new(1).unwrap();let table=Table::new(vec![id("quent_dispatch_test"),id("generated_items")],None).unwrap();
    let mut columns=vec![table.column(id("id"),SqlType::I64,false)];
    columns.extend((0..18).map(|i|table.column(id(&format!("text{i}")),SqlType::Text,false)));
    let facts=ScopeFacts::application_asserted(table.clone(),Dialect::Mysql,columns.clone(),vec![Key::Primary(vec![columns[0].clone()])],TableContext::OrdinaryBaseTableWithoutEffects).unwrap();
    let current=DatabaseContext::new(Settings::new(binding.clone(),SettingsIdentity::new("settings").unwrap(),g,PolicySet::new(binding.clone(),PolicyIdentity::new("policy").unwrap(),g,vec![]).unwrap()).unwrap(),ApplicationContext::application_supplied(binding.clone(),Some(SchemaSnapshot::application_supplied(binding,SchemaIdentity::new("schema").unwrap(),g,vec![facts]).unwrap()),Some(SubjectIdentity::new("subject").unwrap()),None).unwrap()).unwrap();
    let mapping=TypedModelContract::application_asserted("generated_items",table,columns.iter().map(|c|TypedColumn::application_supplied(c.sensitive_name().sensitive_text(),c.sensitive_name().clone(),ColumnType::Native(c.sql_type()),false,ColumnRole::Ordinary,true).unwrap()).collect(),ModelCoverage::Complete,&current).unwrap();
    (current,mapping)
}
#[test] fn final_argument_budget() {
    let (c,m)=budget_context();let r=patchbudget::generated_items::Repository::bind(&c,&m).unwrap();
"#);
    for (size, expected) in [(4096, "Budget"), (4088, "Ok")] {
        runtime_source.push_str("let p=patchbudget::generated_items::Patch::new();");
        for i in 0..15 {
            runtime_source.push_str(&format!(
                "let p=p.set_text{i}(\"x\".repeat(4096)).unwrap();"
            ));
        }
        runtime_source.push_str(&format!("let p=p.set_text15(\"x\".repeat({size})).unwrap();let result=r.compile_patch(patchbudget::generated_items::Key::new(1),p);"));
        if expected == "Budget" {
            runtime_source
                .push_str("assert_eq!(result.unwrap_err(),quent::codegen::PatchError::Budget);");
        } else {
            runtime_source.push_str("let p=result.unwrap();let i=p.inspect();let s=i.sensitive_source();assert_eq!(s.args().len(),17);assert_eq!(s.metadata().ordered_write_data().len(),16);assert_eq!(s.args().last(),Some(&quent_core::Value::I64(1)));");
        }
    }
    runtime_source.push('}');
    fs::write(dir.join("src/lib.rs"), runtime_source).unwrap();
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
        "fresh generated consumer: 76 diagnostic-checked negatives, 8 positive controls in 6 invocations (original 37 negatives/6 controls retained); four finite feature combinations"
    );
}
