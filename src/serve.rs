//! `relic-scan serve` / `up`, and the desktop app's window: OUT/meta/map.html on 127.0.0.1 only. One page, three views:
//!   /  /scan   Smart Scan: the files indexed, the index button        /banks  homes and repositories
//!   /health  checks computed from the index                            /map    the session star map
//!   /tree     homes -> repos -> sessions -> satellites
//!   /paths    every .jsonl the scan saw, by folder: indexed or not, and why (data: /paths.json)
//! and two endpoints for the page's "index" button:
//!   GET  /api/status   {"running", "phase", "done", "total", "secs", "last_secs", "last_error", "runs", "ready"}
//!   POST /api/run      starts an incremental run in this process (202), or 409 when one is running.
//!                      It needs an `X-Relic: 1` header: a page on another site cannot send one without a CORS
//!                      preflight this server never answers, so only this page (or curl) can start a run.
//! Before the first run there is no map.html; every view then shows a welcome page with the same button.
//! The page is read from disk on every request, so a `relic-scan run` in another shell shows up on reload.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const WELCOME: &str = include_str!("welcome.html");

#[derive(Default)]
struct Job {
    running: bool,
    started: Option<Instant>,
    last_secs: Option<f64>,
    last_error: Option<String>,
    runs: u64,
}

struct Ctx {
    out: PathBuf,
    kind: Vec<String>,
    job: Mutex<Job>,
}

fn reply(mut s: TcpStream, status: &str, ctype: &str, body: &[u8]) {
    let head = format!("HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
                       body.len());
    let _ = s.write_all(head.as_bytes()).and_then(|_| s.write_all(body));
}

fn json(s: TcpStream, status: &str, v: serde_json::Value) {
    reply(s, status, "application/json", v.to_string().as_bytes());
}

fn status(c: &Ctx) -> serde_json::Value {
    let j = c.job.lock().unwrap();
    let phase = ["idle", "scan", "parse", "merge", "meta"][crate::PHASE.load(Relaxed).min(4) as usize];
    serde_json::json!({
        "running": j.running,
        "phase": phase,
        "walked": crate::scan::WALKED.load(Relaxed), "listed": crate::scan::LISTED.load(Relaxed),
        "done": crate::parse::DONE.load(Relaxed), "total": crate::parse::TOTAL.load(Relaxed),
        "secs": j.started.filter(|_| j.running).map(|t| t.elapsed().as_secs_f64()),
        "last_secs": j.last_secs, "last_error": j.last_error, "runs": j.runs,
        "ready": c.out.join("meta").join("map.html").exists(),
        "out": c.out.display().to_string(), "kind": c.kind,
    })
}

/// Start a run in a thread; Err(status, message) when it cannot start.
fn start(c: &Arc<Ctx>) -> Result<(), String> {
    let mut j = c.job.lock().unwrap();
    if j.running {
        return Err("a run is already going".into());
    }
    let lock = crate::try_lock_out(&c.out)?;
    j.running = true;
    j.started = Some(Instant::now());
    j.last_error = None;
    drop(j);
    let c = c.clone();
    std::thread::spawn(move || {
        let args = crate::RunArgs {
            out: c.out.display().to_string(), roots: vec![], depth: 8, threads: 0, exclude: vec![], full: false, agent: None,
            jt_compat: false, only: vec![], kind: c.kind.clone(),
        };
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::run_with(args, Some(lock))));
        crate::PHASE.store(0, Relaxed);
        let mut j = c.job.lock().unwrap();
        j.running = false;
        j.runs += 1;
        j.last_secs = j.started.map(|t| t.elapsed().as_secs_f64());
        if let Err(e) = r {
            j.last_error = Some(e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "the run panicked".into()));
        }
    });
    Ok(())
}

fn handle(s: TcpStream, c: &Arc<Ctx>) {
    // a client that never finishes its request must not hold a thread forever
    let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(10)));
    let mut rd = BufReader::new(match s.try_clone() {
        Ok(c) => c,
        Err(_) => return,
    });
    let mut line = String::new();
    if rd.read_line(&mut line).is_err() {
        return;
    }
    let mut relic_header = false;
    let mut h = String::new();
    while rd.read_line(&mut h).is_ok_and(|n| n > 2) {
        relic_header |= h.to_ascii_lowercase().starts_with("x-relic:");
        h.clear();
    }
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");
    let path = path.split(['?', '#']).next().unwrap_or("/");
    let page = c.out.join("meta").join("map.html");
    match (method, path) {
        ("GET", "/" | "/scan" | "/banks" | "/health" | "/map" | "/tree" | "/paths" | "/map.html") => match std::fs::read(&page) {
            Ok(b) => reply(s, "200 OK", "text/html; charset=utf-8", &b),
            Err(_) => reply(s, "200 OK", "text/html; charset=utf-8", WELCOME.replace("{OUT}", &c.out.display().to_string()).as_bytes()),
        },
        ("GET", "/paths.json") => {
            let p = page.with_file_name("paths.json");
            match std::fs::read(&p) {
                Ok(b) => reply(s, "200 OK", "application/json", &b),
                Err(e) => reply(s, "404 Not Found", "text/plain", format!("{}: {e}\n", p.display()).as_bytes()),
            }
        }
        ("GET", "/api/status") => json(s, "200 OK", status(c)),
        ("POST", "/api/run") if !relic_header => json(s, "403 Forbidden", serde_json::json!({"error": "needs the X-Relic header"})),
        ("POST", "/api/run") => match start(c) {
            Ok(()) => json(s, "202 Accepted", status(c)),
            Err(e) => json(s, "409 Conflict", serde_json::json!({"error": e})),
        },
        (_, "/favicon.ico") => reply(s, "204 No Content", "text/plain", b""),
        _ => reply(s, "404 Not Found", "text/plain", b"not found: try /map, /tree or /paths\n"),
    }
}

/// Bind 127.0.0.1:port (0 = any free port), or exit with a message.
pub fn bind(port: u16) -> TcpListener {
    TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| {
        eprintln!("serve: 127.0.0.1:{port}: {e} (another server? try --port)");
        std::process::exit(1);
    })
}

/// Serve OUT on an already bound listener until the process ends. `kind` is what the index button runs with.
pub fn serve_on(l: TcpListener, out: &Path, kind: Vec<String>) {
    let c = Arc::new(Ctx { out: out.to_path_buf(), kind, job: Mutex::default() });
    for s in l.incoming().flatten() {
        let c = c.clone();
        std::thread::spawn(move || handle(s, &c));
    }
}

pub fn serve(out: &Path, port: u16, kind: Vec<String>) {
    let l = bind(port);
    let port = l.local_addr().map(|a| a.port()).unwrap_or(port);
    let ready = out.join("meta").join("map.html").exists();
    eprintln!("serve: http://127.0.0.1:{port}/map · /tree · /paths  ({}{})  ctrl-c to stop", out.display(),
              if ready { "" } else { ", not indexed yet: the page has an index button" });
    serve_on(l, out, kind);
}
