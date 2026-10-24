//! Captures actual tracing emission metadata, span creation/records and Debug fields.
#![allow(dead_code)]
use std::sync::{Arc, Mutex};
use tracing::{
    Event, Metadata, Subscriber,
    span::{Attributes, Id, Record},
};
#[derive(Clone, Default)]
pub struct Capture(pub Arc<Mutex<Vec<String>>>);
struct Fields<'a>(&'a mut String);
impl tracing::field::Visit for Fields<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        write!(self.0, "|{}={value:?}", field.name()).unwrap();
    }
}
impl Subscriber for Capture {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, a: &Attributes<'_>) -> Id {
        let mut s = format!("SPAN|{}|{}", a.metadata().target(), a.metadata().name());
        a.record(&mut Fields(&mut s));
        self.0.lock().unwrap().push(s);
        Id::from_u64(1)
    }
    fn record(&self, _: &Id, r: &Record<'_>) {
        let mut s = "RECORD".to_owned();
        r.record(&mut Fields(&mut s));
        self.0.lock().unwrap().push(s);
    }
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, e: &Event<'_>) {
        let mut s = format!("EVENT|{}|{}", e.metadata().target(), e.metadata().name());
        e.record(&mut Fields(&mut s));
        self.0.lock().unwrap().push(s);
    }
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
}
impl Capture {
    pub fn assert_safe(&self, expected: usize) {
        let entries = self.0.lock().unwrap();
        assert_eq!(entries.len(), expected);
        for entry in entries.iter() {
            assert!(entry.starts_with("EVENT|quent::public|quent_public_view|"));
            assert!(entry.contains("|public_view="));
            for bad in [
                "CANARY",
                "quent_dispatch_test",
                "canary_value",
                "canary",
                "SELECT ",
                "UPDATE ",
                "DELETE ",
                "987654321",
                "\\u001b",
                "[REDACTED]",
            ] {
                assert!(!entry.contains(bad), "unsafe tracing field");
            }
        }
    }
}

pub fn check(v: &impl quent::output_tracing::PublicSummary, expected: &str) {
    let capture = Capture::default();
    tracing::subscriber::with_default(capture.clone(), || quent::output_tracing::emit(v));
    capture.assert_safe(1);
    let entries = capture.0.lock().unwrap();
    let (_, json) = entries[0].split_once("|public_view=").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert!(parsed == serde_json::from_str::<serde_json::Value>(expected).unwrap());
}
