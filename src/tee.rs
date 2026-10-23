//! The server's own output while a run or a check is going, for the page's output popup (`GET /api/log`).
//!
//! `start` points fd 2 at a pipe; a thread reads it, writes every byte on to the real stderr (so the terminal that
//! started `relic-scan serve` still shows it), and keeps the text: finished lines, plus the line being redrawn with
//! `\r` (the scan ticker and the parse bar), which replaces itself instead of piling up. `stop` puts fd 2 back.
//! Unix only: elsewhere `start` returns None and the popup shows the status lines instead.

use std::sync::Mutex;

pub const MAX_LINES: usize = 4000;

#[derive(Default)]
pub struct Log {
    pub base: usize,        // the number of the first line kept (older ones fell off)
    pub lines: Vec<String>, // finished lines
    pub live: String,       // the line being written or redrawn
}

pub static LOG: Mutex<Log> = Mutex::new(Log { base: 0, lines: Vec::new(), live: String::new() });

fn feed(log: &mut Log, text: &str) {
    for ch in text.chars() {
        match ch {
            '\n' => {
                let l = std::mem::take(&mut log.live);
                log.lines.push(l);
                if log.lines.len() > MAX_LINES {
                    let drop = log.lines.len() - MAX_LINES;
                    log.lines.drain(..drop);
                    log.base += drop;
                }
            }
            '\r' => log.live.clear(),
            c => log.live.push(c),
        }
    }
}

/// Lines from number `from` on, the number to ask from next time, and the live line.
pub fn since(from: usize) -> (Vec<String>, usize, String) {
    let l = LOG.lock().unwrap();
    let skip = from.saturating_sub(l.base).min(l.lines.len());
    (l.lines[skip..].to_vec(), l.base + l.lines.len(), l.live.clone())
}

#[cfg(unix)]
pub struct Capture {
    saved: i32,
    reader: std::thread::JoinHandle<()>,
}

#[cfg(unix)]
pub fn start() -> Option<Capture> {
    use std::io::{Read, Write};
    use std::os::fd::FromRawFd;
    *LOG.lock().unwrap() = Log::default();
    let mut fds = [0i32; 2];
    // SAFETY: plain POSIX calls on descriptors this function owns; fd 2 is restored by `stop`
    unsafe {
        if libc::pipe(fds.as_mut_ptr()) != 0 {
            return None;
        }
        let saved = libc::dup(2);
        if saved < 0 || libc::dup2(fds[1], 2) < 0 {
            libc::close(fds[0]);
            libc::close(fds[1]);
            return None;
        }
        libc::close(fds[1]);
        let mut rd = std::fs::File::from_raw_fd(fds[0]);
        let mut out = std::mem::ManuallyDrop::new(std::fs::File::from_raw_fd(saved)); // `stop` closes it
        let reader = std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            let mut pending: Vec<u8> = vec![];
            while let Ok(n) = rd.read(&mut buf) {
                if n == 0 {
                    break;
                }
                let _ = out.write_all(&buf[..n]);
                pending.extend_from_slice(&buf[..n]);
                // keep a UTF-8 sequence cut by the read for the next chunk
                let ok = match std::str::from_utf8(&pending) { Ok(_) => pending.len(), Err(e) => e.valid_up_to() };
                let text = String::from_utf8_lossy(&pending[..ok]).into_owned();
                pending.drain(..ok);
                feed(&mut LOG.lock().unwrap(), &text);
            }
        });
        Some(Capture { saved, reader })
    }
}

#[cfg(unix)]
pub fn stop(c: Capture) {
    // SAFETY: puts the saved stderr back on fd 2; that closes the pipe's last write end, so the reader sees EOF
    unsafe {
        libc::dup2(c.saved, 2);
    }
    let _ = c.reader.join();
    unsafe {
        libc::close(c.saved);
    }
}

#[cfg(not(unix))]
pub struct Capture;
#[cfg(not(unix))]
pub fn start() -> Option<Capture> {
    None
}
#[cfg(not(unix))]
pub fn stop(_: Capture) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_redrawn_line_replaces_itself() {
        let mut l = Log::default();
        feed(&mut l, "scan 1\rscan 2\rscan 3");
        assert_eq!((l.lines.len(), l.live.as_str()), (0, "scan 3"));
        feed(&mut l, "\r\nrun: done\nnext");
        assert_eq!(l.lines, ["", "run: done"]);
        assert_eq!(l.live, "next");
    }
}
