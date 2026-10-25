#![cfg(feature = "codegen")]
mod codegen_support;
use codegen_support::*;
use quent::codegen::*;
use quent::operation_spec::TypedError;
use quent_core::*;
use serde_json::{Value as Json, json};
fn bytes(d: &str, kind: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../fixtures/codegen-v1/{d}-{kind}.json")),
    )
    .unwrap()
}
#[test]
fn c01_c19_exact_bytes_independent_source_api_oracles_and_domains() {
    for d in ["postgres", "mysql"] {
        let source = bytes(d, "source");
        let config = bytes(d, "config");
        let a = generate(&source, &config).unwrap();
        for _ in 0..3 {
            assert_eq!(
                a.sensitive_rust_source(),
                generate(&source, &config).unwrap().sensitive_rust_source()
            );
        }
        let checked = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../fixtures/codegen-v1/{d}-generated.rs")),
        )
        .unwrap();
        assert_eq!(a.sensitive_rust_source(), checked);
        // Handwritten expected API/type fragments, independent from generated golden.
        for expected in [
            "pub struct Key(i16,i64);",
            "pub struct CompactRow {\noptional: Option<i64>,\nid: i64,",
            "pub fn part()->Column<i16>",
            "pub fn small()->Column<Option<i16>>",
            "pub fn flag()->Column<Option<bool>>",
            "pub fn txt()->Column<Option<String>>",
            "pub fn compact()->Projection<CompactRow>",
            "Model::part().eq(self.0)",
            "Model::id().eq(self.1)",
        ] {
            assert!(
                checked.contains(expected),
                "missing independent oracle {expected}"
            );
        }
        assert!(!checked.contains("pub fn tenant()"));
        assert!(!checked.contains("SELECT *"));
        if d == "mysql" {
            for t in ["i8", "u8", "u16", "u32", "u64"] {
                assert!(checked.contains(&format!("Column<Option<{t}>>")));
            }
        }
        let mut tree: Json = serde_json::from_slice(&source).unwrap();
        tree["schema"]["target"]["address"] = json!("changed");
        tree["policy"]["target"]["address"] = json!("changed");
        assert_ne!(
            checked,
            generate(&serde_json::to_vec(&tree).unwrap(), &config)
                .unwrap()
                .sensitive_rust_source()
        );
        let mut cfg: Json = serde_json::from_slice(&config).unwrap();
        cfg["models"][0]["projections"][1]["fields"] = json!(["id", "optional"]);
        assert_ne!(
            checked,
            generate(&source, &serde_json::to_vec(&cfg).unwrap())
                .unwrap()
                .sensitive_rust_source()
        );
        // Policy-only domain change produces distinct code, not a shared schema hash.
        let mut tree: Json = serde_json::from_slice(&source).unwrap();
        tree["policy"]["rules"].as_array_mut().unwrap().push(json!({"kind":"readonly","table":["quent_dispatch_test","generated_items"],"column":"txt"}));
        assert_ne!(
            checked,
            generate(&serde_json::to_vec(&tree).unwrap(), &config)
                .unwrap()
                .sensitive_rust_source()
        );
        assert!(!format!("{a:?} {}", a.public_view().to_json()).contains("dedicated_loopback"));
    }
}
#[test]
fn c24_strict_bounded_unsupported_naming_and_incomplete_refusal() {
    let source = bytes("postgres", "source");
    let cfg = bytes("postgres", "config");
    for bad in [
        b"{}".as_slice(),
        b"{\"version\":1,\"version\":1}",
        b"{\"version\":1,\"versi\\u006fn\":1}",
        b"{\"a\":1e2}",
        &vec![b' '; 65537],
    ] {
        assert!(generate(&source, bad).is_err());
    }
    for (path, value) in [
        ("/version", json!(2)),
        ("/generator", json!("unknown")),
        ("/allow", json!(true)),
        ("/models/0/columns/0/role", json!("unknown")),
        ("/models/0/columns/0/name", json!("alias")),
        ("/models/0/projections/0/fields", json!([])),
        ("/models/0/projections/0/fields", json!(["missing"])),
        ("/models/0/projections/0/fields", json!(["id", "id"])),
    ] {
        let mut c: Json = serde_json::from_slice(&cfg).unwrap();
        if path == "/allow" {
            c["allow"] = value;
        } else {
            *c.pointer_mut(path).unwrap() = value;
        }
        assert!(
            generate(&source, &serde_json::to_vec(&c).unwrap()).is_err(),
            "{path}"
        );
    }
    for spelling in [
        "type", "gen", "self", "Self", "KEY", "2items", "a\"b", "a`b", "é", "a-b", "a::b", "a__b",
        "a_", "a_1", "a1",
    ] {
        let mut c: Json = serde_json::from_slice(&cfg).unwrap();
        c["models"][0]["projections"][0]["name"] = json!(spelling);
        if matches!(spelling, "a_1" | "a1") {
            c["models"][0]["projections"][1]["name"] =
                json!(if spelling == "a_1" { "a1" } else { "a_1" });
        }
        assert!(
            generate(&source, &serde_json::to_vec(&c).unwrap()).is_err(),
            "{spelling}"
        );
    }
    for (path, value) in [
        ("/schema/coverage", json!("partial")),
        ("/schema/tables/0/semantics", json!("rls")),
        ("/schema/tables/0/columns/0/native", json!("unknown")),
        ("/schema/tables/0/columns/0/native", json!("unsupported")),
        ("/schema/tables/0/columns/0/nullability", json!("unknown")),
        ("/schema/tables/0/columns/0/nullability", json!("nullable")),
        (
            "/schema/tables/0/columns/0/default",
            json!({"state":"unsupported_expression","expression":"CANARY"}),
        ),
        ("/schema/tables/0/keys/0/coverage", json!("partial")),
        ("/schema/tables/0/keys/0/columns", json!([])),
        (
            "/schema/tables/0/keys/0/columns",
            json!(["part", "missing"]),
        ),
        ("/schema/tables/0/path", json!(["only_one"])),
        (
            "/schema/tables/0/columns/0/default",
            json!({"state":"literal","value":{"type":"i16","value":32768}}),
        ),
    ] {
        let mut s: Json = serde_json::from_slice(&source).unwrap();
        *s.pointer_mut(path).unwrap() = value;
        assert!(
            generate(&serde_json::to_vec(&s).unwrap(), &cfg).is_err(),
            "{path}"
        );
    }
    let mut c: Json = serde_json::from_slice(&cfg).unwrap();
    let p = c["models"][0]["projections"][0].clone();
    c["models"][0]["projections"] = json!(vec![p; 9]);
    assert!(generate(&source, &serde_json::to_vec(&c).unwrap()).is_err());
}
#[test]
fn c01_c07_c08_c11_c12_generated_mapping_projection_and_native_oracle() {
    use pg::generated_items::{Key, Model, Repository};
    let c = context(Dialect::Postgres, 0, vec![]);
    let m = mapping(&c, Dialect::Postgres, "generated_items");
    let repo = Repository::bind(&c, &m).unwrap();
    let key = Key::new(-16, 7);
    assert_eq!(key.sensitive_components(), (&-16, &7));
    let read = repo
        .select(
            Model::compact(),
            Some(key.predicate().unwrap()),
            vec![Model::id().desc()],
            10,
        )
        .unwrap();
    let inspection = read.inspect();
    let source = inspection.sensitive_source();
    assert_eq!(
        source.sql(),
        "SELECT \"generated_items\".\"optional\", \"generated_items\".\"id\" FROM \"quent_dispatch_test\".\"generated_items\" WHERE (((\"generated_items\".\"part\" = $1) AND (\"generated_items\".\"id\" = $2)) AND (\"generated_items\".\"tenant\" = $3)) ORDER BY \"generated_items\".\"id\" DESC LIMIT $4"
    );
    assert_eq!(
        source.args(),
        &[
            Value::I16(-16),
            Value::I64(7),
            Value::I64(11),
            Value::I64(10)
        ]
    );
    assert_eq!(inspection.scope(), Scope::Single);
    assert_eq!(
        Model::optional().eq(None).unwrap_err(),
        TypedError::NullComparison
    );
    assert!(
        repo.select(
            Model::compact(),
            Some(Model::optional().is_null()),
            vec![],
            10
        )
        .is_ok()
    );
    for variant in [1, 2, 5, 6, 9, 10] {
        let other = context(Dialect::Postgres, variant, vec![]);
        let mapping = mapping(&other, Dialect::Postgres, "generated_items");
        assert!(Repository::bind(&other, &mapping).is_err());
    }
    let other = mapping(&c, Dialect::Postgres, "other_items");
    assert!(Repository::bind(&c, &other).is_err());
    let mut p = Model::id().eq(1).unwrap();
    for _ in 0..8 {
        p = p.and(Model::id().eq(1).unwrap()).unwrap();
    }
    assert!(p.and(Model::id().eq(1).unwrap()).is_err());
}
#[test]
fn c15_generated_carriers_and_fixed_output_redact_canaries() {
    use pg::generated_items::*;
    let c = context(Dialect::Postgres, 0, vec![]);
    let m = mapping(&c, Dialect::Postgres, "generated_items");
    let repo = Repository::bind(&c, &m).unwrap();
    let predicate = Model::txt()
        .eq(Some("CANARY_SECRET_SQL_tenant_/path".into()))
        .unwrap();
    let text = format!(
        "{:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        Model,
        Key::new(16, 1122334455),
        Model::txt(),
        Model::detail(),
        Model::id().asc(),
        predicate,
        repo
    );
    assert!(!text.contains("CANARY"));
    assert!(!text.contains("1122334455"));
    assert!(!text.contains("generated_items"));
    let r = repo
        .select(Model::detail(), Some(predicate), vec![], 10)
        .unwrap();
    assert!(!format!("{r:?} {}", r.public_view().to_json()).contains("CANARY"));
    for e in [
        CodegenError::Wire,
        CodegenError::Version,
        CodegenError::Budget,
        CodegenError::Unsupported,
        CodegenError::Naming,
        CodegenError::Coverage,
        CodegenError::Mapping,
    ] {
        assert!(std::error::Error::source(&e).is_none());
        assert_eq!(
            serde_json::from_str::<Json>(&e.public_view().to_json())
                .unwrap()
                .as_object()
                .unwrap()
                .len(),
            4
        );
    }
}

