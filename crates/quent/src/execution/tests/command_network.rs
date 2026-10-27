//! Dedicated plaintext loopback fixture; never logs or retains protocol payloads.
//! Native transaction control frames only. Not a production proxy/TLS guarantee.
use std::{
    io::{Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
#[derive(Clone, Copy)]
pub(super) enum Mode {
    BeforeCommit,
    LoseReply,
    Pass,
    ChangedRows,
}
pub(super) struct Proxy {
    pub address: SocketAddr,
    pub sent: Arc<AtomicBool>,
    pub reply: Arc<AtomicBool>,
    task: Option<JoinHandle<()>>,
}
pub(super) fn read_frame(s: &mut TcpStream, pg: bool, startup: bool) -> std::io::Result<Vec<u8>> {
    let header = if pg && !startup { 5 } else { 4 };
    let mut b = vec![0; header];
    s.read_exact(&mut b)?;
    let n = if pg {
        let at = usize::from(!startup);
        (u32::from_be_bytes(b[at..at + 4].try_into().unwrap()) as usize)
            .checked_sub(4)
            .ok_or(std::io::ErrorKind::InvalidData)?
    } else {
        usize::from(b[0]) | usize::from(b[1]) << 8 | usize::from(b[2]) << 16
    };
    if n > 1_048_576 {
        return Err(std::io::ErrorKind::InvalidData.into());
    }
    b.resize(header + n, 0);
    s.read_exact(&mut b[header..])?;
    Ok(b)
}
impl Proxy {
    pub fn new(target: SocketAddr, pg: bool, mode: Mode) -> Self {
        assert!(target.ip().is_loopback());
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let sent = Arc::new(AtomicBool::new(false));
        let reply = Arc::new(AtomicBool::new(false));
        let sent2 = sent.clone();
        let reply2 = reply.clone();
        let task = thread::spawn(move || {
            let (client, _) = listener.accept().unwrap();
            let server = TcpStream::connect_timeout(&target, Duration::from_secs(10)).unwrap();
            for s in [&client, &server] {
                s.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
                s.set_write_timeout(Some(Duration::from_secs(15))).unwrap();
            }
            let mut from_client = client.try_clone().unwrap();
            let mut to_server = server.try_clone().unwrap();
            let phase = Arc::new(AtomicBool::new(false));
            let phase2 = phase.clone();
            let upload = thread::spawn(move || {
                let mut startup = pg;
                let mut first = true;
                while let Ok(mut frame) = read_frame(&mut from_client, pg, startup) {
                    // Isolated native changed-count control only. Clear the
                    // CLIENT_FOUND_ROWS bit in the first plaintext MySQL client
                    // handshake; production connections never use this mode.
                    if first && !pg && matches!(mode, Mode::ChangedRows) {
                        assert!(frame.len() >= 36);
                        assert_ne!(frame[4] & 2, 0);
                        frame[4] &= !2;
                    }
                    first = false;
                    let commit = !startup
                        && if pg {
                            frame[0] == b'Q' && &frame[5..] == b"COMMIT\0"
                        } else {
                            &frame[4..] == b"\x03COMMIT"
                        };
                    startup = false;
                    if commit {
                        if matches!(mode, Mode::BeforeCommit) {
                            break;
                        }
                        phase2.store(true, Ordering::SeqCst);
                    }
                    if to_server.write_all(&frame).is_err() {
                        break;
                    }
                    if commit {
                        sent2.store(true, Ordering::SeqCst);
                    }
                }
                let _ = from_client.shutdown(Shutdown::Both);
                let _ = to_server.shutdown(Shutdown::Both);
            });
            let mut server = server;
            let mut client = client;
            while let Ok(frame) = read_frame(&mut server, pg, false) {
                if phase.load(Ordering::SeqCst) {
                    let ack = if pg {
                        frame[0] == b'C' && &frame[5..] == b"COMMIT\0"
                    } else {
                        frame.get(4) == Some(&0)
                    };
                    if ack {
                        reply2.store(true, Ordering::SeqCst);
                    }
                    if matches!(mode, Mode::LoseReply) {
                        // Consume the actual server COMMIT acknowledgement, send
                        // none of it to SQLx, then terminate only this connection.
                        assert!(ack, "unexpected fixed commit response class");
                        break;
                    }
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
