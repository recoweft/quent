#![cfg(feature = "codegen")]
mod codegen_support;
use codegen_support::*;
use quent::codegen::*;
use quent_core::*;
use serde_json::{Value as Json, json};
#[allow(dead_code)]
mod pg {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/codegen-v2/postgres-generated.rs"
    ));
}
#[allow(dead_code)]
mod my {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/codegen-v2/mysql-generated.rs"
    ));
}
fn bytes(version: u8, d: &str, kind: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../fixtures/codegen-v{version}/{d}-{kind}.json")),
    )
    .unwrap()
}
fn protected(d: Dialect) -> DatabaseContext {
    protected_variant(d, 0)
}
fn protected_variant(d: Dialect, variant: u8) -> DatabaseContext {
    let cs = columns(d, "generated_items");
    context(
        d,
        variant,
        vec![
            PolicyDeclaration::Readonly(cs[3].clone()),
            PolicyDeclaration::Generated(cs[4].clone()),
        ],
    )
}
#[test]
fn c01_c07_c10_c11_patch_exact_native_sql_order_metadata_and_refusal() {
    use pg::generated_items::*;
    let c = protected(Dialect::Postgres);
    let m = mapping(&c, Dialect::Postgres, "generated_items");
    let repo = Repository::bind(&c, &m).unwrap();
    assert_eq!(
        repo.compile_patch(Key::new(1, 2), Patch::new())
            .unwrap_err(),
        PatchError::Empty
    );
    assert_eq!(
        Patch::new().set_big(1).unwrap().set_null_big().unwrap_err(),
        PatchError::Duplicate
    );
    assert_eq!(
        Patch::new().set_null_big().unwrap().set_big(1).unwrap_err(),
        PatchError::Duplicate
    );
    assert_eq!(
        Patch::new().set_txt("x".repeat(4097)).unwrap_err(),
        PatchError::Native
    );
    assert_eq!(
        Patch::new().set_txt("x\0".into()).unwrap_err(),
        PatchError::Native
    );
    let p = Patch::new()
        .set_optional(i64::MAX)
        .unwrap()
        .set_null_big()
        .unwrap()
        .set_txt(String::new())
        .unwrap()
        .set_flag(false)
        .unwrap();
    let compiled = repo.compile_patch(Key::new(i16::MIN, i64::MIN), p).unwrap();
    let inspect = compiled.inspect();
    let source = inspect.sensitive_source();
    assert_eq!(
        source.sql(),
        "UPDATE \"quent_dispatch_test\".\"generated_items\" SET \"big\" = $1, \"flag\" = $2, \"txt\" = $3, \"optional\" = $4 WHERE (((\"generated_items\".\"part\" = $5) AND (\"generated_items\".\"id\" = $6)) AND (\"generated_items\".\"tenant\" = $7))"
    );
    assert_eq!(
        source.args(),
        &[
            Value::Null(SqlType::I64),
            Value::Bool(false),
            Value::Text(String::new()),
            Value::I64(i64::MAX),
            Value::I16(i16::MIN),
            Value::I64(i64::MIN),
            Value::I64(11)
        ]
    );
    assert_eq!(source.metadata().family(), Family::Update);
    assert!(!source.caller_missing_where());
    assert_eq!(source.metadata().ordered_write_data().len(), 4);
    assert_eq!(inspect.risk(), Risk::Low);
    assert_eq!(inspect.scope(), Scope::Single);
    assert!(compiled.matches_current(&c, &m));
    for variant in 1..=10 {
        let other = protected_variant(Dialect::Postgres, variant);
        let other_mapping = mapping(&other, Dialect::Postgres, "generated_items");
        assert!(!compiled.matches_current(&other, &other_mapping));
        if matches!(variant, 3 | 4 | 7 | 8) {
            let new = Repository::bind(&other, &other_mapping)
                .unwrap()
                .compile_patch(Key::new(1, 2), Patch::new().set_flag(true).unwrap())
                .unwrap();
            assert!(new.matches_current(&other, &other_mapping));
            assert!(!new.matches_current(&c, &m));
            if variant == 4 {
                assert_eq!(
                    new.inspect().sensitive_source().args().last(),
                    Some(&Value::I64(22))
                );
            }
        }
    }
    assert!(Repository::bind(&context(Dialect::Postgres, 0, vec![]), &m).is_err());
    let wrong = mapping(&c, Dialect::Postgres, "other_items");
    assert!(Repository::bind(&c, &wrong).is_err());
    // Final common layer independently rejects all three protected categories.
    for index in [2, 3, 4] {
        let col = columns(Dialect::Postgres, "generated_items")[index].clone();
        let ty = col.sql_type();
        let op = Draft::update(
            table("generated_items"),
            vec![(
                col,
                Input::Value(match ty {
                    SqlType::I16 => Value::I16(1),
                    SqlType::I32 => Value::I32(1),
                    _ => Value::I64(11),
                }),
            )],
        )
        .filter(quent_core::Predicate::Compare(
            columns(Dialect::Postgres, "generated_items")[1].clone(),
            Compare::Eq,
            Value::I64(2),
        ))
        .freeze();
        assert!(c.prepare().prepare_operation(&op).is_err());
    }
}
#[test]
fn c07_mysql_all_widths_extrema_typed_null_and_unchanged() {
    use my::generated_items::*;
    let c = context(Dialect::Mysql, 0, vec![]);
    let m = mapping(&c, Dialect::Mysql, "generated_items");
    let r = Repository::bind(&c, &m).unwrap();
    for max in [false, true] {
        let p = Patch::new()
            .set_small(if max { i16::MAX } else { i16::MIN })
            .unwrap()
            .set_normal(if max { i32::MAX } else { i32::MIN })
            .unwrap()
            .set_big(if max { i64::MAX } else { i64::MIN })
            .unwrap()
            .set_tiny(if max { i8::MAX } else { i8::MIN })
            .unwrap()
            .set_u_tiny(if max { u8::MAX } else { 0 })
            .unwrap()
            .set_u_small(if max { u16::MAX } else { 0 })
            .unwrap()
            .set_u_normal(if max { u32::MAX } else { 0 })
            .unwrap()
            .set_u_big(if max { u64::MAX } else { 0 })
            .unwrap()
            .set_flag(max)
            .unwrap()
            .set_txt(String::new())
            .unwrap();
        let compiled = r.compile_patch(Key::new(1, 2), p).unwrap();
        let inspect = compiled.inspect();
        let s = inspect.sensitive_source();
        assert_eq!(
            s.args(),
            &[
                Value::I16(if max { i16::MAX } else { i16::MIN }),
                Value::I32(if max { i32::MAX } else { i32::MIN }),
                Value::I64(if max { i64::MAX } else { i64::MIN }),
                Value::Bool(max),
                Value::Text(String::new()),
                Value::I8(if max { i8::MAX } else { i8::MIN }),
                Value::U8(if max { u8::MAX } else { 0 }),
                Value::U16(if max { u16::MAX } else { 0 }),
                Value::U32(if max { u32::MAX } else { 0 }),
                Value::U64(if max { u64::MAX } else { 0 }),
                Value::I16(1),
                Value::I64(2),
                Value::I64(11)
            ]
        );
        assert_eq!(s.metadata().ordered_write_data().len(), 10);
        assert!(!s.sql().contains("`optional` ="));
    }
    let p = Patch::new()
        .set_null_small()
        .unwrap()
        .set_null_normal()
        .unwrap()
        .set_null_big()
        .unwrap()
        .set_null_flag()
        .unwrap()
        .set_null_txt()
        .unwrap()
        .set_null_optional()
        .unwrap()
        .set_null_tiny()
        .unwrap()
        .set_null_u_tiny()
        .unwrap()
        .set_null_u_small()
        .unwrap()
        .set_null_u_normal()
        .unwrap()
        .set_null_u_big()
        .unwrap();
    let compiled = r.compile_patch(Key::new(1, 2), p).unwrap();
    let i = compiled.inspect();
    assert_eq!(
        &i.sensitive_source().args()[..11],
        &[
            Value::Null(SqlType::I16),
            Value::Null(SqlType::I32),
            Value::Null(SqlType::I64),
            Value::Null(SqlType::Bool),
            Value::Null(SqlType::Text),
            Value::Null(SqlType::I64),
            Value::Null(SqlType::I8),
            Value::Null(SqlType::U8),
            Value::Null(SqlType::U16),
            Value::Null(SqlType::U32),
            Value::Null(SqlType::U64)
        ]
    );
}
#[test]
fn c15_patch_and_regeneration_redaction() {
    let p = pg::generated_items::Patch::new()
        .set_txt("CANARY_name_value_path".into())
        .unwrap();
    assert!(!format!("{p:?}").contains("CANARY"));
    let c = protected(Dialect::Postgres);
    let m = mapping(&c, Dialect::Postgres, "generated_items");
    let compiled = pg::generated_items::Repository::bind(&c, &m)
        .unwrap()
        .compile_patch(pg::generated_items::Key::new(1, 2), p)
        .unwrap();
    assert!(!format!("{compiled:?} {}", compiled.public_view().to_json()).contains("CANARY"));
    for e in [
        PatchError::Empty,
        PatchError::Duplicate,
        PatchError::Native,
        PatchError::Budget,
        PatchError::Binding,
        PatchError::Policy,
        PatchError::Gate,
    ] {
        assert!(std::error::Error::source(&e).is_none());
        assert!(!format!("{e} {:?} {}", e, e.public_view().to_json()).contains("CANARY"));
    }
}
#[test]
fn c19_c24_regeneration_exact_versions_domains_and_nondestructive_bytes() {
    for d in ["postgres", "mysql"] {
        let s = bytes(2, d, "source");
        let c = bytes(2, d, "config");
        let a = generate(&s, &c).unwrap();
        let golden = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../fixtures/codegen-v2/{d}-generated.rs")),
        )
        .unwrap();
        assert_eq!(a.sensitive_rust_source().as_bytes(), golden);
        let again = generate(&s, &c).unwrap();
        assert_eq!(a.sensitive_rust_source(), again.sensitive_rust_source());
        assert_eq!(
            compare(&a, &again, Some(&golden)).state(),
            RegenerationState::NoChange
        );
        assert_eq!(
            compare(&a, &again, None).state(),
            RegenerationState::GeneratedChanged
        );
        let old = generate(&bytes(1, d, "source"), &bytes(1, d, "config")).unwrap();
        assert_eq!(
            compare(&old, &a, Some(old.sensitive_rust_source().as_bytes())).state(),
            RegenerationState::Unsupported
        );
        for bad in [
            b"// kind=quent_rust_code version=2 generator=quent-codegen/2; CANARY_handwritten"
                .to_vec(),
            golden
                .iter()
                .copied()
                .chain(b"// changed CANARY".iter().copied())
                .collect(),
            a.sensitive_rust_source().replace('\n', "\r\n").into_bytes(),
            vec![],
        ] {
            let original = bad.clone();
            let summary = compare(&a, &again, Some(&bad));
            assert_eq!(summary.state(), RegenerationState::Conflict);
            assert_eq!(bad, original);
            assert!(!format!("{summary:?} {}", summary.to_json()).contains("CANARY"));
        }
        for bad in [vec![b'x'; 524289], vec![0xff]] {
            assert_eq!(
                compare(&a, &again, Some(&bad)).state(),
                RegenerationState::Unsupported
            );
        }
        for path in [
            "/schema/target/address",
            "/schema/tables/0/keys/0/columns",
            "/schema/tables/0/columns/5/nullability",
            "/policy/rules/2/kind",
        ] {
            let mut input: Json = serde_json::from_slice(&s).unwrap();
            match path {
                "/schema/target/address" => {
                    input["schema"]["target"]["address"] = json!("CANARY_target");
                    input["policy"]["target"]["address"] = json!("CANARY_target");
                }
                "/schema/tables/0/keys/0/columns" => {
                    *input.pointer_mut(path).unwrap() = json!(["id", "part"])
                }
                "/schema/tables/0/columns/5/nullability" => {
                    *input.pointer_mut(path).unwrap() = json!("nonnull")
                }
                _ => *input.pointer_mut(path).unwrap() = json!("pii"),
            }
            let next = generate(&serde_json::to_vec(&input).unwrap(), &c).unwrap();
            let result = compare(&a, &next, Some(&golden));
            assert_eq!(result.state(), RegenerationState::GeneratedChanged);
            assert!(!result.to_json().contains("CANARY"));
            assert_eq!(
                compare(&a, &next, Some(next.sensitive_rust_source().as_bytes())).state(),
                RegenerationState::Conflict
            );
        }
    }
}

