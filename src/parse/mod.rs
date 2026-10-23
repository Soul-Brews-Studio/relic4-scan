//! Phase 2, `relic-scan parse`: every indexed file -> rows -> ONE file per Bangkok month.
//!
//!   index (phase 1)  ->  sort by size, biggest first (LPT)  ->  N threads take the next file (atomic index)
//!   per file: the bytes jt would send (complete lines; a last line without "\n" only if it is JSON;
//!             --jt-compat: stop at a line longer than jt's 8 MB chunk, as jt does)
//!   per line: store.parse_chunk exactly (engine.rs) -> event rows (text, thinking) + tool rows + one source
//!   per thread, per month: raw rows buffered, spilled to parts/<month>/t<thread>-<n>.arrow (copies kept)
//!   merge, per month: fold (s, h, idx) -> first copy by (file, line); sources = one per (h, agent, file),
//!             first line; machines = distinct agents; sorted by (ts, s, h, idx) -> out/<month>.parquet
//!
//! Output schema = relic4 truth (src/relic4/truth.py) minus msgkey, which `relic4 truth import` adds.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use arrow::array::{Array, ArrayRef, BooleanArray, BooleanBuilder, Int64Array, Int64Builder, ListBuilder, RecordBatch,
                   StringArray, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;

use crate::engine::{self, Tool};

const JT_CHUNK: usize = 8 << 20;

/// Bytes read so far and bytes to read in the current parse, for /api/status.
pub static DONE: AtomicU64 = AtomicU64::new(0);
pub static TOTAL: AtomicU64 = AtomicU64::new(0);
const SPILL_ROWS: usize = 20_000;

/// One row per (line, stream[, tool]) per copy: the live store's events/tools/sources, flattened.
#[derive(Clone)]
pub struct Raw {
    pub month: String,
    pub s: &'static str,
    pub h: i64,
    pub idx: i64,
    pub engine: &'static str,
    pub role: Option<String>,
    pub typ: Option<String>,
    pub sid: String,
    pub ts: String,
    pub ts_th: String,
    pub text: Option<String>,
    pub kind: Option<&'static str>,
    pub call_id: Option<String>,
    pub name: Option<String>,
    pub body: Option<String>,
    pub err: Option<bool>,
    pub red: Option<String>,
    pub file: Arc<str>, // one allocation per file, shared by its rows
    pub line: i64,
}

#[derive(Default)]
pub struct Stats {
    pub files: AtomicU64,
    pub bytes: AtomicU64,
    pub lines: AtomicU64,
    pub bad_json: AtomicU64,
    pub rows: AtomicU64,
    pub stuck: AtomicU64,
    pub out_of_range: AtomicU64,
}

/// The bytes jt would send for a whole file: cut after the last "\n"; a final line without "\n" counts only
/// if it parses as JSON (agent/jt.ts:113).
fn complete(data: &[u8]) -> &[u8] {
    match memchr::memrchr(b'\n', data) {
        Some(nl) if nl + 1 == data.len() => data,
        Some(nl) => {
            let tail = &data[nl + 1..];
            if serde_json::from_slice::<serde_json::Value>(tail).is_ok() { data } else { &data[..=nl] }
        }
        None => if serde_json::from_slice::<serde_json::Value>(data).is_ok() { data } else { &[] },
    }
}

/// Python bytes.splitlines(keepends=True): \n, \r\n and a lone \r end a line; the terminator stays on it.
fn split_lines(data: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while let Some(off) = memchr::memchr2(b'\n', b'\r', &data[i..]) {
        let j = i + off;
        let end = if data[j] == b'\r' && data.get(j + 1) == Some(&b'\n') { j + 2 } else { j + 1 };
        out.push(&data[start..end]);
        start = end;
        i = end;
        if i >= data.len() {
            break;
        }
    }
    if start < data.len() {
        out.push(&data[start..]);
    }
    out
}

/// Where a file was left: the byte after its last complete line, and the parse context at that point
/// (store.parse_chunk's ctx = line count, first session id, last timestamp).
#[derive(Clone, Default, Debug)]
pub struct Ctx {
    pub offset: u64,
    pub lines: i64,
    pub session: String,
    pub last_ms: Option<i64>,
    /// the ledger's view of the lines read so far: the first line's hash and the chain over every line (ledger.rs)
    pub first: i64,
    pub chain: crate::ledger::Chain,
}

