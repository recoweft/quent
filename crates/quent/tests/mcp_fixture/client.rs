#![allow(dead_code)]
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, SyncSender},
    time::{Duration, Instant},
};
pub const CANARY: &str = "CANARY_SQL_tenant_path_secret";
pub const SPEC: &str = r#"{"kind":"quent_operation_spec","version":1,"model":"canary_items","fields":["id","payload"],"order":[],"limit":10,"filter":{"op":"eq","field":"id","value":{"type":"i64","value":7}}}"#;
pub struct Client {
    child: Child,
    tx: SyncSender<Option<Vec<u8>>>,
    ack: Receiver<()>,
    pub rx: Receiver<Vec<u8>>,
    err: Receiver<Vec<u8>>,
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Client {
    pub fn new(mode: &str) -> Self {
        Self::command(
            Command::new(std::env::current_exe().unwrap()).args(["--fixture-child", mode]),
        )
    }
    pub fn command(cmd: &mut Command) -> Self {
        Self::command_dsn(cmd, &format!("postgres://{CANARY}@127.0.0.1:1/secret"))
    }
    pub fn command_dsn(cmd: &mut Command, dsn: &str) -> Self {
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("DATABASE_URL", dsn)
            .env("RUST_LOG", "trace")
            .env("RUST_BACKTRACE", "full")
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let (tx, wire) = mpsc::sync_channel::<Option<Vec<u8>>>(1);
        let (done, ack) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            while let Ok(Some(b)) = wire.recv() {
                if stdin.write_all(&b).and_then(|_| stdin.flush()).is_err() {
                    break;
                }
                if done.send(()).is_err() {
                    break;
                }
            }
        });
        let stdout = child.stdout.take().unwrap();
        let (out, rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut r = BufReader::new(stdout);
            loop {
                let mut b = Vec::new();
                let n = r.by_ref().take(65_537).read_until(b'\n', &mut b).unwrap();
                if n == 0 {
                    return;
                }
                if n > 65_536 || !b.ends_with(b"\n") {
                    return;
                }
                if out.send(b).is_err() {
                    return;
                }
            }
        });
        let stderr = child.stderr.take().unwrap();
        let (etx, err) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut b = Vec::new();
            stderr.take(4097).read_to_end(&mut b).unwrap();
            let _ = etx.send(b);
        });
        Self {
            child,
            tx,
            ack,
            rx,
            err,
        }
    }
    pub fn raw(&self, b: Vec<u8>) {
        self.tx.send(Some(b)).unwrap();
        let _ = self.ack.recv_timeout(Duration::from_secs(3));
    }
    pub fn send(&self, v: Value) {
        let mut b = serde_json::to_vec(&v).unwrap();
        b.push(b'\n');
        self.raw(b);
    }
    pub fn response(&self, id: Value) -> Value {
        let b = self
            .rx
            .recv_timeout(Duration::from_secs(4))
            .expect("bounded stdio response");
        let mut v: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], id);
        let keys = v
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert!(keys == ["id", "jsonrpc", "result"] || keys == ["error", "id", "jsonrpc"]);
        if let Some(error) = v.get("error") {
            assert_eq!(
                error
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                ["code", "message"]
            );
            let code = error["code"].as_i64().unwrap();
            let message = error["message"].as_str().unwrap();
            assert!(
                [
                    (-32600, "invalid_state"),
                    (-32601, "method_not_found"),
                    (-32602, "invalid_params"),
                    (-32602, "input_refused"),
                    (-32002, "resource_not_found"),
                    (-32000, "missing_application_context")
                ]
                .contains(&(code, message))
            );
        }
        // ONLY top-level /id is exempt. Nested id/error data are still scanned.
        v.as_object_mut().unwrap().remove("id");
        assert_safe(&v);
        v
    }
    pub fn request(&self, id: Value, method: &str, params: Value) -> Value {
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        self.response(id)
    }
    pub fn init(&self) {
        let r=self.request(json!(0),"initialize",json!({"protocolVersion":"2025-11-25","clientInfo":{"name":CANARY,"version":"1"},"capabilities":{}}));
        assert_eq!(
            r["result"],
            json!({"protocolVersion":"2025-11-25","capabilities":{"resources":{},"tools":{}},"serverInfo":{"name":"quent-readonly","version":"0.1.0"}})
        );
        self.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    }
    pub fn call(&self, id: Value, name: &str, source: &str) -> Value {
        self.request(
            id,
            "tools/call",
            json!({"name":name,"arguments":{"source":source}}),
        )
    }
    pub fn finish(&mut self, expected: i32) {
        let _ = self.tx.send(None);
        self.wait(expected);
    }
    pub fn wait(&mut self, expected: i32) {
        let end = Instant::now() + Duration::from_secs(8);
        loop {
            if let Some(s) = self.child.try_wait().unwrap() {
                assert_eq!(s.code(), Some(expected));
                break;
            }
            assert!(
                Instant::now() < end,
                "child deadline (Drop kills and reaps)"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(self.err.recv_timeout(Duration::from_secs(1)).unwrap(), b"");
    }
}
pub fn assert_safe(v: &Value) {
    assert!(!leaks(v));
}

pub fn leaks(v: &Value) -> bool {
    v.to_string().contains(CANARY) || v.to_string().contains("canary_")
}