#[test]
fn c10_c24_v2_config_reduction_revision_and_naming_refuse() {
    let source = bytes(2, "postgres", "source");
    let config = bytes(2, "postgres", "config");
    for (version, generator) in [
        (1, "quent-codegen/2"),
        (2, "quent-codegen/1"),
        (3, "quent-codegen/3"),
    ] {
        let mut cfg: Json = serde_json::from_slice(&config).unwrap();
        cfg["version"] = json!(version);
        cfg["generator"] = json!(generator);
        assert_eq!(
            generate(&source, &serde_json::to_vec(&cfg).unwrap()).unwrap_err(),
            CodegenError::Version
        );
    }
    let mut cfg: Json = serde_json::from_slice(&config).unwrap();
    cfg["models"][0]["columns"][7]["read"] = json!(false);
    for p in cfg["models"][0]["projections"].as_array_mut().unwrap() {
        p["fields"].as_array_mut().unwrap().retain(|v| v != "txt");
    }
    let reduced = generate(&source, &serde_json::to_vec(&cfg).unwrap()).unwrap();
    let first = reduced
        .sensitive_rust_source()
        .split("pub mod other_items")
        .next()
        .unwrap();
    assert!(!first.contains("fn set_txt("));
    assert!(!first.contains("fn set_null_txt("));
    for reserved in [
        "patch",
        "compiled_patch",
        "compile_patch",
        "matches_current",
    ] {
        let mut s: Json = serde_json::from_slice(&source).unwrap();
        let mut c: Json = serde_json::from_slice(&config).unwrap();
        s["schema"]["tables"][0]["columns"][8]["name"] = json!(reserved);
        c["models"][0]["columns"][8]["name"] = json!(reserved);
        for p in c["models"][0]["projections"].as_array_mut().unwrap() {
            for f in p["fields"].as_array_mut().unwrap() {
                if f == "optional" {
                    *f = json!(reserved);
                }
            }
        }
        assert_eq!(
            generate(
                &serde_json::to_vec(&s).unwrap(),
                &serde_json::to_vec(&c).unwrap()
            )
            .unwrap_err(),
            CodegenError::Naming
        );
    }
    let mut s: Json = serde_json::from_slice(&source).unwrap();
    let mut c: Json = serde_json::from_slice(&config).unwrap();
    s["schema"]["tables"][0]["columns"][8]["name"] = json!("null_big");
    c["models"][0]["columns"][8]["name"] = json!("null_big");
    for p in c["models"][0]["projections"].as_array_mut().unwrap() {
        for f in p["fields"].as_array_mut().unwrap() {
            if f == "optional" {
                *f = json!("null_big");
            }
        }
    }
    assert_eq!(
        generate(
            &serde_json::to_vec(&s).unwrap(),
            &serde_json::to_vec(&c).unwrap()
        )
        .unwrap_err(),
        CodegenError::Naming
    );
}