/// Every complete line of a file from `from`, in chunks of `want` bytes, cut exactly as a whole-file read would
/// (complete(), split_lines()). `f` gets each line with its terminator and says whether to go on. Returns the bytes
/// of the lines it was given, or None when the file cannot be read.
pub(crate) fn each_line(path: &str, from: u64, want: usize, bytes_read: &AtomicU64, f: &mut dyn FnMut(&[u8]) -> bool) -> Option<u64> {
    use std::io::{Seek, SeekFrom};
    let mut file = fs::File::open(path).ok()?;
    file.seek(SeekFrom::Start(from)).ok()?;
    let mut used: u64 = 0;
    let (mut buf, mut eof, mut stop) = (Vec::new(), false, false);
    while !eof && !stop {
        let n = Read::by_ref(&mut file).take(want as u64).read_to_end(&mut buf).ok()?;
        bytes_read.fetch_add(n as u64, Ordering::Relaxed);
        eof = n < want;
        let body: &[u8] = if eof {
            complete(&buf)
        } else {
            match memchr::memrchr(b'\n', &buf) {
                Some(nl) => &buf[..=nl],
                None => continue, // one line longer than a chunk: read on until it ends
            }
        };
        let took = body.len();
        for line in split_lines(body) {
            if !f(line) {
                stop = true;
                break;
            }
            used += line.len() as u64;
        }
        buf.drain(..took);
    }
    Some(used)
}

/// store.parse_chunk from `ctx.offset` with `ctx`, to the end of the file's complete lines. Returns the new ctx.
/// How much of a file is read at a time (RELIC_SCAN_CHUNK bytes, default 32 MiB). The biggest files are parsed
/// first and side by side, so reading each one whole held the top N files in memory at once (3.3 GB on an M5 Mac).
fn chunk_size() -> usize {
    static C: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *C.get_or_init(|| std::env::var("RELIC_SCAN_CHUNK").ok().and_then(|v| v.parse().ok()).filter(|&n| n > 0).unwrap_or(32 << 20))
}

pub fn parse_file(path: &str, ctx: &Ctx, jt_compat: bool, st: &Stats, emit: &mut dyn FnMut(Raw)) -> Option<Ctx> {
    parse_file_chunked(path, ctx, jt_compat, st, emit, chunk_size())
}

fn parse_file_chunked(path: &str, ctx: &Ctx, jt_compat: bool, st: &Stats, emit: &mut dyn FnMut(Raw), want: usize) -> Option<Ctx> {
    let eng = engine::engine_of(path);
    let file: Arc<str> = Arc::from(path);
    let (mut nline, mut session, mut last) = (ctx.lines, ctx.session.clone(), ctx.last_ms);
    let (mut first, mut chain) = (ctx.first, ctx.chain);
    // read a chunk, parse up to its last "\n", carry the rest into the next chunk; at the end of the file the rest
    // is cut as a whole-file read would cut it (complete()), so the rows are the same for any chunk size
    let used = each_line(path, ctx.offset, want, &st.bytes, &mut |line: &[u8]| {
        if jt_compat && line.len() > JT_CHUNK {
            st.stuck.fetch_add(1, Ordering::Relaxed); // jt never ships a line longer than its chunk: the file stops here
            return false;
        }
        nline += 1;
        // every line goes into the ledger's chain, rows or not
        let h = engine::key(line);
        if nline == 1 {
            first = h;
        }
        chain = crate::ledger::step(&chain, h);
        let r = match engine::parse(line) {
            Some(serde_json::Value::Object(m)) => m,
            Some(_) => serde_json::Map::new(),
            None => {
                st.bad_json.fetch_add(1, Ordering::Relaxed);
                serde_json::Map::new()
            }
        };
        let x = engine::extract(&r);
        if !x.sess.is_empty() && session.is_empty() {
            session = x.sess.clone();
        }
        let mut ts = x.ts;
        if ts.is_some() {
            last = ts;
        } else if last.is_some() {
            ts = last; // a line without its own time takes the file's last one
        }
        let trs: Vec<Tool> = engine::extract_tools(line, &r);
        let think = engine::thinking_of(line, &r);
        if x.text.is_empty() && think.is_none() && trs.is_empty() {
            return true;
        }
        let stp = engine::stamp(ts);
        let sid = if x.sess.is_empty() { session.clone() } else { x.sess.clone() };
        // a thinking row is kept even with empty text: the model thought, only the text was not saved
        let think_text = think.clone().unwrap_or_default();
        for (s, body) in [("text", &x.text), ("thinking", &think_text)] {
            if body.is_empty() && !(s == "thinking" && think.is_some()) {
                continue;
            }
            let (clean, kinds) = engine::redact(body);
            emit(Raw {
                month: stp.month.clone(), s, h, idx: 0, engine: eng,
                role: Some(if s == "thinking" { "thinking".into() } else { x.role.clone() }), typ: Some(x.typ.clone()),
                sid: sid.clone(), ts: stp.ts.clone(), ts_th: stp.ts_th.clone(), text: Some(clean), kind: None, call_id: None,
                name: None, body: None, err: None, red: (!kinds.is_empty()).then_some(kinds), file: file.clone(), line: nline,
            });
        }
        for (idx, t) in trs.into_iter().enumerate() {
            let (clean, kinds) = engine::redact(&t.body);
            emit(Raw {
                month: stp.month.clone(), s: "tools", h, idx: idx as i64, engine: eng, role: None, typ: None,
                sid: sid.clone(), ts: stp.ts.clone(), ts_th: stp.ts_th.clone(), text: None, kind: Some(t.kind),
                call_id: Some(t.call_id), name: (!t.name.is_empty()).then_some(t.name), body: Some(clean), err: Some(t.err),
                red: (!kinds.is_empty()).then_some(kinds), file: file.clone(), line: nline,
            });
        }
        true
    })?;
    st.lines.fetch_add((nline - ctx.lines) as u64, Ordering::Relaxed);
    st.files.fetch_add(1, Ordering::Relaxed);
    Some(Ctx { offset: ctx.offset + used, lines: nline, session, last_ms: last, first, chain })
}

