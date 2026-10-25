#![cfg(feature = "review")]
use quent::review::*;
use quent_core::*;
use serde_json::{Value as Json, json};
const SOURCE: &[u8] = include_bytes!("../../../fixtures/review-v1/select.json");
fn source() -> Json {
    serde_json::from_slice(SOURCE).unwrap()
}
fn bytes(v: &Json) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}
fn output(r: &Review) -> Json {
    serde_json::from_str(&r.to_json()).unwrap()
}
#[test]
fn c01_c08_c12_c15_independent_expected_and_claim_nonpromotion() {
    let expected: Json = serde_json::from_slice(include_bytes!(
        "../../../fixtures/review-v1/select.expected.json"
    ))
    .unwrap();
    let r = plan_source(SOURCE).unwrap();
    assert_eq!(output(&r), expected);
    assert_eq!(r.exit_code(), 3);
    for _ in 0..8 {
        assert_eq!(plan_source(SOURCE).unwrap().to_json(), r.to_json());
    }
    for risk in ["low", "high", "blocked"] {
        let mut s = source();
        s["claims"] = json!({"sql":"SELECT CANARY_credential FROM CANARY_name", "args":[{"type":"text","value":"CANARY_value"}], "metadata":{"family":"delete","argument_count":999},"verdict":"accepted","risk":risk,"coverage":"complete","precision":"complete","freshness":"live_verified","fingerprint":"same_hash_CANARY"});
        let changed = plan_source(&bytes(&s)).unwrap();
        let mut o = output(&changed);
        assert_eq!(o["claims"], "unverified");
        assert_eq!(o["claim_count"], 9);
        o["claims"] = json!("absent");
        o["claim_count"] = json!(0);
        assert_eq!(o, expected);
        for view in [
            changed.to_json(),
            changed.to_pretty(),
            format!("{changed:?}"),
            changed.to_string(),
        ] {
            for secret in ["CANARY", "private_", "1234567", "SELECT", "same_hash"] {
                assert!(!view.contains(secret));
            }
        }
    }
    // Same fingerprint, different operation is not equality/authority.
    let mut a = source();
    a["claims"] = json!({"fingerprint":"same"});
    let mut b = a.clone();
    b["operation"]["family"] = json!("delete");
    b["operation"]["projection"] = json!([]);
    b["operation"].as_object_mut().unwrap().remove("filter");
    b["operation"].as_object_mut().unwrap().remove("limit");
    assert_ne!(
        output(&plan_source(&bytes(&a)).unwrap())["inspection"],
        output(&plan_source(&bytes(&b)).unwrap())["inspection"]
    );
    assert_eq!(plan_source(&bytes(&b)).unwrap().exit_code(), 5);
}
#[test]
fn c01_c24_all_families_native_null_and_block_survive() {
    for dialect in ["postgres", "mysql"] {
        for family in ["select", "count", "insert", "update", "delete"] {
            let mut v = source();
            v["dialect"] = json!(dialect);
            v["operation"]["family"] = json!(family);
            if family != "select" {
                v["operation"]["projection"] = json!([]);
                v["operation"].as_object_mut().unwrap().remove("limit");
            }
            if matches!(family, "insert" | "update") {
                v["operation"]["writes"] = json!([{"column":0,"value":{"type":"i64","value":0}}]);
            }
            v["operation"].as_object_mut().unwrap().remove("filter");
            let r = plan_source(&bytes(&v)).unwrap();
            let o = output(&r);
            assert_eq!(o["inspection"]["family"], family);
            if matches!(family, "update" | "delete") {
                assert_eq!(r.state(), State::Refused);
                assert_eq!(o["risk"], "blocked");
                assert_eq!(o["inspection"]["findings"], json!(["missing_where"]));
                assert_eq!(r.exit_code(), 5);
            } else {
                assert_eq!(r.state(), State::Unknown);
                assert_eq!(r.exit_code(), 3);
            }
            if family == "insert" {
                assert_eq!(o["inspection"]["findings"], json!(["insert_unverified"]));
            }
        }
    }
    for (ty, value) in [
        ("i8", json!(-128)),
        ("i16", json!(-32768)),
        ("i32", json!(2147483647)),
        ("i64", json!(i64::MIN)),
        ("u8", json!(255)),
        ("u16", json!(65535)),
        ("u32", json!(u32::MAX)),
        ("u64", json!(u64::MAX)),
        ("bool", json!(false)),
        ("text", json!("CANARY")),
    ] {
        let mut v = source();
        v["dialect"] = json!("mysql");
        v["operation"]["columns"][0]["type"] = json!(ty);
        v["operation"]["filter"]["value"] = json!({"type":ty,"value":value});
        assert!(plan_source(&bytes(&v)).is_ok(), "{ty}");
        v["operation"]["columns"][0]["nullable"] = json!(true);
        v["operation"]["filter"]["value"]["value"] = Json::Null;
        assert!(plan_source(&bytes(&v)).is_ok());
    }
}
#[test]
fn c24_closed_data_not_output_or_authority_and_resource_refusals() {
    for kind in [
        "quent_local_source",
        "quent_plan_view",
        "quent_diagnostic_view",
        "quent_review",
        "other",
    ] {
        let mut v = source();
        v["kind"] = json!(kind);
        assert_eq!(plan_source(&bytes(&v)), Err(ReviewError::Unsupported));
    }
    for mutate in [
        "version", "extra", "missing", "null", "permit", "native", "index", "range", "tenant",
        "role",
    ] {
        let mut v = source();
        match mutate {
            "version" => v["version"] = json!(2),
            "extra" => v["extra"] = json!("CANARY"),
            "missing" => {
                v.as_object_mut().unwrap().remove("dialect");
            }
            "null" => v["operation"]["filter"] = Json::Null,
            "permit" => v["claims"] = json!({"permit":true}),
            "native" => v["operation"]["columns"][0]["type"] = json!("decimal"),
            "index" => v["operation"]["projection"] = json!([64]),
            "range" => v["operation"]["filter"]["value"] = json!({"type":"i8","value":128}),
            "tenant" => v["operation"]["tenant"] = json!(1),
            "role" => v["operation"]["columns"][0]["role"] = json!("unknown"),
            _ => unreachable!(),
        }
        let e = plan_source(&bytes(&v)).unwrap_err();
        assert!(matches!(e.exit_code(), 2 | 4));
        assert!(!format!("{e:?} {e}").contains("CANARY"));
        assert!(std::error::Error::source(&e).is_none());
    }
    for prefix in [
        r#"{"kind":"CANARY","kind":"quent_review_plan_source","#,
        r#"{"\u006bind":"CANARY","kind":"quent_review_plan_source","#,
    ] {
        let rest = std::str::from_utf8(SOURCE)
            .unwrap()
            .trim()
            .trim_start_matches('{');
        assert!(plan_source(format!("{prefix}{rest}").as_bytes()).is_err());
    }
    for bad in [
        b"\xff".to_vec(),
        vec![b' '; 65537],
        format!("{}0{}", "[".repeat(17), "]".repeat(17)).into_bytes(),
        format!("[{}]", vec!["0"; 257].join(",")).into_bytes(),
        b"{} {}".to_vec(),
    ] {
        assert!(plan_source(&bad).is_err());
    }
    let mut exact = SOURCE.to_vec();
    exact.resize(65536, b' ');
    assert!(plan_source(&exact).is_ok());
    exact.push(b' ');
    assert_eq!(plan_source(&exact), Err(ReviewError::Budget));
}
#[test]
fn c01_c19_spec_and_manifest_independent_domains_remain_unverified() {
    let spec = operation_spec(include_bytes!("../../../fixtures/review-v1/spec.json")).unwrap();
    let expected: Json = serde_json::from_slice(include_bytes!(
        "../../../fixtures/review-v1/spec.expected.json"
    ))
    .unwrap();
    assert_eq!(output(&spec), expected);
    assert_eq!(spec.exit_code(), 3);
    let manifest = include_bytes!("../../../fixtures/manifest-v1/postgres.json");
    let r = quent::review::manifest(manifest).unwrap();
    let o = output(&r);
    assert_eq!(r.exit_code(), 3);
    assert_eq!(o["precision"]["freshness"], "unverified");
    assert_eq!(
        o["manifest"],
        serde_json::from_str::<Json>(
            &quent::manifest::ManifestSource::parse(manifest)
                .unwrap()
                .public_view()
                .to_json()
        )
        .unwrap()
    );
    assert_eq!(o["manifest"]["artifacts"][0]["coverage"], "Complete");
    assert_eq!(o["manifest"]["artifacts"][2]["coverage"], "Unknown");
    for a in o["manifest"]["artifacts"].as_array().unwrap() {
        assert_eq!(a["freshness"], "Unverified");
    }
    let mut partial: Json = serde_json::from_slice(manifest).unwrap();
    partial["schema"]["coverage"] = json!("partial");
    assert_eq!(
        quent::review::manifest(&bytes(&partial)).unwrap().state(),
        State::Partial
    );
    partial["schema"]["tables"][0]["semantics"] = json!("view");
    assert_eq!(
        quent::review::manifest(&bytes(&partial)).unwrap().state(),
        State::Unsupported
    );
}
#[test]
fn c01_c15_application_inspection_keeps_record_and_incomplete_axes() {
    let t = Table::new(vec![Ident::new("CANARY_table").unwrap()], None).unwrap();
    let col = t.column(Ident::new("CANARY_id").unwrap(), SqlType::I64, false);
    let plan = Draft::select(t.clone(), vec![Projection::new(col.clone(), None)])
        .filter(Predicate::Compare(col.clone(), Compare::Eq, Value::I64(7)))
        .limit(1)
        .freeze()
        .finalize(Dialect::Postgres)
        .unwrap();
    let facts = ScopeFacts::application_asserted(
        t,
        Dialect::Postgres,
        vec![col.clone()],
        vec![Key::Primary(vec![col])],
        TableContext::OrdinaryBaseTableWithoutEffects,
    )
    .unwrap();
    let i = plan.inspect(Some(&facts));
    let before = i.sensitive_source().sql().as_ptr();
    let r = inspection(&i);
    assert_eq!(before, i.sensitive_source().sql().as_ptr());
    let o = output(&r);
    assert_eq!(
        o["inspection"],
        serde_json::from_str::<Json>(&i.public_view().to_json()).unwrap()
    );
    assert_eq!(o["risk"], "low");
    assert_eq!(o["coverage"], "complete");
    assert_eq!(r.exit_code(), 3);
    assert_eq!(o["precision"]["policy"], "unverified");
    assert!(!r.to_json().contains("CANARY"));
}

