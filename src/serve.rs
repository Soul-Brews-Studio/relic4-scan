//! `relic-scan serve` / `up`, and the desktop app's window: OUT/meta/map.html on 127.0.0.1 only. One page, three views:
//!   /  /scan   Smart Scan: the files indexed, the index button        /banks  homes and repositories
//!   /scans    the scan log; Smart Scan also holds the contents sunburst (rows by kind, .jsonl kind, place)
//!   /health  checks computed from the index                            /map    the session star map
//!   /tree     homes -> repos -> sessions -> satellites
//!   /paths    every .jsonl the scan saw, by folder: indexed or not, and why (data: /paths.json)
//! and three endpoints for the page's "index" and "check changes" buttons:
//!   GET  /api/status   {"running", "job", "phase", "done", "total", "secs", "last_secs", "last_error", "runs", "ready", "check"}
//!   POST /api/run      starts an incremental run in this process (202), or 409 when a run or a check is going.
//!   POST /api/check    starts a dry run (202): scan and compare with the state, write nothing; the result is
//!                      "check" in the status (new, grown, rewritten, unchanged, gone, files, bytes, at) until a run.
//!   GET  /api/log?from=N  the server's output while a run or check goes (lines from N on, the next N, the live line)
//!   GET  /api/export?sid=…&months=…&fmt=md|jsonl  one session's rows as a download (Markdown or JSON lines)
//!   POST /api/run?only=…, /api/check?only=…  the same, over the files whose path holds that text
//!   POST /api/reveal?path=…|sid=…  show a .jsonl the scan saw (or a folder holding one) in Finder; other paths are refused
//!   POST /api/stop     stop the run or check going on: it stops with nothing written, unless its merge has started
//!   GET  /api/config   the Settings (OUT/_config.json), the defaults, OUT and the version; POST saves (X-Relic)
//!   POST /api/pick     the desktop's folder picker (macOS choose folder); the chosen path, or 204 when cancelled
//!   POST /api/save?name=…  write the page's export (CSV/JSON body) to ~/Downloads and show it in Finder
//!   GET  /api/scans    the scan log (OUT/_scans.jsonl), newest first, without file lists
//!   GET  /api/scans/N  one entry with its file list (the files the run read, or a check would read, and gone ones)
//!                      Both POSTs need an `X-Relic: 1` header: a page on another site cannot send one without a CORS
//!                      preflight this server never answers, so only this page (or curl) can start them.
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
    job: &'static str,                // "run" or "check": what is going, or went last
    check: Option<crate::Report>,     // the last check, until a run makes it stale
    stopped: bool,                    // the last run or check ended on Stop, with nothing written
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
        "running": j.running, "job": j.job, "stopping": j.running && crate::CANCEL.load(Relaxed), "stopped": !j.running && j.stopped,
        "phase": phase, "check": j.check,
        "walked": crate::scan::WALKED.load(Relaxed), "listed": crate::scan::LISTED.load(Relaxed),
        "done": crate::parse::DONE.load(Relaxed), "total": crate::parse::TOTAL.load(Relaxed),
        "secs": j.started.filter(|_| j.running).map(|t| t.elapsed().as_secs_f64()),
        "last_secs": j.last_secs, "last_error": j.last_error, "runs": j.runs,
        "ready": c.out.join("meta").join("map.html").exists(),
        "out": c.out.display().to_string(), "kind": c.kind,
    })
}

/// A path the page draws itself: "/", "/map.html", or one lower-case word ("/scan", "/disk", a view a newer page adds).
/// Serving the page for any such word keeps a newer page working with an older server.
fn is_view(p: &str) -> bool {
    p == "/" || p == "/map.html" || (p.len() > 1 && p[1..].bytes().all(|b| b.is_ascii_lowercase()))
}

/// Every .jsonl the last scan saw (OUT/meta/jsonl.parquet), indexed or not.
fn seen_paths(out: &Path) -> Vec<String> {
    use arrow::array::{Array, StringArray};
    let Ok(f) = std::fs::File::open(out.join("meta").join("jsonl.parquet")) else { return vec![] };
    let Ok(b) = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(f) else { return vec![] };
    let Ok(r) = b.build() else { return vec![] };
    let mut v = vec![];
    for batch in r.flatten() {
        if let Some(a) = batch.column_by_name("path").and_then(|c| c.as_any().downcast_ref::<StringArray>()) {
            v.extend((0..a.len()).filter(|&i| !a.is_null(i)).map(|i| a.value(i).to_string()));
        }
    }
    v
}