mod merge;
mod parts;
#[cfg(test)]
pub(crate) mod tests;

pub use merge::merge_month;
use parts::*;

// ---------- driver ----------

pub struct Job {
    pub path: String,
    pub size: u64,
    pub verdict: String,
    pub ctx: Ctx, // where to start (offset 0 + empty ctx for a new or rewritten file)
}

pub fn read_index(p: &str) -> Vec<Job> {
    let text = fs::read_to_string(p).expect("read index");
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().unwrap_or("").split('\t').collect();
    let (ip, is) = (head.iter().position(|c| *c == "path").unwrap(), head.iter().position(|c| *c == "size").unwrap());
    let iv = head.iter().position(|c| *c == "verdict");
    lines.map(|l| {
        let c: Vec<&str> = l.split('\t').collect();
        Job { path: c[ip].to_string(), size: c[is].parse().unwrap_or(0), verdict: iv.map(|i| c[i].to_string()).unwrap_or_default(),
              ctx: Ctx::default() }
    }).collect()
}

/// Parse the jobs (each from its own ctx) and merge every touched month into out/<month>.parquet.
/// Returns each job's new ctx (by path) for the state file. With a range, rows outside it are read and dropped.
pub fn run(mut jobs: Vec<Job>, out: &Path, agent: &str, threads: usize, jt_compat: bool, keep_parts: bool,
           range: Option<&crate::range::Range>) -> HashMap<String, Ctx> {
    let t0 = Instant::now();
    // LPT on the bytes still to read: the biggest pieces first, so none is left for last
    jobs.sort_by(|a, b| (b.size.saturating_sub(b.ctx.offset)).cmp(&(a.size.saturating_sub(a.ctx.offset))));
    let new_ctx: Mutex<HashMap<String, Ctx>> = Mutex::default();
    let parts = out.join("_parts");
    let _ = fs::remove_dir_all(&parts);
    fs::create_dir_all(&parts).unwrap();
    let n = if threads == 0 { std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8) } else { threads };
    let next = AtomicUsize::new(0);
    let st = Stats::default();
    let total: u64 = jobs.iter().map(|j| j.size.saturating_sub(j.ctx.offset)).sum();
    let months: Mutex<std::collections::BTreeSet<String>> = Mutex::default();
    TOTAL.store(total, Ordering::Relaxed);
    DONE.store(0, Ordering::Relaxed);
    std::thread::scope(|sc| {
        for tid in 0..n {
            let (jobs, next, st, parts, months, new_ctx) = (&jobs, &next, &st, &parts, &months, &new_ctx);
            sc.spawn(move || {
                let mut sp = Spiller { dir: parts.clone(), tid, n: 0, buf: HashMap::new() };
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(j) = jobs.get(i) else { break };
                    if crate::CANCEL.load(Ordering::Relaxed) {
                        break; // stopped: take no more files
                    }
                    if let Some(c) = parse_file(&j.path, &j.ctx, jt_compat, st, &mut |r| {
                        if range.is_some_and(|g| !g.keeps(&r.ts_th)) {
                            st.out_of_range.fetch_add(1, Ordering::Relaxed);
                            return;
                        }
                        st.rows.fetch_add(1, Ordering::Relaxed);
                        sp.push(r)
                    }) {
                        new_ctx.lock().unwrap().insert(j.path.clone(), c);
                    }
                }
                let m: Vec<String> = sp.buf.keys().cloned().collect();
                sp.flush();
                months.lock().unwrap().extend(m);
            });
        }
        // progress on stderr while the workers run
        let (st, next, nj) = (&st, &next, jobs.len());
        sc.spawn(move || {
            while next.load(Ordering::Relaxed) < nj {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let b = st.bytes.load(Ordering::Relaxed);
                DONE.store(b, Ordering::Relaxed);
                let frac = if total > 0 { b as f64 / total as f64 } else { 0.0 };
                // the bar shrinks on a narrow terminal, and the line is cut to its width (see ui::live)
                let bw = crate::ui::width().saturating_sub(80).clamp(8, 28);
                crate::ui::live(&format!("  {} {} ", crate::ui::pink("parse"), crate::ui::bar(frac, bw)), 10 + bw,
                                &format!("{:>3.0}%  {}/{} files · {:.1}/{:.1} GB · {:.1} GB/s · {} rows", frac * 100.0, st.files.load(Ordering::Relaxed),
                                         nj, b as f64 / 1e9, total as f64 / 1e9, b as f64 / 1e9 / t0.elapsed().as_secs_f64(), st.rows.load(Ordering::Relaxed)));
            }
        });
    });
    // stopped before the merge: nothing has been written to OUT yet, so throw the parts away and change nothing
    if crate::CANCEL.load(Ordering::Relaxed) {
        let _ = fs::remove_dir_all(&parts);
        eprintln!("\nparse: stopped; nothing written");
        return HashMap::new();
    }
    let t_parse = t0.elapsed().as_secs_f64();
    eprintln!("\nparse: {} files · {:.2} GB · {} lines ({} not JSON) · {} raw rows · {} stuck at a >8 MB line · {:.1}s ({:.0} MB/s) · {} threads",
              st.files.load(Ordering::Relaxed), st.bytes.load(Ordering::Relaxed) as f64 / 1e9, st.lines.load(Ordering::Relaxed),
              st.bad_json.load(Ordering::Relaxed), st.rows.load(Ordering::Relaxed), st.stuck.load(Ordering::Relaxed), t_parse,
              st.bytes.load(Ordering::Relaxed) as f64 / 1e6 / t_parse, n);
    if let Some(g) = range {
        eprintln!("parse: {} rows outside {} left out", st.out_of_range.load(Ordering::Relaxed), g.human());
    }
    DONE.store(total, Ordering::Relaxed);
    crate::PHASE.store(3, Ordering::Relaxed); // from here on a Stop waits: the month files are being replaced
    // merge: months in parallel (each holds only its own rows)
    let t1 = Instant::now();
    let ms: Vec<String> = fs::read_dir(&parts).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
    // only months this run wrote rows into are rewritten; every other month file is left as it is
    let done = Mutex::new(vec![]);
    let next = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..n.min(ms.len()).max(1) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(m) = ms.get(i) else { break };
                let (rows, copies) = merge_month(&parts, m, out, agent);
                done.lock().unwrap().push((m.clone(), rows, copies));
            });
        }
    });
    let mut done = done.into_inner().unwrap();
    done.sort();
    for (m, rows, copies) in &done {
        eprintln!("  {m}: {rows} rows ({copies} new copies merged) -> {}/{m}.parquet", out.display());
    }
    eprintln!("merge: {} months in {:.1}s · total {:.1}s", done.len(), t1.elapsed().as_secs_f64(), t0.elapsed().as_secs_f64());
    if !keep_parts {
        let _ = fs::remove_dir_all(&parts);
    }
    let _ = months;
    new_ctx.into_inner().unwrap()
}