#[test]
fn c24_model_column_item_budgets_and_ordered_identity() {
    let mut s: Json = serde_json::from_slice(&bytes("postgres", "source")).unwrap();
    let mut c: Json = serde_json::from_slice(&bytes("postgres", "config")).unwrap();
    let a = generate(
        &serde_json::to_vec(&s).unwrap(),
        &serde_json::to_vec(&c).unwrap(),
    )
    .unwrap();
    s["schema"]["tables"][0]["keys"][0]["columns"] = json!(["id", "part"]);
    let reversed = generate(
        &serde_json::to_vec(&s).unwrap(),
        &serde_json::to_vec(&c).unwrap(),
    )
    .unwrap();
    assert!(
        reversed
            .sensitive_rust_source()
            .contains("pub struct Key(i64,i16);")
    );
    assert_ne!(a.sensitive_rust_source(), reversed.sensitive_rust_source());
    let template = s["schema"]["tables"][0].clone();
    let model = c["models"][0].clone();
    s["policy"]["rules"] = json!([]);
    for (count, columns, projections, expected) in [
        (8, 1, 1, true),
        (9, 1, 1, false),
        (1, 32, 1, true),
        (1, 33, 1, false),
        (4, 32, 1, true),
        (5, 32, 1, false),
        (4, 32, 8, false),
    ] {
        let mut tables = vec![];
        let mut models = vec![];
        for i in 0..count {
            let mut table = template.clone();
            let mut m = model.clone();
            let path = json!(["quent_dispatch_test", format!("t{i}")]);
            table["path"] = path.clone();
            m["table"] = path;
            table["columns"]=json!((0..columns).map(|j|json!({"name":format!("f{j}"),"native":"i64","nullability":"nonnull","default":{"state":"none"}})).collect::<Vec<_>>());
            table["keys"] = json!([{"kind":"primary","coverage":"complete","columns":["f0"]}]);
            m["columns"] = json!(
                (0..columns)
                    .map(|j| json!({"name":format!("f{j}"),"role":"ordinary","read":true}))
                    .collect::<Vec<_>>()
            );
            m["projections"]=json!((0..projections).map(|j|json!({"name":format!("p{j}"),"fields":(0..columns).map(|k|format!("f{k}")).collect::<Vec<_>>()})).collect::<Vec<_>>());
            tables.push(table);
            models.push(m);
        }
        s["schema"]["tables"] = json!(tables);
        c["models"] = json!(models);
        let result = generate(
            &serde_json::to_vec(&s).unwrap(),
            &serde_json::to_vec(&c).unwrap(),
        );
        assert_eq!(
            result.is_ok(),
            expected,
            "models={count}, cols={columns}, projections={projections}: {:?}",
            result.err()
        );
    }
}