/// An export the page made (CSV or JSON), written to ~/Downloads under a plain name; the page cannot save files
/// itself inside the desktop app's web view. The name is checked: letters, digits, dot, dash, underscore, .csv or .json.
fn save_export(q: &std::collections::HashMap<String, String>, body: &[u8]) -> Result<String, String> {
    let name = q.get("name").ok_or("needs a name")?;
    let ok = name.len() <= 120 && !name.starts_with('.') && name.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
        && (name.ends_with(".csv") || name.ends_with(".json"));
    if !ok || body.is_empty() {
        return Err("a plain .csv or .json name and a body".into());
    }
    let dir = Path::new(&crate::expand("~/Downloads")).to_path_buf();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let p = dir.join(name);
    std::fs::write(&p, body).map_err(|e| e.to_string())?;
    Ok(p.display().to_string())
}

/// The desktop's own folder picker, shown by the server (it runs on this machine): `choose folder` on macOS, zenity
/// elsewhere. None when the user cancels or there is no picker. The page cannot get a real path from a browser picker.
fn pick_folder() -> Option<String> {
    let out = if cfg!(target_os = "macos") {
        std::process::Command::new("osascript").args(["-e", r#"POSIX path of (choose folder with prompt "A folder for relic-scan to walk")"#]).output()
    } else {
        std::process::Command::new("zenity").args(["--file-selection", "--directory"]).output()
    }.ok()?;
    let p = String::from_utf8_lossy(&out.stdout).trim().trim_end_matches('/').to_string();
    (out.status.success() && !p.is_empty()).then_some(p)
}

/// What `/api/reveal` may show: a .jsonl the scan saw (by path, or a session by id), or a folder holding one.
/// Anything else is refused, so the button can never be used to open an arbitrary path.
fn reveal_target(out: &Path, q: &std::collections::HashMap<String, String>) -> Option<(String, bool)> {
    // the index's own tables (sessions, files, every .jsonl), for an export too big to save from the page
    match q.get("what").map(String::as_str) {
        Some("meta") => return Some((out.join("meta").display().to_string(), false)),
        // the whole index folder: the month files, _state.tsv, the scan log and meta/
        Some("out") => return Some((out.display().to_string(), false)),
        // one month's file (OUT/2026-10.parquet), shown selected
        Some("month") => return crate::meta::month_file(out, q.get("m")?).map(|p| (p.display().to_string(), true)),
        // a ψ vault, or a note or folder inside one (only vaults the last run listed)
        Some("vault") => return crate::vault::reveal_target(out, q.get("path")?),
        _ => {}
    }
    let st = crate::state::load(out);
    if let Some(sid) = q.get("sid").filter(|s| !s.is_empty() && !s.contains('/')) {
        let tail = format!("/{sid}.jsonl");
        let mut hits: Vec<&String> = st.keys().filter(|k| k.ends_with(&tail)).collect();
        hits.sort_by_key(|k| (k.contains("mirror") || k.contains("archive") || k.contains("_bak"), k.len()));
        return hits.first().map(|p| (p.to_string(), true));
    }
    let p = q.get("path")?.trim_end_matches('/').to_string();
    if p.is_empty() || p.contains("/../") || p.ends_with("/..") {
        return None;
    }
    let seen = seen_paths(out);
    if st.contains_key(&p) || seen.iter().any(|k| *k == p) {
        return Some((p, true));
    }
    let dir = format!("{p}/");
    (st.keys().any(|k| k.starts_with(&dir)) || seen.iter().any(|k| k.starts_with(&dir))).then_some((p, false))
}

/// Show a file in the desktop's file manager (selected), or open a folder.
fn reveal(path: &str, file: bool) -> std::io::Result<()> {
    let mut cmd = if cfg!(target_os = "macos") {
        let mut c = std::process::Command::new("open");
        if file { c.arg("-R"); }
        c.arg(path);
        c
    } else {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(if file { Path::new(path).parent().map(|d| d.display().to_string()).unwrap_or_default() } else { path.to_string() });
        c
    };
    cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map(|_| ())
}

/// The query string of a request line, %-decoded (`+` is a space).
fn query(line: &str) -> std::collections::HashMap<String, String> {
    let target = line.split_whitespace().nth(1).unwrap_or("");
    let q = target.split_once('?').map(|(_, q)| q).unwrap_or("").split('#').next().unwrap_or("");
    let dec = |s: &str| {
        let b = s.as_bytes();
        let mut out = Vec::with_capacity(b.len());
        let mut i = 0;
        while i < b.len() {
            match b[i] {
                b'+' => out.push(b' '),
                b'%' if i + 2 < b.len() => match std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    Some(v) => {
                        out.push(v);
                        i += 2;
                    }
                    None => out.push(b'%'),
                },
                c => out.push(c),
            }
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    };
    q.split('&').filter(|p| !p.is_empty()).map(|p| { let (k, v) = p.split_once('=').unwrap_or((p, "")); (dec(k), dec(v)) }).collect()
}

/// Start a run (or, with `dry`, a check that writes nothing) in a thread, over every file or only the files whose
/// path holds `only`; Err(message) when it cannot start.
fn start(c: &Arc<Ctx>, dry: bool, only: Option<String>) -> Result<(), String> {
    let mut j = c.job.lock().unwrap();
    if j.running {
        return Err(format!("a {} is already going", j.job));
    }
    // a check takes no lock: it only reads, so it may run beside a `relic-scan run` in another shell
    let lock = if dry { None } else { Some(crate::try_lock_out(&c.out)?) };
    j.running = true;
    j.job = if dry { "check" } else { "run" };
    j.started = Some(Instant::now());
    j.last_error = None;
    drop(j);
    let c = c.clone();
    std::thread::spawn(move || {
        let args = crate::RunArgs {
            out: c.out.display().to_string(), roots: vec![], depth: 8, threads: 0, exclude: vec![], full: false, agent: None,
            jt_compat: false, only: only.into_iter().collect(), kind: c.kind.clone(), month: None, day: None, from: None, to: None, dry_run: dry,
        };
        // the run's own output (scan ticker, parse bar, summary card) goes to the page's output popup as well
        let cap = crate::tee::start();
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::run_inner(args, lock, true)));
        if let Some(cap) = cap {
            crate::tee::stop(cap);
        }
        crate::PHASE.store(0, Relaxed);
        let mut j = c.job.lock().unwrap();
        j.running = false;
        j.stopped = matches!(&r, Ok(rep) if rep.stopped);
        crate::CANCEL.store(false, Relaxed);
        if !dry && !j.stopped {
            j.runs += 1;
            j.last_secs = j.started.map(|t| t.elapsed().as_secs_f64());
        }
        match &r {
            Ok(_) if j.stopped => {}
            Ok(rep) if dry => j.check = Some(rep.clone()),
            Ok(_) => j.check = None, // the index now holds what the check saw
            Err(_) => {}
        }
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
    let mut length = 0usize;
    let mut h = String::new();
    while rd.read_line(&mut h).is_ok_and(|n| n > 2) {
        let low = h.to_ascii_lowercase();
        relic_header |= low.starts_with("x-relic:");
        if let Some(v) = low.strip_prefix("content-length:") {
            length = v.trim().parse().unwrap_or(0);
        }
        h.clear();
    }
    // a POST body: settings (small) or an export to save (at most 32 MB)
    let mut body = vec![0u8; length.min(32 << 20)];
    if !body.is_empty() && std::io::Read::read_exact(&mut rd, &mut body).is_err() {
        return;
    }
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");
    let path = path.split(['?', '#']).next().unwrap_or("/");
    let page = c.out.join("meta").join("map.html");
    match (method, path) {
        ("GET", p) if is_view(p) => match std::fs::read(&page) {
            Ok(b) => reply(s, "200 OK", "text/html; charset=utf-8", &b),
            Err(_) => reply(s, "200 OK", "text/html; charset=utf-8", WELCOME.replace("{OUT}", &c.out.display().to_string()).as_bytes()),
        },
        ("GET", "/paths.json" | "/vaults.json") => {
            let p = page.with_file_name(&path[1..]);
            match std::fs::read(&p) {
                Ok(b) => reply(s, "200 OK", "application/json", &b),
                Err(e) => reply(s, "404 Not Found", "text/plain", format!("{}: {e}\n", p.display()).as_bytes()),
            }
        }
        ("GET", "/api/status") => json(s, "200 OK", status(c)),
        ("GET", "/api/months") => json(s, "200 OK", crate::meta::months(&c.out)),
        // a vault note's text for the reader: /api/note?path=… (markdown inside a listed vault only)
        ("GET", "/api/note") => match query(&line).get("path").and_then(|p| crate::vault::note_text(&c.out, p)) {
            Some(t) => reply(s, "200 OK", "text/markdown; charset=utf-8", t.as_bytes()),
            None => reply(s, "404 Not Found", "text/plain", b"not a note in a listed vault\n"),
        },
        // one vault's notes: /vaults/<id>.json
        ("GET", p) if p.starts_with("/vaults/") && p.ends_with(".json") => match crate::vault::notes_file(&c.out, &p[8..p.len() - 5]).map(std::fs::read) {
            Some(Ok(b)) => reply(s, "200 OK", "application/json", &b),
            _ => reply(s, "404 Not Found", "text/plain", b"no such vault\n"),
        },
        ("GET", "/api/log") => {
            // /api/log?from=N: the output lines from N on, the next N, and the line being redrawn
            let from = line.split_whitespace().nth(1).and_then(|u| u.split_once("from=")).and_then(|(_, v)| v.split('&').next())
                .and_then(|v| v.parse().ok()).unwrap_or(0);
            let (lines, next, live) = crate::tee::since(from);
            let j = c.job.lock().unwrap();
            json(s, "200 OK", serde_json::json!({"lines": lines, "next": next, "live": live, "running": j.running, "job": j.job}))
        }
        ("GET", "/api/export") => {
            // /api/export?sid=…&months=2026-09,2026-10&fmt=md|jsonl&title=…: one session's rows, as a download
            let q = query(&line);
            let sid = q.get("sid").cloned().unwrap_or_default();
            let months: Vec<String> = q.get("months").map(|m| m.split(',').map(str::to_string).collect()).unwrap_or_default();
            let md = q.get("fmt").map(String::as_str) != Some("jsonl");
            if sid.is_empty() || sid.contains(['/', '\\', '"']) {
                return json(s, "400 Bad Request", serde_json::json!({"error": "needs a session id"}));
            }
            match crate::export::session(&c.out, &sid, &months) {
                Ok(rows) => {
                    let body = if md { crate::export::markdown(q.get("title").map(String::as_str).unwrap_or(&sid), &sid, &rows) } else { crate::export::jsonl(&rows) };
                    let (ctype, ext) = if md { ("text/markdown; charset=utf-8", "md") } else { ("application/x-ndjson", "jsonl") };
                    let head = format!("HTTP/1.1 200 OK\r\nContent-Type: {ctype}\r\nContent-Disposition: attachment; filename=\"{sid}.{ext}\"\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", body.len());
                    let mut s = s;
                    let _ = s.write_all(head.as_bytes()).and_then(|_| s.write_all(body.as_bytes()));
                }
                Err(e) => json(s, "500 Internal Server Error", serde_json::json!({"error": e})),
            }
        }
        ("GET", "/api/scans") => {
            let mut all = crate::scanlog::read(&c.out);
            all.reverse();
            all.truncate(500);
            json(s, "200 OK", serde_json::json!({"entries": all.iter().map(crate::scanlog::summary).collect::<Vec<_>>()}))
        }
        ("GET", p) if p.starts_with("/api/scans/") => {
            let i: Option<u64> = p["/api/scans/".len()..].parse().ok();
            match crate::scanlog::read(&c.out).into_iter().find(|e| e["i"].as_u64() == i && i.is_some()) {
                Some(e) => json(s, "200 OK", e),
                None => json(s, "404 Not Found", serde_json::json!({"error": "no such scan"})),
            }
        }
        ("GET", "/api/config") => json(s, "200 OK", serde_json::json!({
            "config": crate::config::load(&c.out), "defaults": crate::config::Config::default(),
            "out": c.out.display().to_string(), "kind": c.kind, "version": env!("CARGO_PKG_VERSION"),
        })),
        ("POST", "/api/config") if !relic_header => json(s, "403 Forbidden", serde_json::json!({"error": "needs the X-Relic header"})),
        ("POST", "/api/config") => match serde_json::from_slice::<crate::config::Config>(&body).map_err(|e| e.to_string())
            .and_then(|cfg| crate::config::save(&c.out, cfg)) {
            Ok(cfg) => json(s, "200 OK", serde_json::json!({"config": cfg})),
            Err(e) => json(s, "400 Bad Request", serde_json::json!({"error": e})),
        },
        ("POST", "/api/save") if !relic_header => json(s, "403 Forbidden", serde_json::json!({"error": "needs the X-Relic header"})),
        ("POST", "/api/save") => match save_export(&query(&line), &body) {
            Ok(p) => {
                let _ = reveal(&p, true);
                json(s, "200 OK", serde_json::json!({"path": p}))
            }
            Err(e) => json(s, "400 Bad Request", serde_json::json!({"error": e})),
        },
        ("POST", "/api/pick") if !relic_header => json(s, "403 Forbidden", serde_json::json!({"error": "needs the X-Relic header"})),
        ("POST", "/api/pick") => match pick_folder() {
            Some(p) => json(s, "200 OK", serde_json::json!({"path": p})),
            None => json(s, "204 No Content", serde_json::json!({})),
        },
        ("POST", "/api/stop") if !relic_header => json(s, "403 Forbidden", serde_json::json!({"error": "needs the X-Relic header"})),
        ("POST", "/api/stop") => {
            // Stop: honoured during the walk and the parse; once the merge has started the run finishes
            let running = c.job.lock().unwrap().running;
            let merging = crate::PHASE.load(Relaxed) >= 3;
            if running && !merging {
                crate::CANCEL.store(true, Relaxed);
            }
            json(s, if running && !merging { "202 Accepted" } else { "409 Conflict" }, status(c))
        }
        ("POST", "/api/reveal") if !relic_header => json(s, "403 Forbidden", serde_json::json!({"error": "needs the X-Relic header"})),
        ("POST", "/api/reveal") => match reveal_target(&c.out, &query(&line)) {
            Some((p, file)) => match reveal(&p, file) {
                Ok(()) => json(s, "200 OK", serde_json::json!({"path": p})),
                Err(e) => json(s, "500 Internal Server Error", serde_json::json!({"error": e.to_string()})),
            },
            None => json(s, "404 Not Found", serde_json::json!({"error": "not a file or folder this index holds"})),
        },
        ("POST", "/api/run" | "/api/check") if !relic_header => json(s, "403 Forbidden", serde_json::json!({"error": "needs the X-Relic header"})),
        ("POST", "/api/run" | "/api/check") => match start(c, path == "/api/check", query(&line).remove("only").filter(|o| !o.is_empty())) {
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
    // Settings > auto check: once a minute, start a check when none has started for that long and nothing is running
    {
        let c = c.clone();
        std::thread::spawn(move || {
            let mut last = Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
                let every = crate::config::load(&c.out).auto_check_min as u64;
                let busy = { let j = c.job.lock().unwrap(); if let Some(t) = j.started { last = last.max(t); } j.running };
                if every > 0 && !busy && last.elapsed().as_secs() >= every * 60 && start(&c, true, None).is_ok() {
                    last = Instant::now();
                }
            }
        });
    }
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

#[cfg(test)]
mod tests {
    #[test]
    fn views_are_single_words() {
        for v in ["/", "/scan", "/disk", "/map.html", "/somefutureview"] {
            assert!(super::is_view(v), "{v}");
        }
        for v in ["/api/status", "/paths.json", "/favicon.ico", "/Scan", "/../etc", "/a-b"] {
            assert!(!super::is_view(v), "{v}");
        }
    }

    #[test]
    fn exports_save_under_plain_names_only() {
        let q = |n: &str| std::collections::HashMap::from([("name".to_string(), n.to_string())]);
        for bad in ["../x.csv", "x.sh", ".hidden.csv", "a/b.json", "x y.csv"] {
            assert!(super::save_export(&q(bad), b"a").is_err(), "{bad}");
        }
        assert!(super::save_export(&q("x.csv"), b"").is_err(), "an empty body is refused");
    }

    #[test]
    fn query_strings_decode() {
        let q = super::query("GET /api/export?sid=ab-1&title=neo%20oracle+x&months=2026-09%2C2026-10&bad=%zz HTTP/1.1");
        assert_eq!(q["sid"], "ab-1");
        assert_eq!(q["title"], "neo oracle x");
        assert_eq!(q["months"], "2026-09,2026-10");
        assert_eq!(q["bad"], "%zz");
        assert!(super::query("POST /api/run HTTP/1.1").is_empty());
    }
}
