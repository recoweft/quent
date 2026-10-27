//! Finite plaintext loopback UPDATE-response control, compiled only with tests.
//! One dedicated SQLx connection, one prepared business execute, no payload logs.
use super::command_network::read_frame;
use std::{
    io::Write,
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub(super) struct Proxy {
    pub address: SocketAddr,
    pub sent: Arc<AtomicUsize>,
    pub reply: Arc<AtomicBool>,
    pub withheld: Arc<AtomicBool>,
    task: Option<JoinHandle<()>>,
}
impl Proxy {
    pub fn new(target: SocketAddr, pg: bool, hold: bool) -> Self {
        assert!(target.ip().is_loopback());
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let sent = Arc::new(AtomicUsize::new(0));
        let reply = Arc::new(AtomicBool::new(false));
        let withheld = Arc::new(AtomicBool::new(false));
        let (sent2, reply2, withheld2) = (sent.clone(), reply.clone(), withheld.clone());
        let task = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let (client, _) = loop {
                match listener.accept() {
                    Ok(pair) => break pair,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "fixture accept deadline");
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(_) => panic!("fixture accept failed"),
                }
            };
            let server = TcpStream::connect_timeout(&target, Duration::from_secs(10)).unwrap();
            for socket in [&client, &server] {
                socket
                    .set_read_timeout(Some(Duration::from_secs(15)))
                    .unwrap();
                socket
                    .set_write_timeout(Some(Duration::from_secs(15)))
                    .unwrap();
            }
            let mut from_client = client.try_clone().unwrap();
            let mut to_server = server.try_clone().unwrap();
            let executing = Arc::new(AtomicBool::new(false));
            let executing2 = executing.clone();
            let upload = thread::spawn(move || {
                let mut startup = pg;
                while let Ok(frame) = read_frame(&mut from_client, pg, startup) {
                    // PG extended Execute / MySQL COM_STMT_EXECUTE. Prepare and
                    // handshake replies must flow normally, including MySQL's OK.
                    let execute = !startup
                        && if pg {
                            frame[0] == b'E'
                        } else {
                            frame.get(4) == Some(&0x17)
                        };
                    startup = false;
                    if execute {
                        assert_eq!(
                            sent2.fetch_add(1, Ordering::SeqCst),
                            0,
                            "fixture execute count"
                        );
                        executing2.store(true, Ordering::SeqCst);
                    }
                    if to_server.write_all(&frame).is_err() {
                        break;
                    }
                }
                let _ = from_client.shutdown(Shutdown::Both);
                let _ = to_server.shutdown(Shutdown::Both);
            });
            let mut server = server;
            let mut client = client;
            while let Ok(frame) = read_frame(&mut server, pg, false) {
                let completion = executing.load(Ordering::SeqCst)
                    && if pg {
                        // This fixture's UPDATE affects exactly one row. COMMIT
                        // and prepare responses cannot satisfy this detector.
                        frame[0] == b'C' && &frame[5..] == b"UPDATE 1\0"
                    } else {
                        // OK with affected_rows=1 on this single prepared UPDATE.
                        frame.get(4..6) == Some(&[0, 1])
                    };
                if completion {
                    reply2.store(true, Ordering::SeqCst);
                }
                if hold && reply2.load(Ordering::SeqCst) {
                    // Consume C/OK and following ReadyForQuery without delivering
                    // completion to SQLx. Keep reading so server autocommit can
                    // finish; observer visibility, not C/OK alone, proves apply.
                    withheld2.store(true, Ordering::SeqCst);
                    continue;
                }
                if client.write_all(&frame).is_err() {
                    break;
                }
            }
            let _ = client.shutdown(Shutdown::Both);
            let _ = server.shutdown(Shutdown::Both);
            upload.join().unwrap();
        });
        Self {
            address,
            sent,
            reply,
            withheld,
            task: Some(task),
        }
    }
}
impl Drop for Proxy {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.join().unwrap();
        }
    }
}
