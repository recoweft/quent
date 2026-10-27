//! Test-only bounded plaintext loopback server-response holding. No SQL/payload logs.
use std::{
    io::{Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
fn frame(s: &mut TcpStream, pg: bool, startup: bool) -> std::io::Result<Vec<u8>> {
    let mut h = vec![0; if pg && !startup { 5 } else { 4 }];
    s.read_exact(&mut h)?;
    let n = if pg {
        u32::from_be_bytes(h[if startup { 0 } else { 1 }..].try_into().unwrap()) as usize - 4
    } else {
        usize::from(h[0]) | (usize::from(h[1]) << 8) | (usize::from(h[2]) << 16)
    };
    if n > 1_048_576 {
        return Err(std::io::ErrorKind::InvalidData.into());
    }
    let old = h.len();
    h.resize(old + n, 0);
    s.read_exact(&mut h[old..])?;
    Ok(h)
}
pub(super) struct Proxy {
    pub address: SocketAddr,
    pub server_ack: Arc<AtomicBool>,
    pub ddl_executes: Arc<AtomicUsize>,
    sockets: Arc<Mutex<Vec<TcpStream>>>,
    task: Option<JoinHandle<()>>,
}
impl Proxy {
    /// hold_step is zero-based; None holds PostgreSQL COMMIT instead.
    pub fn new(target: SocketAddr, pg: bool, hold_step: Option<usize>) -> Self {
        assert!(target.ip().is_loopback());
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let sockets = Arc::new(Mutex::new(vec![]));
        let server_ack = Arc::new(AtomicBool::new(false));
        let ddl_executes = Arc::new(AtomicUsize::new(0));
        let (sockets2, ack2, exec2) = (sockets.clone(), server_ack.clone(), ddl_executes.clone());
        let task = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let (client, _) = loop {
                match listener.accept() {
                    Ok(v) => break v,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "fixture accept deadline");
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(_) => panic!("fixture accept"),
                }
            };
            let server = TcpStream::connect_timeout(&target, Duration::from_secs(5)).unwrap();
            for s in [&client, &server] {
                s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                s.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
            }
            sockets2
                .lock()
                .unwrap()
                .extend([client.try_clone().unwrap(), server.try_clone().unwrap()]);
            let pending = Arc::new(AtomicBool::new(false));
            let pending2 = pending.clone();
            let mut upload_client = client.try_clone().unwrap();
            let mut upload_server = server.try_clone().unwrap();
            let upload = thread::spawn(move || {
                let mut startup = pg;
                let mut prepared_ddl = false;
                let mut prepared_commit = false;
                while let Ok(f) = frame(&mut upload_client, pg, startup) {
                    if !startup {
                        let sql = if pg && f[0] == b'P' {
                            let body = &f[5..];
                            let pos = body.iter().position(|b| *b == 0).unwrap();
                            Some(&body[pos + 1..])
                        } else if (pg && f[0] == b'Q') || (!pg && f.get(4) == Some(&0x16)) {
                            Some(&f[5..])
                        } else {
                            None
                        };
                        if let Some(sql) = sql {
                            prepared_ddl = sql.starts_with(b"ALTER TABLE ");
                            prepared_commit = sql.starts_with(b"COMMIT");
                        }
                        let execute = if pg {
                            f[0] == b'E' || f[0] == b'Q'
                        } else {
                            f.get(4) == Some(&0x17)
                        };
                        if execute {
                            let ddl = if prepared_ddl {
                                Some(exec2.fetch_add(1, Ordering::SeqCst))
                            } else {
                                None
                            };
                            if (ddl.is_some() && ddl == hold_step)
                                || (pg && hold_step.is_none() && prepared_commit)
                            {
                                pending2.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                    startup = false;
                    if upload_server.write_all(&f).is_err() {
                        break;
                    }
                }
                let _ = upload_server.shutdown(Shutdown::Both);
                let _ = upload_client.shutdown(Shutdown::Both);
            });
            let (mut server, mut client) = (server, client);
            let mut holding = false;
            while let Ok(f) = frame(&mut server, pg, false) {
                let completion = if pg {
                    f[0] == b'C'
                        && (f[5..].starts_with(b"ALTER TABLE") || f[5..].starts_with(b"COMMIT"))
                } else {
                    f.get(4) == Some(&0)
                };
                if pending.load(Ordering::SeqCst) && completion {
                    ack2.store(true, Ordering::SeqCst);
                    holding = true;
                }
                if !holding && client.write_all(&f).is_err() {
                    break;
                }
            }
            let _ = client.shutdown(Shutdown::Both);
            let _ = server.shutdown(Shutdown::Both);
            let _ = upload.join();
        });
        Self {
            address,
            server_ack,
            ddl_executes,
            sockets,
            task: Some(task),
        }
    }
}
impl Drop for Proxy {
    fn drop(&mut self) {
        for s in self.sockets.lock().unwrap().iter() {
            let _ = s.shutdown(Shutdown::Both);
        }
        if let Some(t) = self.task.take() {
            t.join().unwrap();
        }
    }
}
