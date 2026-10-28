//! Process-owned stdio only. Bounded channels, no unbounded input/history queue.
use super::*;
use std::{
    collections::BTreeSet,
    io::{BufReader, Read, Write},
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
    time::{Duration, Instant},
};
const FRAME: usize = 65_536;
const RESPONSE: usize = 65_536;
const WAIT: Duration = Duration::from_secs(2);
const IDLE: Duration = Duration::from_secs(5);
const SESSION: Duration = Duration::from_secs(60);
// No Debug/Display/serde on correlation carriers. They never enter handlers.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Id {
    Text(String),
    Integer(i64),
}
impl Id {
    fn parse(v: &Value) -> Option<Self> {
        match v {
            Value::String(s) if s.len() <= 128 => Some(Self::Text(s.clone())),
            Value::Number(n) => n
                .as_i64()
                .filter(|n| (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(n))
                .map(Self::Integer),
            _ => None,
        }
    }
    fn value(&self) -> Value {
        match self {
            Self::Text(s) => json!(s),
            Self::Integer(n) => json!(n),
        }
    }
}
enum Input {
    #[cfg(test)]
    Barrier(SyncSender<()>),
    Start,
    Frame(Value),
    End,
    Failed(Termination),
}
struct Job {
    method: String,
    params: Value,
}
fn reader(tx: SyncSender<Input>) {
    let mut input = BufReader::new(std::io::stdin());
    for _ in 0..512 {
        let mut bytes = Vec::with_capacity(1024);
        loop {
            let mut b = [0];
            match input.read(&mut b) {
                Ok(0) => {
                    let _ = tx.send(if bytes.is_empty() {
                        Input::End
                    } else {
                        Input::Failed(Termination::Invalid)
                    });
                    return;
                }
                Ok(_) => {}
                Err(_) => {
                    let _ = tx.send(Input::Failed(Termination::Io));
                    return;
                }
            }
            if bytes.is_empty() && tx.send(Input::Start).is_err() {
                return;
            }
            if b[0] == b'\n' {
                break;
            }
            if bytes.len() == FRAME {
                let _ = tx.send(Input::Failed(Termination::Budget));
                return;
            }
            bytes.push(b[0]);
        }
        // Existing pre-tree scanner rejects duplicate DECODED keys, floats,
        // invalid UTF-8, multiple documents, depth/node/key/string over-budget.
        if crate::operation_spec::bounded::check(&bytes).is_err() {
            let _ = tx.send(Input::Failed(Termination::Invalid));
            return;
        }
        match serde_json::from_slice(&bytes) {
            Ok(v) => {
                if tx.send(Input::Frame(v)).is_err() {
                    return;
                }
            }
            Err(_) => {
                let _ = tx.send(Input::Failed(Termination::Invalid));
                return;
            }
        }
    }
    let _ = tx.send(Input::Failed(Termination::Budget));
}
fn writer(rx: Receiver<Vec<u8>>, ack: SyncSender<bool>) {
    let mut out = std::io::stdout().lock();
    while let Ok(bytes) = rx.recv() {
        let ok = out.write_all(&bytes).and_then(|_| out.flush()).is_ok();
        if ack.send(ok).is_err() || !ok {
            return;
        }
    }
}
fn write_response(
    tx: &SyncSender<Vec<u8>>,
    ack: &Receiver<bool>,
    id: &Id,
    result: Result<Value, Fault>,
) -> Result<(), Termination> {
    let value = match result {
        Ok(v) => json!({"jsonrpc":"2.0","id":id.value(),"result":v}),
        Err(f) => json!({"jsonrpc":"2.0","id":id.value(),"error":f.wire()}),
    };
    let mut bytes = serde_json::to_vec(&value).map_err(|_| Termination::Invalid)?;
    // Never truncate findings or replace excess with an apparently clean result.
    if bytes.len() + 1 > RESPONSE {
        return Err(Termination::Budget);
    }
    bytes.push(b'\n');
    tx.try_send(bytes).map_err(|_| Termination::Io)?;
    match ack.recv_timeout(WAIT) {
        Ok(true) => Ok(()),
        Ok(false) => Err(Termination::Io),
        Err(_) => Err(Termination::Timeout),
    }
}
struct Active {
    id: Id,
    initialize: bool,
    cancelled: bool,
    started: Instant,
}
pub(super) fn serve(server: Server) -> Termination {
    // This hook is deliberately process-owned; source locations/payloads never
    // traverse the default panic renderer in any worker, including application use.
    std::panic::set_hook(Box::new(|_| {
        let _ = std::io::stderr().write_all(b"QUENT_PANIC\n");
    }));
    let (itx, irx) = mpsc::sync_channel(1);
    let (otx, orx) = mpsc::sync_channel(1);
    let (atx, arx) = mpsc::sync_channel(1);
    let (jtx, jrx) = mpsc::sync_channel::<Job>(2);
    let (rtx, rrx) = mpsc::sync_channel(1);
    // Spawn failures are fixed outcomes, never OS error text.
    if thread::Builder::new().spawn(move || reader(itx)).is_err()
        || thread::Builder::new()
            .spawn(move || writer(orx, atx))
            .is_err()
        || thread::Builder::new()
            .spawn(move || {
                let mut phase = Phase::New;
                while let Ok(job) = jrx.recv() {
                    let result = if job.method == "notifications/initialized" {
                        if phase == Phase::Initialized {
                            phase = Phase::Ready;
                        }
                        continue;
                    } else {
                        server.request(&job.method, &job.params, &mut phase)
                    };
                    if rtx.send(result).is_err() {
                        return;
                    }
                }
            })
            .is_err()
    {
        return Termination::Io;
    }
    coordinate(irx, otx, arx, jtx, rrx)
}
fn coordinate(
    irx: Receiver<Input>,
    otx: SyncSender<Vec<u8>>,
    arx: Receiver<bool>,
    jtx: SyncSender<Job>,
    rrx: Receiver<Result<Value, Fault>>,
) -> Termination {
    let mut ids = BTreeSet::new();
    let mut active: Option<Active> = None;
    let start = Instant::now();
    let mut last = start;
    let mut frame: Option<Instant> = None;
    let mut eof = false;
    let mut frames = 0usize;
    loop {
        if start.elapsed() >= SESSION
            || last.elapsed() >= IDLE
            || frame.is_some_and(|s| s.elapsed() >= WAIT)
            || active.as_ref().is_some_and(|a| a.started.elapsed() >= WAIT)
        {
            return Termination::Timeout;
        }
        match rrx.try_recv() {
            Ok(result) => {
                let Some(a) = active.take() else {
                    return Termination::Invalid;
                };
                if !a.cancelled
                    && let Err(e) = write_response(&otx, &arx, &a.id, result)
                {
                    return e;
                }
                last = Instant::now();
            }
            Err(mpsc::TryRecvError::Disconnected) => return Termination::Io,
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if eof && active.is_none() {
            return Termination::Eof;
        }
        let event = match irx.recv_timeout(Duration::from_millis(2)) {
            Ok(e) => e,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if eof {
                    continue;
                }
                return Termination::Io;
            }
        };
        last = Instant::now();
        match event {
            #[cfg(test)]
            Input::Barrier(ack) => {
                let _ = ack.send(());
            }
            Input::Start => frame = Some(Instant::now()),
            Input::End => {
                eof = true;
                frame = None;
            }
            Input::Failed(t) => return t,
            Input::Frame(v) => {
                frame = None;
                frames += 1;
                if frames > 512 {
                    return Termination::Budget;
                }
                let Ok(o) = object(
                    &v,
                    &["jsonrpc", "id", "method", "params"],
                    &["jsonrpc", "method"],
                ) else {
                    return Termination::Invalid;
                };
                if o["jsonrpc"] != "2.0" {
                    return Termination::Invalid;
                }
                let Some(method) = o["method"].as_str() else {
                    return Termination::Invalid;
                };
                let empty = json!({});
                let params = o.get("params").unwrap_or(&empty);
                if let Some(raw) = o.get("id") {
                    let Some(id) = Id::parse(raw) else {
                        return Termination::Invalid;
                    };
                    if ids.len() >= 256 || !ids.insert(id.clone()) || active.is_some() {
                        return Termination::Budget;
                    }
                    active = Some(Active {
                        id,
                        initialize: method == "initialize",
                        cancelled: false,
                        started: Instant::now(),
                    });
                    if jtx
                        .try_send(Job {
                            method: method.to_owned(),
                            params: params.clone(),
                        })
                        .is_err()
                    {
                        return Termination::Budget;
                    }
                } else {
                    // No notification generates a response, even if unknown or
                    // malformed. All still consume the finite frame/time budgets.
                    match method {
                        "notifications/initialized" if object(params, &[], &[]).is_ok() => {
                            if active.is_some() {
                                continue;
                            }
                            if jtx
                                .try_send(Job {
                                    method: method.to_owned(),
                                    params: params.clone(),
                                })
                                .is_err()
                            {
                                return Termination::Budget;
                            }
                        }
                        "notifications/cancelled" => {
                            if let Ok(p) = object(params, &["requestId", "reason"], &["requestId"])
                                && p.get("reason").is_none_or(Value::is_string)
                                && let Some(id) = Id::parse(&p["requestId"])
                                && let Some(a) = active.as_mut()
                                && !a.initialize
                                && a.id == id
                            {
                                a.cancelled = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_budget_is_failure_before_any_write_not_truncated_success() {
        let (tx, rx) = mpsc::sync_channel(1);
        let (_ack, ack) = mpsc::sync_channel(1);
        assert_eq!(
            write_response(
                &tx,
                &ack,
                &Id::Integer(1),
                Ok(json!({"content":"x".repeat(RESPONSE)}))
            ),
            Err(Termination::Budget)
        );
        assert!(rx.try_recv().is_err());
    }
    #[test]
    fn controlled_inflight_cancel_is_observed_and_suppresses_only_matching_reply() {
        for (matching, initialize) in [(true, false), (false, false), (true, true)] {
            let (itx, irx) = mpsc::sync_channel(8);
            let (otx, orx) = mpsc::sync_channel(1);
            let (atx, arx) = mpsc::sync_channel(1);
            let (jtx, jrx) = mpsc::sync_channel(2);
            let (rtx, rrx) = mpsc::sync_channel(1);
            let h = thread::spawn(move || coordinate(irx, otx, arx, jtx, rrx));
            itx.send(Input::Frame(
                json!({"jsonrpc":"2.0","id":1,"method":if initialize {"initialize"} else {"ping"}}),
            ))
            .unwrap();
            let _job = jrx.recv_timeout(WAIT).unwrap();
            itx.send(Input::Frame(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":if matching {1}else{2},"reason":"CANARY"}}))).unwrap();
            let (barrier, wait) = mpsc::sync_channel(1);
            itx.send(Input::Barrier(barrier)).unwrap();
            wait.recv_timeout(WAIT).unwrap();
            rtx.send(Ok(json!({}))).unwrap();
            if !matching || initialize {
                let b = orx.recv_timeout(WAIT).unwrap();
                assert_eq!(serde_json::from_slice::<Value>(&b).unwrap()["id"], 1);
                atx.send(true).unwrap();
            }
            itx.send(Input::End).unwrap();
            assert_eq!(h.join().unwrap(), Termination::Eof);
            assert!(orx.try_recv().is_err());
        }
    }
    #[test]
    fn controlled_worker_stall_times_out_and_inflight_overflow_closes() {
        for overflow in [false, true] {
            let (itx, irx) = mpsc::sync_channel(4);
            let (otx, _orx) = mpsc::sync_channel(1);
            let (_atx, arx) = mpsc::sync_channel(1);
            let (jtx, jrx) = mpsc::sync_channel(2);
            let (_rtx, rrx) = mpsc::sync_channel(1);
            let h = thread::spawn(move || coordinate(irx, otx, arx, jtx, rrx));
            itx.send(Input::Frame(
                json!({"jsonrpc":"2.0","id":1,"method":"ping"}),
            ))
            .unwrap();
            jrx.recv_timeout(WAIT).unwrap();
            if overflow {
                itx.send(Input::Frame(
                    json!({"jsonrpc":"2.0","id":2,"method":"ping"}),
                ))
                .unwrap();
            }
            assert_eq!(
                h.join().unwrap(),
                if overflow {
                    Termination::Budget
                } else {
                    Termination::Timeout
                }
            );
        }
    }
}