#[test]
fn c24_every_root_and_operation_field_is_required_closed_and_nonnull() {
    let original = source();
    for key in ["kind", "version", "dialect", "operation"] {
        let mut v = original.clone();
        v.as_object_mut().unwrap().remove(key);
        assert!(plan_source(&bytes(&v)).is_err());
        let mut v = original.clone();
        v[key] = Json::Null;
        assert!(plan_source(&bytes(&v)).is_err());
    }
    for key in [
        "family",
        "table",
        "columns",
        "projection",
        "writes",
        "order",
    ] {
        let mut v = original.clone();
        v["operation"].as_object_mut().unwrap().remove(key);
        assert!(plan_source(&bytes(&v)).is_err());
        let mut v = original.clone();
        v["operation"][key] = Json::Null;
        assert!(plan_source(&bytes(&v)).is_err());
    }
    for claim in [
        json!({"sql":null}),
        json!({"args":null}),
        json!({"metadata":{"family":"select","argument_count":null}}),
        json!({"freshness":"CANARY"}),
        json!({"args":[{"type":"i64","value":1.0}]}),
        json!({"sql":"X".repeat(4097)}),
    ] {
        let mut v = original.clone();
        v["claims"] = claim;
        assert!(plan_source(&bytes(&v)).is_err());
    }
    let mut v = original.clone();
    v["operation"]["columns"][0]["type"] = json!("u64");
    assert_eq!(plan_source(&bytes(&v)), Err(ReviewError::Unsupported));
    let d = ReviewError::Invalid.public_view();
    assert_eq!(
        d.to_json(),
        r#"{"kind":"quent_review_diagnostic","version":1,"state":"refused","code":"invalid","action":"check_supported_input"}"#
    );
}
#[cfg(feature = "output-tracing")]
#[path = "output_capture/mod.rs"]
mod output_capture;
#[cfg(feature = "output-tracing")]
#[test]
fn c01_c15_zero_unsolicited_tracing_has_positive_control() {
    let capture = output_capture::Capture::default();
    tracing::subscriber::with_default(capture.clone(), || {
        let _ = plan_source(SOURCE).unwrap().to_json();
        let _ = operation_spec(include_bytes!("../../../fixtures/review-v1/spec.json"))
            .unwrap()
            .to_pretty();
        let _ = quent::review::manifest(include_bytes!(
            "../../../fixtures/manifest-v1/postgres.json"
        ))
        .unwrap()
        .to_json();
        let _ = plan_source(b"CANARY").unwrap_err().public_view().to_json();
    });
    assert!(capture.0.lock().unwrap().is_empty());
    tracing::subscriber::with_default(capture.clone(), || tracing::info!("detecting_control"));
    assert_eq!(capture.0.lock().unwrap().len(), 1);
}
