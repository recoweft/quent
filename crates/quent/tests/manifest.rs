#![cfg(feature = "manifest")]
use quent::manifest::*;
use serde_json::{Value, json};
use std::error::Error;
const PG: &str = include_str!("../../../fixtures/manifest-v1/postgres.json");
const MY: &str = include_str!("../../../fixtures/manifest-v1/mysql.json");
const KINDS: [ArtifactKind; 4] = [
    ArtifactKind::Schema,
    ArtifactKind::Policy,
    ArtifactKind::GeneratedCode,
    ArtifactKind::LiveObservation,
];
fn parsed(v: &Value) -> ManifestSource {
    ManifestSource::parse(&serde_json::to_vec(v).unwrap()).unwrap()
}
fn value(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}
#[track_caller]
fn refuse(bytes: &[u8]) {
    let e = ManifestSource::parse(bytes).unwrap_err();
    assert!(e.source().is_none());
    assert!(e.to_string().starts_with("QUENT_MANIFEST_"));
    assert!(!format!("{e:?} {e}").contains("CANARY"));
}
#[test]
fn independent_domains_and_offline_evidence_both_dialects() {
    for input in [PG, MY] {
        let a = ManifestSource::parse(input.as_bytes()).unwrap();
        assert_eq!(
            a.public_view().artifacts()[0].coverage(),
            SourceCoverage::Complete
        );
        for s in a.public_view().artifacts() {
            assert_eq!(s.freshness(), Freshness::Unverified);
        }
        let c = a.compare_offline(&a);
        assert_eq!(c.correspondence(), &[Correspondence::ExactContent; 4]);
        for s in c.public_view().artifacts() {
            assert_eq!(s.freshness(), Freshness::OfflineConsistent);
        }
        for (i, field) in ["schema", "policy", "generated_code", "live_observation"]
            .iter()
            .enumerate()
        {
            let mut v = value(input);
            v[field]["target"]["address"] = json!("other");
            let b = parsed(&v);
            let result = a.compare_offline(&b);
            for (j, k) in KINDS.iter().enumerate() {
                assert_eq!(
                    result.correspondence()[j],
                    if i == j {
                        Correspondence::Different
                    } else {
                        Correspondence::ExactContent
                    }
                );
                assert_eq!(
                    a.sensitive_source().fingerprint(*k) == b.sensitive_source().fingerprint(*k),
                    i != j
                );
            }
        }
        let mut claim = value(input);
        claim["live_observation"]["generation"] = json!({"state":"claimed","value":42});
        claim["live_observation"]["schema"] = claim["schema"].clone();
        let first = parsed(&claim);
        claim["live_observation"]["schema"]["tables"][0]["columns"][0]["native"] = json!("i32");
        let second = parsed(&claim);
        assert_eq!(
            first.compare_offline(&second).correspondence()[3],
            Correspondence::Different
        );
        assert_eq!(
            first.public_view().artifacts()[3].freshness(),
            Freshness::Unverified
        );
        claim["live_observation"]["schema"]["target"]["address"] = json!("wrong_target");
        refuse(&serde_json::to_vec(&claim).unwrap());
        let mut empty = value(input);
        for f in ["schema", "policy", "generated_code", "live_observation"] {
            empty[f] = Value::Null;
        }
        let b = parsed(&empty);
        assert_eq!(
            a.compare_offline(&b).correspondence(),
            &[Correspondence::MissingRight; 4]
        );
        assert_eq!(
            b.compare_offline(&a).correspondence(),
            &[Correspondence::MissingLeft; 4]
        );
        assert_eq!(
            b.compare_offline(&b).correspondence(),
            &[Correspondence::MissingBoth; 4]
        );
        assert_eq!(
            b.public_view().artifacts()[0].freshness(),
            Freshness::Missing
        );
        assert_eq!(
            a.application_asserted_summary(None, None).artifacts()[0].freshness(),
            Freshness::ApplicationAsserted
        );
        assert_eq!(
            a.application_asserted_summary(Some(10), Some(10))
                .artifacts()[0]
                .freshness(),
            Freshness::Stale
        );
        assert_eq!(
            a.application_asserted_summary(Some(9), Some(10))
                .artifacts()[0]
                .freshness(),
            Freshness::ApplicationAsserted
        );
        // No clocks or authority are manufactured for unknown live fields.
        assert_eq!(
            a.public_view().artifacts()[3].freshness(),
            Freshness::Unverified
        );
        let bytes = a.sensitive_source().canonical_source();
        assert_eq!(
            bytes,
            ManifestSource::parse(&bytes)
                .unwrap()
                .sensitive_source()
                .canonical_source()
        );
    }
}
#[test]
fn canonical_literal_expectation_and_frozen_hash() {
    // Independent literal expected serialization; not regenerated from API output.
    let a = ManifestSource::parse(MY.as_bytes()).unwrap();
    let expected = br#"{"kind":"quent_schema","version":1,"target":{"database":"db","address":"route","catalog":"catalog","dialect":"mysql"},"coverage":"complete","tables":[{"path":["public","items"],"semantics":"ordinary","columns_coverage":"complete","keys_coverage":"complete","columns":[{"name":"id","native":"i64","nullability":"nonnull","default":{"state":"none"}},{"name":"tenant_id","native":"i64","nullability":"nonnull","default":{"state":"literal","value":{"type":"i64","value":9223372036854775807}}}],"keys":[{"kind":"primary","columns":["tenant_id","id"],"coverage":"complete"}]}]}"#;
    assert_eq!(
        a.sensitive_source()
            .canonical_artifact(ArtifactKind::Schema)
            .unwrap(),
        expected
    );
    // Object order/whitespace are insignificant, sequence order is significant.
    assert_eq!(
        a.compare_offline(&parsed(&value(MY))).correspondence(),
        &[Correspondence::ExactContent; 4]
    );
    let fp = a
        .sensitive_source()
        .fingerprint(ArtifactKind::Schema)
        .unwrap();
    assert_eq!(fp.sensitive_hex(), "ab6e522f683d3ea5");
}
#[test]
fn schema_semantic_differences_are_preserved() {
    for input in [PG, MY] {
        let a = ManifestSource::parse(input.as_bytes()).unwrap();
        for change in 0..12 {
            let mut v = value(input);
            let t = &mut v["schema"]["tables"][0];
            match change {
                0 => t["columns"].as_array_mut().unwrap().swap(0, 1),
                1 => t["keys"][0]["columns"].as_array_mut().unwrap().swap(0, 1),
                2 => t["columns"][0]["native"] = json!("i32"),
                3 => t["columns"][0]["nullability"] = json!("unknown"),
                4 => {
                    t["columns"][0].as_object_mut().unwrap().remove("default");
                }
                5 => t["columns"][0]["default"] = json!({"state":"unknown"}),
                6 => {
                    t["columns"][0]["default"] = json!({"state":"literal","value":{"type":"null"}})
                }
                7 => {
                    t["columns"][0]["default"] =
                        json!({"state":"unsupported_expression","expression":"CANARY()"})
                }
                8 => t["keys_coverage"] = json!("unknown"),
                9 => t["semantics"] = json!("view"),
                10 => t["columns"][0]["native"] = json!("u64"),
                _ => t["keys"][0]["coverage"] = json!("partial"),
            }
            let b = parsed(&v);
            assert_eq!(
                a.compare_offline(&b).correspondence()[0],
                Correspondence::Different,
                "change {change}"
            );
            assert_ne!(
                a.sensitive_source()
                    .canonical_artifact(ArtifactKind::Schema),
                b.sensitive_source()
                    .canonical_artifact(ArtifactKind::Schema)
            );
            if [3, 4, 5, 8].contains(&change) {
                assert_ne!(
                    b.public_view().artifacts()[0].coverage(),
                    SourceCoverage::Complete
                );
            }
            assert_eq!(
                b.public_view().artifacts()[0].freshness(),
                Freshness::Unverified
            );
        }
        for state in ["nullable", "unknown"] {
            let mut v = value(input);
            v["schema"]["tables"][0]["columns"][0]["nullability"] = json!(state);
            v["schema"]["tables"][0]["keys"][0]["kind"] = json!("unique");
            assert_ne!(
                parsed(&v).public_view().artifacts()[0].coverage(),
                SourceCoverage::Complete
            );
        }
    }
}
#[test]
fn exact_integer_width_sign_null_and_escaping() {
    for input in [PG, MY] {
        for (ty, lo, hi) in [
            ("i8", "-128", "127"),
            ("i16", "-32768", "32767"),
            ("i32", "-2147483648", "2147483647"),
            ("i64", "-9223372036854775808", "9223372036854775807"),
            ("u8", "0", "255"),
            ("u16", "0", "65535"),
            ("u32", "0", "4294967295"),
            ("u64", "0", "18446744073709551615"),
        ] {
            for n in [lo, hi] {
                let mut v = value(input);
                v["schema"]["tables"][0]["columns"][0]["native"] = json!(ty);
                v["schema"]["tables"][0]["columns"][0]["default"] = value(&format!(
                    r#"{{"state":"literal","value":{{"type":"{ty}","value":{n}}}}}"#
                ));
                let s = parsed(&v);
                let b = s
                    .sensitive_source()
                    .canonical_artifact(ArtifactKind::Schema)
                    .unwrap();
                assert!(String::from_utf8(b).unwrap().contains(n));
            }
        }
        for (ty, literal) in [
            ("bool", json!({"type":"bool","value":false})),
            ("bool", json!({"type":"bool","value":true})),
            ("text", json!({"type":"text","value":""})),
            ("text", json!({"type":"text","value":"CANARY\"\\\n日本語"})),
        ] {
            let mut v = value(input);
            v["schema"]["tables"][0]["columns"][0]["native"] = json!(ty);
            v["schema"]["tables"][0]["columns"][0]["default"] =
                json!({"state":"literal","value":literal});
            let a = parsed(&v);
            let b = a.sensitive_source().canonical_source();
            assert_eq!(
                a.compare_offline(&ManifestSource::parse(&b).unwrap())
                    .correspondence(),
                &[Correspondence::ExactContent; 4]
            );
        }
        let mut v = value(input);
        v["schema"]["tables"][0]["columns"][0]["name"] = json!("a\"`");
        v["schema"]["tables"][0]["keys"][0]["columns"][1] = json!("a\"`");
        v["generated_code"]["rust_source"] = json!("\"\\\n\t\u{0000}日本語");
        let s = parsed(&v);
        let b = s.sensitive_source().canonical_source();
        assert_eq!(
            s.compare_offline(&ManifestSource::parse(&b).unwrap())
                .correspondence(),
            &[Correspondence::ExactContent; 4]
        );
    }
}
#[test]
fn strict_wire_refusals() {
    for input in [PG, MY] {
        for field in [
            "fresh",
            "tenant",
            "subject",
            "allow",
            "reason",
            "generated_verified",
            "fingerprint",
            "metadata",
        ] {
            let mut v = value(input);
            v[field] = json!("CANARY");
            refuse(&serde_json::to_vec(&v).unwrap());
        }
        for pointer in [
            "/schema",
            "/schema/target",
            "/schema/tables/0",
            "/schema/tables/0/columns/0",
            "/schema/tables/0/keys/0",
            "/policy",
            "/policy/rules/0",
            "/generated_code",
            "/live_observation",
            "/live_observation/observed_at",
        ] {
            let mut v = value(input);
            v.pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("CANARY_UNKNOWN".into(), json!(true));
            refuse(&serde_json::to_vec(&v).unwrap());
        }
        for (ty, n) in [
            ("i16", "32768"),
            ("i32", "2147483648"),
            ("i64", "9223372036854775808"),
            ("u8", "256"),
            ("u16", "65536"),
            ("u32", "4294967296"),
            ("u64", "18446744073709551616"),
            ("u64", "-1"),
        ] {
            let text = input.replace(
                "\"type\":\"i64\",\"value\":9223372036854775807",
                &format!("\"type\":\"{ty}\",\"value\":{n}"),
            );
            let text = text.replace(
                "\"name\":\"tenant_id\",\"native\":\"i64\"",
                &format!("\"name\":\"tenant_id\",\"native\":\"{ty}\""),
            );
            refuse(text.as_bytes());
        }
        for field in ["kind", "version"] {
            let mut v = value(input);
            v.as_object_mut().unwrap().remove(field);
            refuse(&serde_json::to_vec(&v).unwrap());
        }
        for pointer in [
            "/version",
            "/schema/version",
            "/policy/version",
            "/generated_code/version",
            "/live_observation/version",
        ] {
            let mut v = value(input);
            *v.pointer_mut(pointer).unwrap() = json!(2);
            refuse(&serde_json::to_vec(&v).unwrap());
        }
        for (pointer, new) in [
            ("/schema/tables/0/columns/0/default", Value::Null),
            ("/schema/tables/0/columns/0/native", Value::Null),
            ("/schema/tables/0/columns/0/nullability", json!(false)),
            ("/schema/tables/0/keys/0/columns", json!([])),
            (
                "/schema/tables/0/columns/0/default",
                json!({"state":"none","CANARY":"ignored?"}),
            ),
        ] {
            let mut v = value(input);
            *v.pointer_mut(pointer).unwrap() = new;
            refuse(&serde_json::to_vec(&v).unwrap());
        }
        refuse(
            input
                .replace("\"version\":1", "\"version\":1,\"version\":1")
                .as_bytes(),
        );
        // Duplicate nested keys, including escaped equivalent spelling.
        refuse(
            input
                .replace("\"name\":\"id\"", "\"name\":\"id\",\"na\\u006de\":\"id\"")
                .as_bytes(),
        );
        refuse(
            input
                .replace(
                    "\"state\":\"none\"",
                    "\"state\":\"none\",\"state\":\"none\"",
                )
                .as_bytes(),
        );
        for n in ["128", "-129", "18446744073709551616", "1.0", "1e2"] {
            let text = input.replace(
                "\"type\":\"i64\",\"value\":9223372036854775807",
                &format!("\"type\":\"i8\",\"value\":{n}"),
            );
            let text = text.replace(
                "\"name\":\"tenant_id\",\"native\":\"i64\"",
                "\"name\":\"tenant_id\",\"native\":\"i8\"",
            );
            refuse(text.as_bytes());
        }
        refuse(format!("{input} {{}}").as_bytes());
        let mut bad = input.as_bytes().to_vec();
        bad[5] = 255;
        refuse(&bad);
        let mut v = value(input);
        v["schema"]["tables"][0]["columns"][0]
            .as_object_mut()
            .unwrap()
            .remove("native");
        refuse(&serde_json::to_vec(&v).unwrap());
        let mut v = value(input);
        v["schema"]["tables"][0]["columns"] = json!(vec![json!({"name":"id"}); 257]);
        refuse(&serde_json::to_vec(&v).unwrap());
    }
    refuse(&vec![b' '; 65537]);
    refuse(format!("{}0{}", "[".repeat(18), "]".repeat(18)).as_bytes());
    refuse(format!(r#"{{"CANARY":"{}"}}"#, "x".repeat(4097)).as_bytes());
    refuse(format!("[{}]", vec!["0"; 1025].join(",")).as_bytes());
}
#[test]
fn public_canaries_and_live_claim_nonpromotion() {
    for input in [PG, MY] {
        let mut v = value(input);
        for f in ["schema", "policy", "generated_code", "live_observation"] {
            v[f]["target"]["database"] = json!("CANARY_DATABASE");
            v[f]["target"]["address"] = json!("/CANARY/path");
            v[f]["target"]["catalog"] = json!("CANARY_CATALOG");
        }
        v["schema"]["tables"][0]["columns"][0]["default"] = json!({"state":"unsupported_expression","expression":"CANARY_SQL_DEFAULT('CANARY_VALUE')"});
        v["schema"]["tables"][0]["path"] = json!(["public", "CANARY_TABLE"]);
        v["schema"]["tables"][0]["columns"][0]["name"] = json!("CANARY_COLUMN");
        v["schema"]["tables"][0]["keys"][0]["columns"][1] = json!("CANARY_COLUMN");
        v["policy"]["rules"][0]["column"] = json!("CANARY_POLICY_COLUMN");
        v["policy"]["rules"][0]["table"] = json!(["public", "CANARY_POLICY_TABLE"]);
        v["generated_code"]["rust_source"] = json!("CANARY_CODE");
        v["generated_code"]["coverage"] = json!("complete");
        v["live_observation"]["coverage"] = json!("complete");
        v["live_observation"]["observed_at"] = json!({"state":"claimed","value":1});
        v["live_observation"]["expires_at"] = json!({"state":"claimed","value":9999999999u64});
        v["live_observation"]["generation"] = json!({"state":"claimed","value":42});
        let s = parsed(&v);
        let p = s.public_view();
        assert_eq!(p.artifacts()[3].freshness(), Freshness::Unverified);
        let public = format!(
            "{s:?} {:?} {:?} {p:?} {p} {:?}",
            s.sensitive_source(),
            s.sensitive_source().fingerprint(ArtifactKind::Schema),
            s.compare_offline(&s)
        );
        assert!(!public.contains("CANARY"));
        assert!(!public.contains("9999999999"));
        assert!(ManifestSource::parse(p.to_json().as_bytes()).is_err());
        assert!(!p.to_json().contains("fingerprint"));
        // App assertion and offline equality labels never create live evidence.
        assert_eq!(
            s.compare_offline(&s).public_view().artifacts()[3].freshness(),
            Freshness::OfflineConsistent
        );
    }
}

#[cfg(feature = "output-tracing")]
#[path = "output_capture/mod.rs"]
mod output_capture;
#[cfg(feature = "output-tracing")]
#[test]
fn parser_comparison_errors_emit_no_tracing() {
    let capture = output_capture::Capture::default();
    tracing::subscriber::with_default(capture.clone(), || {
        let a = ManifestSource::parse(PG.as_bytes()).unwrap();
        let _ = a.compare_offline(&a).public_view().to_json();
        refuse(br#"{"CANARY_SOURCE_ERROR":false}"#);
    });
    assert!(capture.0.lock().unwrap().is_empty());
}
#[test]
fn literal_null_and_unknown_state_extra_fields_refuse() {
    for input in [PG, MY] {
        for d in [
            json!({"state":"unknown","CANARY":false}),
            json!({"state":"literal","value":{"type":"null","value":"CANARY"}}),
        ] {
            let mut v = value(input);
            v["schema"]["tables"][0]["columns"][0]["default"] = d;
            refuse(&serde_json::to_vec(&v).unwrap());
        }
        let mut v = value(input);
        v["live_observation"]["observed_at"] = json!({"state":"unknown","CANARY":1});
        refuse(&serde_json::to_vec(&v).unwrap());
    }
}
#[test]
fn frozen_independent_full_source_and_all_domain_hashes() {
    for (input, expected) in [
        (
            PG,
            [
                "3dbfe94e134b2124",
                "0bb72bfdf52a6d4f",
                "8ea2ba78cfc13dd6",
                "49049d5f90e42fe6",
            ],
        ),
        (
            MY,
            [
                "ab6e522f683d3ea5",
                "8e5b2f3f45261382",
                "f1e4f0a65645b5d5",
                "a3282c91aa4871eb",
            ],
        ),
    ] {
        let s = ManifestSource::parse(input.as_bytes()).unwrap();
        assert_eq!(
            s.sensitive_source().canonical_source(),
            input.trim().as_bytes()
        );
        for (k, h) in KINDS.into_iter().zip(expected) {
            assert_eq!(
                s.sensitive_source().fingerprint(k).unwrap().sensitive_hex(),
                h
            );
        }
    }
}
#[test]
fn typed_budget_boundaries_and_missing_are_not_complete() {
    for input in [PG, MY] {
        let mut v = value(input);
        let t = &mut v["schema"]["tables"][0];
        t["columns"] = json!((0..256).map(|n| json!({"name":format!("c{n}"),"native":"i64","nullability":"nonnull","default":{"state":"none"}})).collect::<Vec<_>>());
        t["keys"] = json!([]);
        let _ = parsed(&v);
        v["schema"]["tables"][0]["columns"].as_array_mut().unwrap().push(json!({"name":"c256","native":"i64","nullability":"nonnull","default":{"state":"none"}}));
        assert_eq!(
            ManifestSource::parse(&serde_json::to_vec(&v).unwrap()).unwrap_err(),
            ManifestError::Budget
        );
        let mut v = value(input);
        let t = v["schema"]["tables"][0].clone();
        v["schema"]["tables"] = json!(
            (0..17)
                .map(|n| {
                    let mut t = t.clone();
                    t["path"] = json!(["public", format!("t{n}")]);
                    t
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(
            ManifestSource::parse(&serde_json::to_vec(&v).unwrap()).unwrap_err(),
            ManifestError::Budget
        );
        let mut v = value(input);
        let c = &mut v["schema"]["tables"][0]["columns"][0];
        c["nullability"] = json!("unknown");
        c["default"] = json!({"state":"unknown"});
        v["schema"]["tables"][0]["keys"] = json!([]);
        let a = parsed(&v);
        assert_eq!(
            a.public_view().artifacts()[0].coverage(),
            SourceCoverage::Unknown
        );
        assert_eq!(
            a.compare_offline(&a).public_view().artifacts()[0].coverage(),
            SourceCoverage::Unknown
        );
    }
}
