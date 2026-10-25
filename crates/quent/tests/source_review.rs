#![cfg(feature = "source-review")]
use quent::source_review::{Error, review};
use serde_json::{Value, json};
fn view(s: &str) -> Value {
    serde_json::from_str(&review(s.as_bytes()).unwrap().to_json()).unwrap()
}
#[test]
fn c20_independent_candidates_precision_and_no_safe_where_claim() {
    for (source, expected) in [
        (
            include_str!("../../../fixtures/source-review-v1/delete.rs"),
            include_str!("../../../fixtures/source-review-v1/delete.expected.json"),
        ),
        (
            include_str!("../../../fixtures/source-review-v1/filter.rs"),
            include_str!("../../../fixtures/source-review-v1/filter.expected.json"),
        ),
    ] {
        let r = review(source.as_bytes()).unwrap();
        assert_eq!(r.exit_code(), 3);
        assert_eq!(
            view(source),
            serde_json::from_str::<Value>(expected).unwrap()
        );
        assert_eq!(r.to_json(), review(source.as_bytes()).unwrap().to_json());
    }
    for (s, family) in [
        ("fn f(){quent_core::Draft::select(t,p);}", "select"),
        ("fn f(){quent_core::Draft::select(t,p).count();}", "count"),
        ("fn f(){quent_core::Draft::insert(t,d);}", "insert"),
        ("fn f(){quent_core::Draft::update(t,d);}", "update"),
        ("fn f(){::quent_core::Draft::delete(t);}", "delete"),
    ] {
        let o = view(s);
        assert_eq!(o["operations"][0]["family"], family);
        assert_eq!(o["state"], "partial");
        assert_eq!(o["risk"], "unknown");
    }
}
#[test]
fn c20_whole_input_unknowns_survive_candidates_and_nesting() {
    for (prefix, expr, code) in [
        ("", "m!();", "macro_unexpanded"),
        (
            "use quent_core as q;",
            "q::Draft::delete(t);",
            "names_unresolved",
        ),
        ("", "helper();", "receiver_unresolved"),
        ("#[cfg(any())]", "", "attribute_unevaluated"),
        ("#[cfg_attr(any(),derive(X))]", "", "attribute_unevaluated"),
        ("mod quent_core {}", "", "names_unresolved"),
        ("struct Draft;", "", "names_unresolved"),
        ("", "other.delete(t);", "receiver_unresolved"),
        ("", "other_crate::Draft::delete(t);", "receiver_unresolved"),
        ("", "if x {helper();}", "control_flow_unresolved"),
    ] {
        let o = view(&format!(
            "{prefix} fn f(){{quent_core::Draft::delete(t);{expr}}}"
        ));
        assert_eq!(o["state"], "partial");
        assert_eq!(o["coverage"], "partial");
        assert_eq!(o["operations"].as_array().unwrap().len(), 1);
        assert!(
            o["diagnostics"].as_array().unwrap().contains(&json!(code)),
            "{code}"
        );
    }
    for s in [
        "fn f(){if x {quent_core::Draft::delete(t);}}",
        "fn f(){helper(quent_core::Draft::delete(t));}",
        "fn f(){let c=||quent_core::Draft::delete(t);}",
        "fn f(){quent_core::Draft::delete(quent_core::Draft::delete(t));}",
        "fn f(){quent_core::Draft::delete(t).filter(quent_core::Draft::delete(t));}",
    ] {
        let o = view(s);
        assert!(!o["operations"].as_array().unwrap().is_empty());
        assert_eq!(o["coverage"], "partial");
        if s.matches("Draft::delete").count() == 2 {
            assert_eq!(o["operations"].as_array().unwrap().len(), 2);
        }
    }
}
#[test]
fn c20_zero_candidates_fake_source_and_no_capture_claim() {
    for s in [
        "fn f(){let s=\"quent_core::Draft::delete(t)\";}",
        "fn f(){let s=r###\"quent_core::Draft::delete(t)\"###;}",
        "fn f(){let s=br#\"quent_core::Draft::delete(t)\"#;}",
        "fn f(){let s=c\"quent_core::Draft::delete(t)\";}",
        "/* quent_core::Draft::delete(t) /* nested */ */ fn f(){}",
        "// quent_core::Draft::delete(t)\nfn f(){}",
        "m!(quent_core::Draft::delete(t));",
        "#[derive(CANARY)] struct X;",
        "fn f<'a>(s: &'a str){let c='}';}",
        "#[trusted_capture] fn f(){}",
        "fn f(){Draft::delete(t);}",
    ] {
        let r = review(s.as_bytes()).unwrap();
        let o = view(s);
        assert_eq!(r.exit_code(), 4);
        assert_eq!(o["state"], "unsupported");
        assert_eq!(o["operations"], json!([]));
        assert_eq!(o["precision"]["capture"], "not_implemented");
    }
    for s in ["", "   ", "// no candidates", "/* empty */"] {
        let r = review(s.as_bytes()).unwrap();
        assert_eq!(r.exit_code(), 3);
        assert_eq!(view(s)["coverage"], "unknown");
    }
}
#[test]
fn c24_preparser_envelope_covers_every_byte_and_recursion_spelling() {
    for b in [
        vec![b' '; 65537],
        vec![b' '; 4097],
        vec![b'a'; 513],
        format!("fn f(){{{}x;}}", "!".repeat(49)).into_bytes(),
        format!("fn f(){{{}x{};}}", "(".repeat(9), ")".repeat(9)).into_bytes(),
        format!("/*{}*/", " ".repeat(4096)).into_bytes(),
        format!("fn f(){{let x=r#\"{}\"#;}}", " ".repeat(4096)).into_bytes(),
    ] {
        assert_eq!(review(&b).unwrap_err(), Error::Budget);
    }
    assert_eq!(review(&[255]).unwrap_err(), Error::Invalid);
    for s in [
        "fn {",
        "fn f(){let s=\"CANARY;}",
        "/* CANARY",
        "fn f(){let x=r##\"CANARY\"#;}",
        "{\"capture\":\"trusted\"}",
    ] {
        assert_eq!(review(s.as_bytes()).unwrap_err(), Error::Invalid);
    }
    // Budget precedes parsing/UTF8, with no arbitrary parser errors retained.
    assert_eq!(review(&vec![255; 4097]).unwrap_err(), Error::Budget);
}
#[test]
fn c15_all_public_surfaces_redact_source_and_parser_errors() {
    for s in [
        "fn CANARY(){quent_core::Draft::delete(CANARY);}",
        "fn f(){panic!(\"CANARY\");}",
        "fn CANARY(){}",
    ] {
        let r = review(s.as_bytes()).unwrap();
        for o in [r.to_json(), r.to_pretty(), r.to_string(), format!("{r:?}")] {
            assert!(!o.contains("CANARY"));
        }
    }
    for e in [Error::Invalid, Error::Budget] {
        assert!(std::error::Error::source(&e).is_none());
        for o in [
            e.to_string(),
            format!("{e:?}"),
            serde_json::to_string(&e).unwrap(),
        ] {
            assert!(!o.contains("CANARY"));
        }
    }
}
#[cfg(feature = "output-tracing")]
#[path = "output_capture/mod.rs"]
mod output_capture;
#[cfg(feature = "output-tracing")]
#[test]
fn c01_no_implicit_tracing_with_detecting_control() {
    let capture = output_capture::Capture::default();
    tracing::subscriber::with_default(capture.clone(), || {
        review(b"fn f(){quent_core::Draft::delete(t);}").unwrap();
        review(b"CANARY").unwrap_err();
    });
    assert!(capture.0.lock().unwrap().is_empty());
    tracing::subscriber::with_default(capture.clone(), || tracing::info!("detecting_control"));
    assert_eq!(capture.0.lock().unwrap().len(), 1);
}
