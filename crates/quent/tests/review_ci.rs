#![cfg(feature = "review-ci")]
use quent::review_ci::*;
use serde_json::{Value, json};
const EMPTY: &[u8] = include_bytes!("../../../fixtures/review-ci-v1/empty.json");
const BASELINE: &[u8] = include_bytes!("../../../fixtures/review-ci-v1/baseline.json");
fn bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}
fn input(source: &str, format: &str) -> Value {
    json!({"target":"CANARY_target https://private.invalid/a","context":"CANARY_context","scope":"CANARY_scope [secret](https://private.invalid)","format":format,"source":source})
}
fn plan() -> Value {
    let mut p: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/review-v1/select.json")).unwrap();
    p["claims"] = json!({"fingerprint":"CANARY_same_hash","sql":"https://CANARY.invalid/\u{1b}[31m [CANARY](file:///secret)","verdict":"accepted","coverage":"complete","freshness":"fresh"});
    p
}
fn set(inputs: Vec<Value>) -> Value {
    json!({"kind":"quent_review_inputs","version":1,"analysis_version":1,"selection":"explicit_set","inputs":inputs})
}
fn parse(v: &Value) -> InputSet {
    InputSet::parse(&bytes(v)).unwrap()
}
fn report(head: &Value, base: &Value, baseline: &Value, now: Option<u64>) -> Report {
    compare(
        parse(head),
        parse(base),
        &Baseline::parse(&bytes(baseline)).unwrap(),
        now.map(|t| Time::unix_seconds(t).unwrap()),
    )
    .unwrap()
}
fn empty_baseline() -> Value {
    serde_json::from_slice(BASELINE).unwrap()
}
fn baseline(i: Value) -> Value {
    json!({"kind":"quent_review_baseline","version":1,"analysis_version":1,"entries":[{"code":"untrusted_claims_present","input":i,"reason":"CANARY_reason [x](https://private.invalid)","owner":"CANARY_owner","expires":200}]})
}
#[test]
fn c01_c15_independent_goldens_and_deterministic_complete_head_non_success() {
    let r = compare(
        InputSet::parse(EMPTY).unwrap(),
        InputSet::parse(EMPTY).unwrap(),
        &Baseline::parse(BASELINE).unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(
        r.to_json() + "\n",
        include_str!("../../../fixtures/review-ci-v1/empty.expected.json")
    );
    assert_eq!(
        r.to_pretty() + "\n",
        include_str!("../../../fixtures/review-ci-v1/empty.pretty.expected.json")
    );
    assert_eq!(
        r.to_sarif() + "\n",
        include_str!("../../../fixtures/review-ci-v1/empty.sarif.expected.json")
    );
    let i = input(&plan().to_string(), "plan");
    let h = set(vec![i.clone()]);
    let r = report(&h, &set(vec![]), &baseline(i), Some(199));
    for _ in 0..8 {
        assert_eq!(
            r.to_json(),
            report(
                &h,
                &set(vec![]),
                &baseline(input(&plan().to_string(), "plan")),
                Some(199)
            )
            .to_json()
        );
    }
    assert_eq!(r.exit_code(), 3);
    let v: Value = serde_json::from_str(&r.to_json()).unwrap();
    assert_eq!(v["git_head"], "unverified");
    assert_eq!(v["runtime"], "unverified");
    // Full pre-existing review output is retained, independently of hidden notice.
    let expected = quent::review::plan_source(plan().to_string().as_bytes()).unwrap();
    assert_eq!(
        v["head_results"][0],
        serde_json::from_str::<Value>(&expected.to_json()).unwrap()
    );
    assert!(
        r.diagnostics()
            .iter()
            .any(|d| d.code() == Code::UntrustedClaimsPresent && d.hidden())
    );
    for stream in [
        r.to_json(),
        r.to_pretty(),
        r.to_sarif(),
        format!("{r:?}"),
        r.to_string(),
        format!("{:?}", parse(&h)),
        format!(
            "{:?}",
            Baseline::parse(&bytes(&baseline(h["inputs"][0].clone()))).unwrap()
        ),
    ] {
        for secret in [
            "CANARY",
            "private.invalid",
            "file://",
            "https://",
            "[31m",
            "fingerprint",
            "owner",
            "reason",
            "SELECT",
            "snippet",
        ] {
            assert!(!stream.contains(secret), "{secret}");
        }
    }
    let sarif: Value = serde_json::from_str(&r.to_sarif()).unwrap();
    assert_eq!(sarif.as_object().unwrap().len(), 2);
    assert_eq!(sarif["version"], "2.1.0");
    let run = &sarif["runs"][0];
    assert_eq!(run.as_object().unwrap().len(), 2);
    assert_eq!(run["tool"], json!({"driver":{"name":"Quent"}}));
    for result in run["results"].as_array().unwrap() {
        assert_eq!(result.as_object().unwrap().len(), 3);
        assert_eq!(result["message"].as_object().unwrap().len(), 1);
        assert!(result["message"]["text"].as_str().unwrap().is_ascii());
        assert!(result["ruleId"].is_string());
        assert!(result["level"].is_string());
    }
}
#[test]
fn c12_exact_diff_content_context_scope_and_duplicate_multiplicity() {
    let a = input(&plan().to_string(), "plan");
    let b = {
        let mut p = plan();
        p["operation"]["table"] = json!(["different_CANARY"]);
        input(&p.to_string(), "plan")
    };
    // Same public output and same claimed hash, different exact operation target.
    assert_eq!(
        quent::review::plan_source(a["source"].as_str().unwrap().as_bytes())
            .unwrap()
            .to_json(),
        quent::review::plan_source(b["source"].as_str().unwrap().as_bytes())
            .unwrap()
            .to_json()
    );
    let r = report(
        &set(vec![a.clone()]),
        &set(vec![a.clone()]),
        &empty_baseline(),
        None,
    );
    assert!(r.diagnostics().iter().all(|d| d.diff() == Diff::Unchanged));
    for changed in [
        b,
        {
            let mut x = a.clone();
            x["context"] = json!("other");
            x
        },
        {
            let mut x = a.clone();
            x["scope"] = json!("other");
            x
        },
        {
            let mut x = a.clone();
            x["target"] = json!("other");
            x
        },
    ] {
        let r = report(
            &set(vec![changed.clone()]),
            &set(vec![a.clone()]),
            &baseline(a.clone()),
            Some(199),
        );
        assert!(r.diagnostics().iter().any(|d| d.diff() == Diff::Added));
        assert!(r.diagnostics().iter().any(|d| d.diff() == Diff::Removed));
        assert!(r.diagnostics().iter().all(|d| !d.hidden()));
    }
    let r = report(
        &set(vec![a.clone(), a.clone()]),
        &set(vec![a.clone()]),
        &baseline(a.clone()),
        Some(199),
    );
    assert!(r.diagnostics().iter().all(|d| d.diff() == Diff::Unknown));
    assert!(r.diagnostics().iter().all(|d| !d.hidden()));
    let mut c = a.clone();
    c["scope"] = json!("second operation");
    let r = report(
        &set(vec![c.clone(), a.clone()]),
        &set(vec![a.clone(), c]),
        &empty_baseline(),
        None,
    );
    assert!(r.diagnostics().iter().all(|d| d.diff() == Diff::Unchanged));
    let r = report(&set(vec![]), &set(vec![a]), &empty_baseline(), None);
    assert!(r.diagnostics().iter().all(|d| d.diff() == Diff::Removed));
    assert_ne!(r.exit_code(), 0);
}
#[test]
fn c02_c08_c19_expiry_and_non_suppressible_register() {
    let i = input(&plan().to_string(), "plan");
    let b = baseline(i.clone());
    let h = set(vec![i.clone()]);
    for (now, hidden) in [
        (None, false),
        (Some(198), true),
        (Some(199), true),
        (Some(200), false),
        (Some(201), false),
    ] {
        let r = report(&h, &set(vec![]), &b, now);
        assert_eq!(
            r.diagnostics().iter().filter(|d| d.hidden()).count(),
            usize::from(hidden)
        );
        assert_ne!(r.exit_code(), 0);
    }
    let all = [
        Code::ReviewIncomplete,
        Code::StructuralBlock,
        Code::Unsupported,
        Code::InputBudget,
        Code::MissingWhere,
        Code::UnknownProof,
        Code::BulkWrite,
        Code::WeakPredicate,
        Code::ReadWithoutLimit,
        Code::ReadProjection,
        Code::InsertUnverified,
        Code::IdentityUnverified,
        Code::ContextUnverified,
        Code::NoCandidates,
        Code::MacroUnexpanded,
        Code::AttributeUnevaluated,
        Code::NamesUnresolved,
        Code::ControlFlowUnresolved,
        Code::ReceiverUnresolved,
        Code::ArgumentsUnverified,
        Code::UnsupportedSyntax,
    ];
    for code in all {
        assert!(!code.suppressible());
        let mut bad = b.clone();
        bad["entries"][0]["code"] = serde_json::to_value(code).unwrap();
        assert!(Baseline::parse(&bytes(&bad)).is_err());
    }
    for code in [
        "policy_tenant_mismatch",
        "policy_tenant_predicate",
        "execution_expired",
        "freshness_failed",
        "unknown",
        "runtime",
        "read_without_limit",
    ] {
        let mut bad = b.clone();
        bad["entries"][0]["code"] = json!(code);
        assert!(Baseline::parse(&bytes(&bad)).is_err());
    }
    let mut p = plan();
    p["operation"]["family"] = json!("delete");
    p["operation"]["projection"] = json!([]);
    p["operation"].as_object_mut().unwrap().remove("filter");
    p["operation"].as_object_mut().unwrap().remove("limit");
    let blocked = input(&p.to_string(), "plan");
    let r = report(
        &set(vec![blocked.clone()]),
        &set(vec![]),
        &baseline(blocked),
        Some(199),
    );
    assert_eq!(r.exit_code(), 5);
    assert_eq!(r.diagnostics().iter().filter(|d| d.hidden()).count(), 1);
    assert!(
        r.diagnostics()
            .iter()
            .any(|d| d.code() == Code::MissingWhere && !d.hidden())
    );
    let mut b = b;
    b["entries"][0]["expires"] = json!(253402300799u64);
    assert!(
        report(&h, &set(vec![]), &b, Some(1))
            .diagnostics()
            .iter()
            .all(|d| !d.hidden())
    );
    assert!(Time::unix_seconds(u64::MAX).is_err());
}
#[test]
fn c24_closed_bounded_wires_and_authority_claim_refusal() {
    let i = input(&plan().to_string(), "plan");
    let s = set(vec![i.clone()]);
    let b = baseline(i);
    for root in [&s, &b] {
        for field in root.as_object().unwrap().keys() {
            let mut v = root.clone();
            v.as_object_mut().unwrap().remove(field);
            reject(&v);
            let mut v = root.clone();
            v[field] = Value::Null;
            reject(&v);
        }
        for (key, value) in [
            ("kind", json!("quent_review_ci")),
            ("version", json!(2)),
            ("analysis_version", json!(2)),
            ("complete", json!(true)),
            ("now", json!(1)),
            ("allow", json!(true)),
            ("head", json!("complete")),
        ] {
            let mut v = root.clone();
            v[key] = value;
            reject(&v);
        }
        let raw = bytes(root);
        let mut oversized = raw.clone();
        oversized.resize(65537, b' ');
        assert!(InputSet::parse(&oversized).is_err());
        assert!(Baseline::parse(&oversized).is_err());
        let dup =
            String::from_utf8(raw)
                .unwrap()
                .replacen("{", "{\"k\\u0069nd\":\"duplicate\",", 1);
        assert!(InputSet::parse(dup.as_bytes()).is_err());
        assert!(Baseline::parse(dup.as_bytes()).is_err());
    }
    for field in ["target", "context", "scope", "format", "source"] {
        for value in [
            Value::Null,
            json!(""),
            json!("*"),
            json!(" "),
            json!("CANARY\n"),
            json!("x".repeat(4097)),
        ] {
            let mut v = s.clone();
            v["inputs"][0][field] = value;
            assert!(InputSet::parse(&bytes(&v)).is_err());
        }
        let mut v = s.clone();
        v["inputs"][0].as_object_mut().unwrap().remove(field);
        assert!(InputSet::parse(&bytes(&v)).is_err());
    }
    for field in ["code", "input", "reason", "owner", "expires"] {
        let mut v = b.clone();
        v["entries"][0].as_object_mut().unwrap().remove(field);
        assert!(Baseline::parse(&bytes(&v)).is_err());
        let mut v = b.clone();
        v["entries"][0][field] = Value::Null;
        assert!(Baseline::parse(&bytes(&v)).is_err());
    }
    for (field, values) in [
        (
            "reason",
            vec![json!(""), json!(" "), json!("x".repeat(257)), json!("\n")],
        ),
        (
            "owner",
            vec![
                json!(""),
                json!(" "),
                json!("x".repeat(129)),
                json!("\u{1b}"),
            ],
        ),
        (
            "expires",
            vec![
                json!(0),
                json!(-1),
                json!(253402300800u64),
                json!("forever"),
                json!("2026-10-26"),
                json!(1.5),
            ],
        ),
    ] {
        for value in values {
            let mut v = b.clone();
            v["entries"][0][field] = value;
            assert!(Baseline::parse(&bytes(&v)).is_err());
        }
    }
    let mut v = b.clone();
    v["entries"] = json!([b["entries"][0], b["entries"][0]]);
    assert!(Baseline::parse(&bytes(&v)).is_err());
    for raw in [
        vec![255],
        b"null".to_vec(),
        b"{} {}".to_vec(),
        format!("{}0{}", "[".repeat(17), "]".repeat(17)).into_bytes(),
        br#"{"n":18446744073709551616}"#.to_vec(),
    ] {
        assert!(InputSet::parse(&raw).is_err());
        assert!(Baseline::parse(&raw).is_err());
    }
    for count in [9, 257] {
        let v = set(vec![s["inputs"][0].clone(); count]);
        assert!(InputSet::parse(&bytes(&v)).is_err());
    }
    fn reject(v: &Value) {
        assert!(InputSet::parse(&bytes(v)).is_err());
        assert!(Baseline::parse(&bytes(v)).is_err());
    }
}
#[cfg(feature = "source-review")]
#[test]
fn c20_changed_only_unknown_public_budget_and_app_unmatched() {
    let i = input(
        "fn f(){ let x=quent_core::Draft::delete(t); helper(x); }",
        "rust",
    );
    let mut h = set(vec![i.clone()]);
    h["selection"] = json!("changed_only");
    let r = report(&h, &set(vec![]), &empty_baseline(), None);
    assert_eq!(r.exit_code(), 3);
    assert!(
        r.diagnostics()
            .iter()
            .any(|d| d.code() == Code::IdentityUnverified)
    );
    let r = report(
        &set(vec![i.clone(); 8]),
        &set(vec![i; 8]),
        &empty_baseline(),
        None,
    );
    assert_eq!(r.exit_code(), 2);
    assert!(r.diagnostics().len() > PUBLIC_DIAGNOSTICS);
    let v: Value = serde_json::from_str(&r.to_json()).unwrap();
    assert_eq!(v["public_budget_exceeded"], true);
    assert_eq!(v["head_results"], json!([]));
    assert!(r.to_sarif().contains("input_budget"));
    use quent_core::{Dialect, Draft, Ident, Table};
    let t = Table::new(vec![Ident::new("CANARY").unwrap()], None).unwrap();
    let p = Draft::delete(t)
        .freeze()
        .finalize(Dialect::Postgres)
        .unwrap();
    let inspection = p.inspect(None);
    let r = compare(
        InputSet::inspections(&[&inspection]).unwrap(),
        InputSet::inspections(&[&inspection]).unwrap(),
        &Baseline::empty(),
        None,
    )
    .unwrap();
    assert_eq!(r.exit_code(), 5);
    assert!(
        r.diagnostics()
            .iter()
            .all(|d| d.diff() == Diff::Unknown && !d.hidden())
    );
}

#[cfg(feature = "output-tracing")]
#[path = "output_capture/mod.rs"]
mod output_capture;
#[cfg(feature = "output-tracing")]
#[test]
fn c01_no_implicit_tracing_with_detecting_control() {
    let capture = output_capture::Capture::default();
    tracing::subscriber::with_default(capture.clone(), || {
        let r = report(
            &set(vec![input(&plan().to_string(), "plan")]),
            &set(vec![]),
            &empty_baseline(),
            None,
        );
        let _ = r.to_json();
        let _ = r.to_pretty();
        let _ = r.to_sarif();
        let _ = InputSet::parse(b"CANARY").unwrap_err();
    });
    assert!(capture.0.lock().unwrap().is_empty());
    tracing::subscriber::with_default(capture.clone(), || tracing::info!("detecting_control"));
    assert_eq!(capture.0.lock().unwrap().len(), 1);
}
