//! Actual subprocess client. Synthetic registration is ONLY in this test target;
//! product CLI has no test command, environment switch or embedded model.
mod mcp_fixture;
use quent::mcp::*;
use quent_core::{Dialect, SqlType};
use serde_json::{Value, json};
#[path = "mcp_fixture/client.rs"]
mod client;
use client::*;
fn content(v: &Value) -> Value {
    serde_json::from_str(v["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}
fn public_keys(v: &Value) -> bool {
    v.as_object().is_some_and(|o| {
        o.keys().map(String::as_str).collect::<Vec<_>>()
            == [
                "assumptions",
                "caller_missing_where",
                "completeness",
                "evidence",
                "family",
                "findings",
                "has_limit",
                "kind",
                "order_count",
                "precision",
                "projection",
                "projection_count",
                "returning_count",
                "risk",
                "scope",
                "scope_upper_bound",
                "version",
                "versioned",
                "write_count",
            ]
    })
}
fn independent_public(v: &Value) {
    assert!(public_keys(v));
    assert_eq!(v["kind"], "quent_plan_view");
    assert_eq!(v["version"], 1);
    for (key, values) in [
        ("assumptions", vec!["none", "application_asserted"]),
        ("completeness", vec!["complete", "partial", "unknown"]),
        ("family", vec!["select"]),
        (
            "precision",
            vec!["unevaluated", "conditional_conservative", "unknown"],
        ),
        ("projection", vec!["columns", "count", "none"]),
        (
            "risk",
            vec![
                "unevaluated",
                "low",
                "medium",
                "high",
                "destructive",
                "blocked",
            ],
        ),
        (
            "scope",
            vec![
                "zero",
                "single",
                "finite_bound",
                "broad",
                "unknown",
                "not_applicable",
            ],
        ),
    ] {
        assert!(values.contains(&v[key].as_str().unwrap()));
    }
    for k in [
        "order_count",
        "projection_count",
        "returning_count",
        "write_count",
    ] {
        assert!(v[k].as_u64().is_some_and(|n| n <= 256));
    }
    for k in ["caller_missing_where", "has_limit", "versioned"] {
        assert!(v[k].is_boolean());
    }
    assert!(
        v["scope_upper_bound"].is_null()
            || v["scope_upper_bound"].as_u64().is_some_and(|n| n <= 65535)
    );
    assert!(
        v["evidence"].is_null()
            || [
                "missing_facts",
                "inconsistent_facts",
                "unsupported_context",
                "unsupported_key",
                "analysis_budget",
                "empty_truth_set",
                "complete_key_tuples",
                "unbounded_predicate",
                "unsupported_predicate",
                "no_predicate",
                "insert_shape"
            ]
            .contains(&v["evidence"].as_str().unwrap())
    );
    let fs = v["findings"].as_array().unwrap();
    assert!(fs.len() <= 7);
    for f in fs {
        assert!(
            [
                "missing_where",
                "unknown_proof",
                "bulk_write",
                "weak_predicate",
                "read_without_limit",
                "read_projection",
                "insert_unverified"
            ]
            .contains(&f.as_str().unwrap())
        );
    }
}

fn allowed() {
    for mode in ["offline", "pg", "my"] {
        let mut c = Client::new(mode);
        c.init();
        let t = c.request(json!(1), "tools/list", json!({}));
        assert_eq!(
            t["result"]["tools"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| t["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "quent_plan",
                "quent_compile",
                "quent_review",
                "quent_manifest_status"
            ]
        );
        for tool in t["result"]["tools"].as_array().unwrap() {
            assert_eq!(
                tool["inputSchema"],
                json!({"type":"object","properties":{"source":{"type":"string","maxLength":4096}},"required":["source"],"additionalProperties":false})
            );
            assert_eq!(
                tool.as_object()
                    .unwrap()
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                ["annotations", "description", "inputSchema", "name"]
            );
            assert_eq!(
                tool["annotations"],
                json!({"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true,"openWorldHint":false})
            );
            assert!(["Typed read compilation requires application registration; unavailable in unregistered CLI.","Offline spec review; typed with application registration, syntax-only otherwise.","Offline manifest public summary; never registration or live verification."].contains(&tool["description"].as_str().unwrap()));
        }
        let r = c.request(json!(2), "resources/list", json!({}));
        assert_eq!(
            r["result"],
            json!({"resources":[
                {"uri":"quent://public/context","name":"context","mimeType":"application/json"},
                {"uri":"quent://public/schema","name":"schema","mimeType":"application/json"},
                {"uri":"quent://public/manifest-status","name":"manifest-status","mimeType":"application/json"},
                {"uri":"quent://public/review-rules","name":"review-rules","mimeType":"application/json"}
            ]})
        );
        for (n, uri) in RESOURCES.iter().enumerate() {
            let r = c.request(json!(n + 3), "resources/read", json!({"uri":uri}));
            assert_eq!(r["result"]["contents"][0]["uri"], *uri);
            let body: Value =
                serde_json::from_str(r["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
            let expected = match n {
                0 => {
                    json!({"kind":"quent_mcp_context","version":1,"registration":if mode=="offline"{"missing"}else{"application_asserted"},"freshness":"unverified","runtime":"unverified","execution":"unavailable"})
                }
                1 => {
                    json!({"kind":"quent_mcp_schema","version":1,"models":if mode=="offline"{0}else{1},"columns":if mode=="offline"{0}else{4},"coverage":if mode=="offline"{"unknown"}else{"application_asserted"},"freshness":"unverified"})
                }
                2 => {
                    json!({"kind":"quent_mcp_manifest_status","version":1,"schema":"missing","policy":"missing","generated_code":"missing","live":"unverified"})
                }
                _ => {
                    json!({"kind":"quent_mcp_rules","version":1,"mode":"strict","authority":"none","unsupported":"refuse","runtime":"unverified","rules":["read_only","offline_only","public_output_only","application_context_required","no_execution","no_permit","no_live_freshness"]})
                }
            };
            assert_eq!(body, expected);
        }
        for (n, tool) in ["quent_compile", "quent_plan"].iter().enumerate() {
            let r = c.call(json!(n + 10), tool, SPEC);
            if mode == "offline" {
                assert_eq!(r["error"]["message"], "missing_application_context");
            } else {
                let d = if mode == "pg" {
                    Dialect::Postgres
                } else {
                    Dialect::Mysql
                };
                let ctx = mcp_fixture::context(d, SqlType::Text, true, "user", vec![]);
                let map = mcp_fixture::mapping(&ctx, SqlType::Text, true);
                let expected = quent::operation_spec::SpecSource::parse(SPEC.as_bytes())
                    .unwrap()
                    .compile(&map, &ctx)
                    .unwrap()
                    .public_view()
                    .to_json();
                let actual = content(&r);
                assert_eq!(actual, serde_json::from_str::<Value>(&expected).unwrap());
                independent_public(&actual);
            }
        }
        let r = c.call(json!(12), "quent_review", SPEC);
        let expected = if mode == "offline" {
            quent::review::operation_spec(SPEC.as_bytes())
                .unwrap()
                .to_json()
        } else {
            let d = if mode == "pg" {
                Dialect::Postgres
            } else {
                Dialect::Mysql
            };
            let ctx = mcp_fixture::context(d, SqlType::Text, true, "user", vec![]);
            let map = mcp_fixture::mapping(&ctx, SqlType::Text, true);
            quent::review::typed_spec(
                quent::operation_spec::SpecSource::parse(SPEC.as_bytes()).unwrap(),
                &map,
                &ctx,
            )
            .unwrap()
            .to_json()
        };
        assert_eq!(
            content(&r),
            serde_json::from_str::<Value>(&expected).unwrap()
        );
        c.finish(0);
    }
}
fn refusal() {
    for mode in ["offline", "pg", "my", "mismatch"] {
        let mut c = Client::new(mode);
        c.init();
        let mut id = 1;
        for name in [
            "select", "insert", "update", "delete", "raw", "apply", "permit", "live", "explain",
            "command", "tx", CANARY,
        ] {
            let r = c.call(json!(id), name, CANARY);
            id += 1;
            assert!(r.get("error").is_some());
        }
        for uri in [
            "file:///etc/passwd",
            "https://example.invalid",
            CANARY,
            "quent://public/schema/CANARY_SQL_tenant_path_secret",
        ] {
            assert!(
                c.request(json!(id), "resources/read", json!({"uri":uri}))
                    .get("error")
                    .is_some()
            );
            id += 1;
        }
        for source in [
            SPEC.replace("\"version\":1", "\"version\":2"),
            SPEC.replace(
                "\"model\":\"canary_items\"",
                "\"model\":\"CANARY_SQL_tenant_path_secret\"",
            ),
            SPEC.replace("\"fields\":[\"id\",\"payload\"]", "\"fields\":[\"absent\"]"),
            SPEC.replace("\"field\":\"id\"", "\"field\":\"tenant\""),
            SPEC.replace("\"limit\":10", "\"limit\":10,\"tenant\":123"),
            SPEC.replace("\"limit\":10", "\"limit\":10,\"fresh\":true"),
            SPEC.replace(
                "\"limit\":10",
                "\"limit\":10,\"raw\":\"CANARY_SQL_tenant_path_secret\"",
            ),
            SPEC.replace("\"limit\":10", "\"limit\":10,\"join\":[]"),
            "{}".into(),
        ] {
            let r = c.call(json!(id), "quent_compile", &source);
            id += 1;
            assert!(r.get("error").is_some());
        }
        if mode == "mismatch" {
            assert!(
                c.call(json!(id), "quent_compile", SPEC)
                    .get("error")
                    .is_some()
            );
        }
        c.finish(0);
    }
}
fn ids_and_protocol() {
    let mut c = Client::new("offline");
    assert_eq!(
        c.request(json!(1), "tools/list", json!({}))["error"]["message"],
        "invalid_state"
    );
    assert_eq!(c.request(json!(2),"initialize",json!({"protocolVersion":CANARY,"clientInfo":{"name":CANARY,"version":"1"},"capabilities":{}}))["result"]["protocolVersion"],PROTOCOL_VERSION);
    assert!(
        c.request(json!(3), "tools/list", json!({}))
            .get("error")
            .is_some()
    );
    c.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    for id in [
        json!(CANARY),
        json!("1"),
        json!("λ\n\"\\"),
        json!("x".repeat(128)),
        json!("λ".repeat(64)),
        json!(9_007_199_254_740_991i64),
        json!(-9_007_199_254_740_991i64),
    ] {
        assert_eq!(c.request(id, "ping", json!({}))["result"], json!({}));
    }
    assert!(
        c.request(json!(4), "initialize", json!({}))
            .get("error")
            .is_some()
    );
    c.call(json!(CANARY.to_owned() + "2"), CANARY, CANARY);
    c.send(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":CANARY,"reason":CANARY}}));
    c.send(json!({"jsonrpc":"2.0","method":"notifications/unknown","params":{"secret":CANARY}}));
    assert_eq!(c.request(json!(5), "ping", json!({}))["result"], json!({}));
    c.finish(0);
}
fn malformed() {
    let invalid = [
        br#"{"jsonrpc":"2.0","id":-0,"method":"ping"}
"#
        .to_vec(),
        br#"{"jsonrpc":"2.0","id":1e0,"method":"ping"}
"#
        .to_vec(),
        b"{\"jsonrpc\":\"2.0\",\"id\":1,\"id\":2,\"method\":\"ping\"}\n".to_vec(),
        b"{} {}\n".to_vec(),
        vec![255, b'\n'],
        b"[]\n".to_vec(),
        b"{\"jsonrpc\":\"1.0\",\"id\":1,\"method\":\"ping\"}\n".to_vec(),
        format!("{}0{}\n", "[".repeat(17), "]".repeat(17)).into_bytes(),
    ];
    for b in invalid {
        let mut c = Client::new("offline");
        c.raw(b);
        c.finish(2);
        assert!(c.rx.try_recv().is_err());
    }
    for id in [
        json!(null),
        json!(true),
        json!([]),
        json!({}),
        json!(1.5),
        json!(9_007_199_254_740_992i64),
        json!(-9_007_199_254_740_992i64),
        json!("x".repeat(129)),
        json!("λ".repeat(65)),
    ] {
        let mut c = Client::new("offline");
        c.send(json!({"jsonrpc":"2.0","id":id,"method":"ping"}));
        c.finish(2);
    }
    let mut c = Client::new("offline");
    c.raw([vec![b' '; 65_537], vec![b'\n']].concat());
    c.wait(3);
    let mut c = Client::new("offline");
    c.raw(b"{\"jsonrpc\":".to_vec());
    c.wait(6);
    let mut c = Client::new("offline");
    c.raw(b"{".to_vec());
    c.finish(2);
    let mut c = Client::new("offline");
    c.send(json!({"jsonrpc":"2.0","id":1e0,"method":"ping"}));
    c.finish(2);
}
fn exact_budgets_and_negotiation() {
    let mut c = Client::new("offline");
    let mut exact = br#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#.to_vec();
    exact.resize(65536, b' ');
    exact.push(b'\n');
    c.raw(exact);
    assert_eq!(c.response(json!(1))["result"], json!({}));
    c.finish(0);
    for source in [
        "{\"a\":0,\"\\u0061\":1}\n".to_owned(),
        format!("{{\"s\":\"{}\"}}\n", "x".repeat(4097)),
        format!("[{}]\n", vec!["0"; 257].join(",")),
        format!(
            "{{{} }}\n",
            (0..33)
                .map(|i| format!("\"k{i}\":0"))
                .collect::<Vec<_>>()
                .join(",")
        ),
        format!("{{\"{}\":0}}\n", "k".repeat(129)),
    ] {
        let mut c = Client::new("offline");
        c.raw(source.into_bytes());
        c.finish(2);
    }
    let mut c = Client::new("offline");
    c.init();
    let mut s = SPEC.to_owned();
    s.push_str(&" ".repeat(4096 - s.len()));
    assert!(c.call(json!(1), "quent_review", &s).get("result").is_some());
    s.push(' ');
    c.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"quent_review","arguments":{"source":s}}}));
    c.finish(2);
    let mut c = Client::new("offline");
    assert_eq!(c.request(json!(1),"initialize",json!({"protocolVersion":"old", "capabilities":{},"clientInfo":{"name":"client","version":"1"}}))["result"]["protocolVersion"],PROTOCOL_VERSION);
    c.finish(0);
    let mut c = Client::new("offline");
    for (i,p) in [json!({"protocolVersion":2,"capabilities":{},"clientInfo":{"name":"c","version":"1"}}),json!({"capabilities":{},"clientInfo":{"name":"c","version":"1"}}),json!({"protocolVersion":"2025-11-25","capabilities":{"unknown":{}},"clientInfo":{"name":"c","version":"1"}})].into_iter().enumerate(){assert!(c.request(json!(i),"initialize",p).get("error").is_some());}
    c.finish(0);
}
fn session_limits() {
    let mut c = Client::new("offline");
    for id in 0..256 {
        assert_eq!(c.request(json!(id), "ping", json!({}))["result"], json!({}));
    }
    c.send(json!({"jsonrpc":"2.0","id":256,"method":"ping"}));
    c.finish(3);
    let mut c = Client::new("offline");
    c.request(json!("x"), "ping", json!({}));
    c.send(json!({"jsonrpc":"2.0","id":"x","method":"ping"}));
    c.finish(3);
    let mut c = Client::new("offline");
    for _ in 0..512 {
        c.send(json!({"jsonrpc":"2.0","method":"notifications/unknown"}));
    }
    c.wait(3);
}
fn detector_controls() {
    for v in [
        json!({"result":CANARY}),
        json!({"result":{"id":CANARY}}),
        json!({"error":{"data":{"id":CANARY}}}),
    ] {
        assert!(leaks(&v));
    }
    assert!(String::from_utf8_lossy(CANARY.as_bytes()).contains(CANARY));
    assert!(!public_keys(&json!({"sql":CANARY})));
}
fn manifest_and_canary() {
    for mode in ["offline", "pg", "my"] {
        let mut c = Client::new(mode);
        c.init();
        let source = include_str!("../../../fixtures/manifest-v1/postgres.json");
        let v = c.call(json!(CANARY), "quent_manifest_status", source);
        let expected: Value = serde_json::from_str(
            &quent::review::manifest(source.as_bytes())
                .unwrap()
                .to_json(),
        )
        .unwrap();
        assert_eq!(content(&v), expected);
        let s = SPEC.replace(
            "\"field\":\"id\",\"value\":{\"type\":\"i64\",\"value\":7}",
            &format!(
                "\"field\":\"payload\",\"value\":{{\"type\":\"text\",\"value\":\"{CANARY}\"}}"
            ),
        );
        assert!(
            c.call(json!(CANARY.to_owned() + "-value"), "quent_review", &s)
                .get("result")
                .is_some()
        );
        let r = c.request(
            json!(2),
            "resources/read",
            json!({"uri":"quent://public/manifest-status"}),
        );
        let body: Value =
            serde_json::from_str(r["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(body["schema"], "missing");
        assert_eq!(body["live"], "unverified");
        c.finish(0);
    }
}
fn network_detector() {
    use std::net::{TcpListener, TcpStream};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let control = TcpStream::connect(addr).unwrap();
    let (accepted, _) = listener.accept().unwrap();
    drop(control);
    drop(accepted);
    listener.set_nonblocking(true).unwrap();
    for mode in ["offline", "pg", "my"] {
        let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
        cmd.args(["--fixture-child", mode]);
        let mut c = Client::command_dsn(&mut cmd, &format!("postgres://CANARY@{addr}/secret"));
        c.init();
        let _ = c.call(json!(1), "quent_review", SPEC);
        let _ = c.call(json!(2), "quent_compile", SPEC);
        let _ = c.call(json!(3), "live", CANARY);
        c.finish(0);
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
}
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.get(1).map(String::as_str) == Some("--fixture-child") {
        let mode = args.get(2).unwrap();
        let s = if mode == "offline" {
            Server::offline()
        } else {
            let d = if mode == "my" {
                Dialect::Mysql
            } else {
                Dialect::Postgres
            };
            let c = mcp_fixture::context(d, SqlType::Text, true, "user", vec![]);
            let m = mcp_fixture::mapping(&c, SqlType::Text, true);
            let current = if mode == "mismatch" {
                mcp_fixture::context(d, SqlType::Text, true, "other_subject", vec![])
            } else {
                c
            };
            Server::application_registered(m, current)
        };
        std::process::exit(i32::from(s.serve_stdio().exit_code()));
    }
    for (name, test) in [
        ("allowed", allowed as fn()),
        ("refusal", refusal),
        ("ids_and_protocol", ids_and_protocol),
        ("malformed", malformed),
        ("session_limits", session_limits),
        (
            "exact_budgets_and_negotiation",
            exact_budgets_and_negotiation,
        ),
        ("detector_controls", detector_controls),
        ("manifest_and_canary", manifest_and_canary),
        ("network_detector", network_detector),
    ] {
        test();
        println!("test mcp_stdio::{name} ... ok");
    }
    println!("test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out");
}
