#![cfg(feature = "operation-spec")]
use quent::operation_spec::*;
use quent_core::*;
use serde_json::{Value as Json, json};
fn base() -> Json {
    json!({"kind":"quent_operation_spec","version":1,"model":"items","fields":["id"],"order":[],"limit":10})
}
fn parse(j: &Json) -> Result<SpecSource, SpecError> {
    SpecSource::parse(&serde_json::to_vec(j).unwrap())
}
fn context(dialect: Dialect, target: &str, subject: &str) -> DatabaseContext {
    let b = DatabaseBinding::application_supplied(
        DatabaseIdentity::new(target).unwrap(),
        TargetAddress::new("route").unwrap(),
        Ident::new("catalog").unwrap(),
        dialect,
    );
    let g = Generation::new(1).unwrap();
    let p = PolicySet::new(b.clone(), PolicyIdentity::new("policy").unwrap(), g, vec![]).unwrap();
    let s = Settings::new(b.clone(), SettingsIdentity::new("settings").unwrap(), g, p).unwrap();
    DatabaseContext::new(
        s,
        ApplicationContext::application_supplied(
            b,
            None,
            Some(SubjectIdentity::new(subject).unwrap()),
            None,
        )
        .unwrap(),
    )
    .unwrap()
}
fn contract(c: &DatabaseContext) -> ModelContract {
    ModelContract::application_asserted(
        "items",
        vec![
            ("id".into(), ColumnRole::Ordinary, true),
            ("account_key".into(), ColumnRole::Tenant, true),
            ("secret".into(), ColumnRole::Ordinary, false),
            ("mystery".into(), ColumnRole::Unknown, true),
        ],
        c,
    )
    .unwrap()
}
#[test]
fn c01_c07_independent_structure_both_dialects_no_compilation() {
    let bytes = br#"{"kind":"quent_operation_spec","version":1,"model":"items","fields":["id"],"filter":{"op":"and","args":[{"op":"eq","field":"id","value":{"type":"i64","value":-9223372036854775808}},{"op":"or","args":[{"op":"not","arg":{"op":"is_null","field":"id"}},{"op":"in","field":"id","values":[{"type":"u64","value":18446744073709551615},{"type":"i32","value":null}]}]}]},"order":[{"field":"id","direction":"desc"}],"limit":1}"#;
    let expected = json!({"kind":"quent_operation_spec_summary","version":1,"state":"syntax_only","counts":{"fields":1,"order":1,"predicates":6,"branches":4,"literals":3,"literal_bytes":16}});
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let source = SpecSource::parse(bytes).unwrap();
        // Independent expected tree, not serialization roundtrip alone.
        assert_eq!(
            source.sensitive_source()["filter"]["args"][0]["value"],
            json!({"type":"i64","value":i64::MIN})
        );
        assert_eq!(
            source.sensitive_source()["filter"]["args"][1]["args"][1]["values"][0]["value"]
                .as_u64(),
            Some(u64::MAX)
        );
        assert_eq!(
            serde_json::from_str::<Json>(&source.public_view().to_json()).unwrap(),
            expected
        );
        let c = context(d, "db", "user");
        let checked = source.check_roles(contract(&c), &c).unwrap();
        assert!(checked.matches_snapshot(&c));
        assert!(
            checked
                .public_view()
                .to_json()
                .contains("application_roles_checked")
        );
        assert_eq!(checked.sensitive_source()["limit"], 1);
    }
}
#[test]
fn c07_native_width_sign_extrema_overflow_and_tags() {
    for (tag, min, max, under, over) in [
        ("i8", "-128", "127", "-129", "128"),
        ("i16", "-32768", "32767", "-32769", "32768"),
        (
            "i32",
            "-2147483648",
            "2147483647",
            "-2147483649",
            "2147483648",
        ),
        (
            "i64",
            "-9223372036854775808",
            "9223372036854775807",
            "-9223372036854775809",
            "9223372036854775808",
        ),
        ("u8", "0", "255", "-1", "256"),
        ("u16", "0", "65535", "-1", "65536"),
        ("u32", "0", "4294967295", "-1", "4294967296"),
        (
            "u64",
            "0",
            "18446744073709551615",
            "-1",
            "18446744073709551616",
        ),
    ] {
        for (n, ok) in [
            (min, true),
            (max, true),
            (under, false),
            (over, false),
            ("null", true),
            ("1.0", false),
            ("1e0", false),
            ("\"1\"", false),
            ("true", false),
        ] {
            let s = format!(
                r#"{{"kind":"quent_operation_spec","version":1,"model":"items","fields":["id"],"order":[],"limit":1,"filter":{{"op":"eq","field":"id","value":{{"type":"{tag}","value":{n}}}}}}}"#
            );
            assert_eq!(SpecSource::parse(s.as_bytes()).is_ok(), ok, "{tag}/{n}");
        }
    }
    for (tag, value, ok) in [
        ("bool", json!(false), true),
        ("bool", json!(0), false),
        ("text", json!("a\n\"\\雪🦀"), true),
        ("text", json!(1), false),
        ("bool", Json::Null, true),
        ("text", Json::Null, true),
    ] {
        let mut j = base();
        j["filter"] = json!({"op":"eq","field":"id","value":{"type":tag,"value":value}});
        assert_eq!(parse(&j).is_ok(), ok);
    }
    for tag in [
        "float", "decimal", "uuid", "time", "date", "bytes", "unknown",
    ] {
        let mut j = base();
        j["filter"] = json!({"op":"eq","field":"id","value":{"type":tag,"value":null}});
        assert_eq!(parse(&j).unwrap_err(), SpecError::Unsupported);
    }
}
#[test]
fn c24_closed_shapes_and_null_distinctions() {
    for key in ["kind", "version", "model", "fields", "order", "limit"] {
        let mut j = base();
        j.as_object_mut().unwrap().remove(key);
        assert!(parse(&j).is_err());
        let mut j = base();
        j[key] = Json::Null;
        assert!(parse(&j).is_err());
    }
    for (key, value) in [
        ("fields", json!([])),
        ("fields", json!(["id", "id"])),
        ("fields", json!(["x.y"])),
        (
            "order",
            json!([{"field":"id","direction":"asc"},{"field":"id","direction":"desc"}]),
        ),
        ("limit", json!(0)),
        ("limit", json!(10001)),
        ("version", json!(2)),
        ("kind", json!("quent_manifest")),
        ("filter", Json::Null),
    ] {
        let mut j = base();
        j[key] = value;
        assert!(parse(&j).is_err(), "{key}");
    }
    for op in ["and", "or", "in"] {
        let mut j = base();
        j["filter"] = if op == "in" {
            json!({"op":op,"field":"id","values":[]})
        } else {
            json!({"op":op,"args":[]})
        };
        assert!(parse(&j).is_err());
    }
    for op in [
        "eq", "ne", "lt", "le", "gt", "ge", "in", "is_null", "not", "and", "or",
    ] {
        let leaf = json!({"op":"eq","field":"id","value":{"type":"i32","value":null}});
        let filter = match op {
            "and" | "or" => json!({"op":op,"args":[leaf]}),
            "not" => json!({"op":op,"arg":leaf}),
            "is_null" => json!({"op":op,"field":"id"}),
            "in" => json!({"op":op,"field":"id","values":[{"type":"i32","value":null}]}),
            _ => json!({"op":op,"field":"id","value":{"type":"i32","value":null}}),
        };
        let mut j = base();
        j["filter"] = filter;
        assert!(parse(&j).is_ok(), "{op}");
        j["filter"]["reason"] = json!("hidden");
        assert!(parse(&j).is_err());
    }
    for op in [
        "update",
        "delete",
        "insert",
        "raw",
        "join",
        "union",
        "select",
        "function",
        "add",
        "apply",
        "command",
        "returning",
        "batch",
    ] {
        let mut j = base();
        j["filter"] = json!({"op":op});
        assert!(parse(&j).is_err());
        let mut j = base();
        j[op] = json!(true);
        assert!(parse(&j).is_err());
    }
    let mut j = base();
    j["filter"] = json!({"op":"eq","field":"id","value":{"type":"i32"}});
    assert!(parse(&j).is_err());
    j["filter"]["value"] = Json::Null;
    assert!(parse(&j).is_err());
}
#[test]
fn c24_duplicates_utf8_multidoc_and_preallocation_budgets() {
    for s in [
        r#"{"kind":1,"kind":2}"#,
        r#"{"kind":1,"\u006bind":2}"#,
        r#"{"x":{"a":1,"\u0061":2}}"#,
        r#"{"x":[{"a":1,"a":2}]}"#,
        r#"{} {}"#,
        r#"{"x":1e9999}"#,
    ] {
        assert!(SpecSource::parse(s.as_bytes()).is_err());
    }
    let valid = serde_json::to_string(&base()).unwrap();
    let duplicate = valid.replacen("\"limit\":10", "\"limit\":10,\"\\u006cimit\":1", 1);
    assert!(SpecSource::parse(duplicate.as_bytes()).is_err());
    let valid_nested = r#"{"kind":"quent_operation_spec","version":1,"model":"items","fields":["id"],"order":[],"limit":1,"filter":{"op":"not","arg":{"op":"in","field":"id","values":[{"type":"i64","value":1}]}}}"#;
    assert!(SpecSource::parse(valid_nested.as_bytes()).is_ok());
    for (from, to) in [
        ("\"field\":\"id\"", r#""field":"id","\u0066ield":"id""#),
        ("\"value\":1", r#""value":1,"\u0076alue":2"#),
        ("\"op\":\"not\"", r#""op":"not","op":"not""#),
    ] {
        let duplicate = valid_nested.replacen(from, to, 1);
        assert!(SpecSource::parse(duplicate.as_bytes()).is_err());
    }
    assert!(SpecSource::parse(&[0xff, 0xfe]).is_err());
    assert_eq!(
        SpecSource::parse(&vec![b' '; 65537]).unwrap_err(),
        SpecError::Budget
    );
    assert!(
        SpecSource::parse(format!("{}0{}", "[".repeat(10000), "]".repeat(10000)).as_bytes())
            .is_err()
    );
    assert!(SpecSource::parse(format!("[{}]", vec!["0"; 10000].join(",")).as_bytes()).is_err());
    let mut j = base();
    j["fields"] = json!((0..64).map(|n| format!("f{n}")).collect::<Vec<_>>());
    assert!(parse(&j).is_ok());
    j["fields"].as_array_mut().unwrap().push(json!("last"));
    assert_eq!(parse(&j).unwrap_err(), SpecError::Budget);
    let mut j = base();
    j["order"] = json!(
        (0..16)
            .map(|n| json!({"field":format!("f{n}"),"direction":"asc"}))
            .collect::<Vec<_>>()
    );
    assert!(parse(&j).is_ok());
    j["order"]
        .as_array_mut()
        .unwrap()
        .push(json!({"field":"last","direction":"asc"}));
    assert!(parse(&j).is_err());
    let mut j = base();
    j["filter"] = json!({"op":"in","field":"id","values":vec![json!({"type":"i8","value":1});64]});
    assert!(parse(&j).is_ok());
    j["filter"]["values"]
        .as_array_mut()
        .unwrap()
        .push(json!({"type":"i8","value":1}));
    assert!(parse(&j).is_err());
    let mut j = base();
    j["filter"] = json!({"op":"eq","field":"id","value":{"type":"text","value":"x".repeat(4096)}});
    assert!(parse(&j).is_ok());
    j["filter"]["value"]["value"] = json!("x".repeat(4097));
    assert!(parse(&j).is_err());
    let leaf = json!({"op":"eq","field":"id","value":{"type":"text","value":"x".repeat(4096)}});
    let mut j = base();
    j["filter"] = json!({"op":"and","args":vec![leaf.clone();4]});
    assert!(parse(&j).is_ok());
    j["filter"]["args"].as_array_mut().unwrap().push(leaf);
    assert!(parse(&j).is_err());
}
#[test]
fn c08_c11_c12_nested_tenant_unknown_and_exact_snapshot() {
    for dialect in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(dialect, "db", "user");
        for field in ["account_key", "mystery", "absent", "secret"] {
            let leaf = json!({"op":"in","field":field,"values":[{"type":"i64","value":7}]});
            for filter in [
                leaf.clone(),
                json!({"op":"not","arg":leaf}),
                json!({"op":"or","args":[{"op":"is_null","field":"id"},leaf]}),
                json!({"op":"and","args":[leaf]}),
            ] {
                let mut j = base();
                j["filter"] = filter;
                let err = parse(&j)
                    .unwrap()
                    .check_roles(contract(&c), &c)
                    .unwrap_err();
                assert_eq!(
                    err,
                    match field {
                        "account_key" => SpecError::TenantPredicate,
                        "secret" => SpecError::ReadDenied,
                        _ => SpecError::Unverified,
                    }
                );
            }
        }
        for claim in [
            "tenant",
            "subject",
            "allow",
            "reason",
            "fresh",
            "permit",
            "policy",
            "settings",
            "schema",
            "target",
            "generation",
            "alias",
            "metadata",
            "rawSQL",
            "tablepath",
        ] {
            let mut j = base();
            j[claim] = json!(true);
            assert!(parse(&j).is_err());
            let mut j = base();
            j["filter"] = json!({"op":"not","arg":{"op":"in","field":"id","values":[{"type":"i64","value":1,claim:7}]}});
            assert!(parse(&j).is_err());
        }
        let mut j = base();
        j["model"] = json!("other");
        assert_eq!(
            parse(&j)
                .unwrap()
                .check_roles(contract(&c), &c)
                .unwrap_err(),
            SpecError::Unverified
        );
        for other in [
            context(dialect, "other", "user"),
            context(dialect, "db", "other"),
        ] {
            assert_eq!(
                parse(&base())
                    .unwrap()
                    .check_roles(contract(&c), &other)
                    .unwrap_err(),
                SpecError::SnapshotMismatch
            );
            let checked = parse(&base())
                .unwrap()
                .check_roles(contract(&c), &c)
                .unwrap();
            assert!(!checked.matches_snapshot(&other));
        }
        let unknown = ModelContract::application_asserted(
            "items",
            vec![("id".into(), ColumnRole::Unknown, true)],
            &c,
        )
        .unwrap();
        assert_eq!(
            parse(&base())
                .unwrap()
                .check_roles(unknown, &c)
                .unwrap_err(),
            SpecError::Unverified
        );
        let mut j = base();
        j["fields"] = json!(["account_key"]);
        assert!(parse(&j).unwrap().check_roles(contract(&c), &c).is_ok());
    }
}
#[test]
fn c15_public_canaries_errors_and_sensitive_separation() {
    let c = context(Dialect::Postgres, "canary_target", "canary_subject");
    let mut j = base();
    j["model"] = json!("canary_model");
    j["fields"] = json!(["canary_field"]);
    j["filter"] = json!({"op":"eq","field":"canary_field","value":{"type":"text","value":"CANARY_literal_雪"}});
    let s = parse(&j).unwrap();
    assert_eq!(s.sensitive_source(), &j);
    let m = ModelContract::application_asserted(
        "canary_model",
        vec![("canary_field".into(), ColumnRole::Ordinary, true)],
        &c,
    )
    .unwrap();
    for out in [
        format!("{s:?}"),
        format!("{m:?}"),
        format!("{:?}", s.public_view()),
        s.public_view().to_json(),
    ] {
        assert!(!out.contains("canary"));
        assert!(!out.contains("CANARY"));
    }
    let checked = s.check_roles(m, &c).unwrap();
    for out in [format!("{checked:?}"), checked.public_view().to_json()] {
        assert!(!out.contains("canary"));
        assert!(!out.contains("CANARY"));
    }
    use std::error::Error;
    for e in [
        SpecError::Wire,
        SpecError::Budget,
        SpecError::Shape,
        SpecError::Unsupported,
        SpecError::Range,
        SpecError::Unverified,
        SpecError::ReadDenied,
        SpecError::TenantPredicate,
        SpecError::SnapshotMismatch,
    ] {
        assert!(e.source().is_none());
        for out in [
            e.to_string(),
            format!("{e:?}"),
            e.public_view().to_json(),
            format!("{:?}", e.public_view()),
        ] {
            assert!(!out.contains("canary"));
        }
    }
}
#[cfg(feature = "output-tracing")]
#[path = "output_capture/mod.rs"]
mod output_capture;
#[test]
#[cfg(feature = "output-tracing")]
fn c01_c15_no_implicit_tracing() {
    let capture = output_capture::Capture::default();
    tracing::subscriber::with_default(capture.clone(), || {
        let c = context(Dialect::Postgres, "canary_target", "canary_subject");
        let _ = parse(&base())
            .unwrap()
            .check_roles(contract(&c), &c)
            .unwrap()
            .public_view()
            .to_json();
        let _ = SpecSource::parse(b"CANARY_bad")
            .unwrap_err()
            .public_view()
            .to_json();
    });
    assert!(capture.0.lock().unwrap().is_empty());
}

#[test]
fn c24_semantic_predicate_depth_nodes_and_exact_source_bytes() {
    let leaf = json!({"op":"is_null","field":"id"});
    for (n, ok) in [(13, true), (14, false)] {
        let mut filter = json!({"op":"eq","field":"id","value":{"type":"i8","value":1}});
        for _ in 0..n {
            filter = json!({"op":"not","arg":filter});
        }
        let mut j = base();
        j["filter"] = filter;
        assert_eq!(parse(&j).is_ok(), ok);
    }
    for (n, ok) in [(32, true), (33, false)] {
        let mut j = base();
        j["filter"] = json!({"op":"and","args":vec![leaf.clone();n]});
        assert_eq!(parse(&j).is_ok(), ok);
    }
    for (last, ok) in [(23, true), (24, false)] {
        // 1 root + 8 intermediate + 7*32 + last = 256/257 predicates.
        let mut args = vec![json!({"op":"or","args":vec![leaf.clone();32]}); 7];
        args.push(json!({"op":"or","args":vec![leaf.clone();last]}));
        let mut j = base();
        j["filter"] = json!({"op":"and","args":args});
        assert_eq!(parse(&j).is_ok(), ok);
    }
    let mut bytes = serde_json::to_vec(&base()).unwrap();
    bytes.resize(65536, b' ');
    assert!(SpecSource::parse(&bytes).is_ok());
    bytes.push(b' ');
    assert_eq!(SpecSource::parse(&bytes).unwrap_err(), SpecError::Budget);
}

#[test]
fn c24_total_literal_count_boundary() {
    for (extra, ok) in [(false, true), (true, false)] {
        let mut args = vec![
            json!({"op":"in","field":"id","values":vec![json!({"type":"i8","value":1});64]});
            8
        ];
        if extra {
            args.push(json!({"op":"eq","field":"id","value":{"type":"i8","value":1}}));
        }
        let mut j = base();
        j["filter"] = json!({"op":"and","args":args});
        assert_eq!(parse(&j).is_ok(), ok);
    }
}
