#![cfg(feature = "mcp")]
#[path = "../../quent/tests/mcp_fixture/client.rs"]
mod client;
use client::*;
use serde_json::json;
use std::process::Command;
#[test]
fn real_unregistered_cli_initialize_list_read_and_fixed_compile_refusal() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_quent-cli"));
    cmd.args(["mcp", "--stdio"]);
    let mut c = Client::command(&mut cmd);
    c.init();
    let tools = c.request(json!(1), "tools/list", json!({}));
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 4);
    let resources = c.request(json!(2), "resources/list", json!({}));
    assert_eq!(
        resources["result"]["resources"].as_array().unwrap().len(),
        4
    );
    for (id, uri) in quent::mcp::RESOURCES.iter().enumerate() {
        let r = c.request(json!(id + 3), "resources/read", json!({"uri":uri}));
        assert!(r.get("result").is_some());
    }
    for (id, name) in ["quent_compile", "quent_plan"].iter().enumerate() {
        assert_eq!(
            c.call(json!(id + 7), name, SPEC)["error"]["message"],
            "missing_application_context"
        );
    }
    let r = c.call(json!(9), "quent_review", SPEC);
    let r: serde_json::Value =
        serde_json::from_str(r["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(r["state"], "unknown");
    assert_eq!(r["precision"]["binding"], "unverified");
    assert!(c.call(json!(10), "permit", CANARY).get("error").is_some());
    c.finish(0);
}
#[test]
fn grammar_refuses_before_stdio_or_file_io() {
    for args in [
        vec!["mcp"],
        vec!["mcp", "--http"],
        vec!["mcp", "--stdio", CANARY],
    ] {
        let o = Command::new(env!("CARGO_BIN_EXE_quent-cli"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.is_empty());
        assert_eq!(o.stderr, b"QUENT_MCP_INVALID\n");
    }
}
