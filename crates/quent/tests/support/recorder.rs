//! Scripted observations only: no connection, runtime gate, SQL or SQLx executor.
use super::fixture::{Dialect, Input, Operation};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Descriptor {
    Fixture { case: String, scenario: String },
    Probe,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub operation: Operation,
    pub dialect: Dialect,
    pub descriptor: Descriptor,
    pub args: Vec<Input>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Begin,
    Call(Call),
    Commit,
    Rollback,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountKind {
    PostgresAffected,
    MysqlChanged,
    MysqlMatched,
    MysqlUpsert,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    Driver,
    Decode,
    Commit,
    Unexpected,
    InvalidState,
    InvalidInput,
    InvalidResult,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    Unit,
    Count { kind: CountKind, value: u64 },
    Rows(Vec<Vec<Input>>),
    Error(Failure),
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub event: Event,
    pub reply: Reply,
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum Tx {
    #[default]
    Idle,
    Active,
    Unknown,
}
// Clone is an independent deep snapshot, deliberately never Arc<Mutex<...>>.
#[derive(Debug, Clone, Default)]
pub struct Recorder {
    script: VecDeque<Observation>,
    log: Vec<Observation>,
    tx: Tx,
}
impl Recorder {
    pub fn new(script: Vec<Observation>) -> Self {
        Self {
            script: script.into(),
            ..Self::default()
        }
    }
    pub fn reset(&mut self, script: Vec<Observation>) {
        *self = Self::new(script);
    }
    pub fn observations(&self) -> &[Observation] {
        &self.log
    }
    pub fn call_count(&self) -> usize {
        self.log
            .iter()
            .filter(|o| matches!(o.event, Event::Call(_)))
            .count()
    }
    pub fn verify(&self, expected: &[Observation]) -> Result<(), Failure> {
        if !self.script.is_empty() || self.log != expected {
            Err(Failure::Unexpected)
        } else {
            Ok(())
        }
    }
    pub fn record(&mut self, event: Event) -> Reply {
        let invalid = match &event {
            Event::Begin if self.tx != Tx::Idle => Some(Failure::InvalidState),
            Event::Commit | Event::Rollback if self.tx != Tx::Active => Some(Failure::InvalidState),
            Event::Call(_) if self.tx == Tx::Unknown => Some(Failure::InvalidState),
            Event::Call(c)
                if c.args
                    .iter()
                    .any(|v| matches!(v, Input::Missing { .. }) || v.validate().is_err()) =>
            {
                Some(Failure::InvalidInput)
            }
            _ => None,
        };
        let mut reply = if let Some(e) = invalid {
            Reply::Error(e)
        } else if self.script.front().is_some_and(|s| s.event == event) {
            self.script.pop_front().unwrap().reply
        } else {
            Reply::Error(Failure::Unexpected)
        };
        let valid_result = match (&event, &reply) {
            (_, Reply::Error(_) | Reply::Unknown) => true,
            (Event::Call(c), Reply::Count { kind, .. }) => matches!(
                (&c.dialect, kind),
                (Dialect::Postgres, CountKind::PostgresAffected)
                    | (
                        Dialect::Mysql,
                        CountKind::MysqlChanged | CountKind::MysqlMatched | CountKind::MysqlUpsert
                    )
            ),
            (Event::Call(_), Reply::Rows(rows)) => rows
                .iter()
                .flatten()
                .all(|v| !matches!(v, Input::Missing { .. }) && v.validate().is_ok()),
            (Event::Begin | Event::Commit | Event::Rollback, Reply::Unit) => true,
            _ => false,
        };
        if !valid_result {
            reply = Reply::Error(Failure::InvalidResult);
        }
        match (&event, &reply) {
            (_, Reply::Unknown) => self.tx = Tx::Unknown,
            (Event::Begin, Reply::Unit) => self.tx = Tx::Active,
            (Event::Commit | Event::Rollback, Reply::Unit) => self.tx = Tx::Idle,
            // Commit failure does not imply rollback or success; outcome is unknown.
            (Event::Commit | Event::Rollback, Reply::Error(_)) => self.tx = Tx::Unknown,
            _ => {}
        }
        self.log.push(Observation {
            event,
            reply: reply.clone(),
        });
        reply
    }
}
