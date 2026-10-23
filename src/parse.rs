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
}

/// store.parse_chunk from `ctx.offset` with `ctx`, to the end of the file's complete lines. Returns the new ctx.
/// How much of a file is read at a time (RELIC_SCAN_CHUNK bytes, default 32 MiB). The biggest files are parsed
/// first and side by side, so reading each one whole held the top N files in memory at once (3.3 GB on m5).
fn chunk_size() -> usize {
    static C: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *C.get_or_init(|| std::env::var("RELIC_SCAN_CHUNK").ok().and_then(|v| v.parse().ok()).filter(|&n| n > 0).unwrap_or(32 << 20))
}

pub fn parse_file(path: &str, ctx: &Ctx, jt_compat: bool, st: &Stats, emit: &mut dyn FnMut(Raw)) -> Option<Ctx> {
    parse_file_chunked(path, ctx, jt_compat, st, emit, chunk_size())
}

fn parse_file_chunked(path: &str, ctx: &Ctx, jt_compat: bool, st: &Stats, emit: &mut dyn FnMut(Raw), want: usize) -> Option<Ctx> {
    use std::io::{Seek, SeekFrom};
    let mut f = fs::File::open(path).ok()?;
    f.seek(SeekFrom::Start(ctx.offset)).ok()?;
    let eng = engine::engine_of(path);
    let file: Arc<str> = Arc::from(path);
    let (mut nline, mut session, mut last) = (ctx.lines, ctx.session.clone(), ctx.last_ms);
    let mut used: u64 = 0;
    // read a chunk, parse up to its last "\n", carry the rest into the next chunk; at the end of the file the rest
    // is cut as a whole-file read would cut it (complete()), so the rows are the same for any chunk size
    let (mut buf, mut eof, mut stop) = (Vec::new(), false, false);
    while !eof && !stop {
        let n = Read::by_ref(&mut f).take(want as u64).read_to_end(&mut buf).ok()?;
        st.bytes.fetch_add(n as u64, Ordering::Relaxed);
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
            if jt_compat && line.len() > JT_CHUNK {
                st.stuck.fetch_add(1, Ordering::Relaxed); // jt never ships a line longer than its chunk: the file stops here
                stop = true;
                break;
            }
            used += line.len() as u64;
            nline += 1;
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
            if x.text.is_empty() && think.is_empty() && trs.is_empty() {
                continue;
            }
            let stp = engine::stamp(ts);
            let sid = if x.sess.is_empty() { session.clone() } else { x.sess.clone() };
            let h = engine::key(line);
            for (s, body) in [("text", &x.text), ("thinking", &think)] {
                if body.is_empty() {
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
        }
        buf.drain(..took);
    }
    st.lines.fetch_add((nline - ctx.lines) as u64, Ordering::Relaxed);
    st.files.fetch_add(1, Ordering::Relaxed);
    Some(Ctx { offset: ctx.offset + used, lines: nline, session, last_ms: last })
}

// ---------- arrow: raw parts ----------

fn raw_schema() -> Arc<Schema> {
    let u = |n: &str| Field::new(n, DataType::Utf8, true);
    Arc::new(Schema::new(vec![
        u("s"), Field::new("h", DataType::Int64, false), Field::new("idx", DataType::Int64, false), u("engine"), u("role"),
        u("type"), u("sid"), u("ts"), u("ts_th"), u("text"), u("kind"), u("call_id"), u("name"), u("body"),
        Field::new("err", DataType::Boolean, true), u("red"), u("file"), Field::new("line", DataType::Int64, false),
    ]))
}

fn raw_batch(rows: &[Raw]) -> RecordBatch {
    let st = |f: &dyn Fn(&Raw) -> Option<&str>| -> ArrayRef { Arc::new(rows.iter().map(|r| f(r)).collect::<StringArray>()) };
    let cols: Vec<ArrayRef> = vec![
        st(&|r| Some(r.s)), Arc::new(rows.iter().map(|r| r.h).collect::<Int64Array>()),
        Arc::new(rows.iter().map(|r| r.idx).collect::<Int64Array>()), st(&|r| Some(r.engine)), st(&|r| r.role.as_deref()),
        st(&|r| r.typ.as_deref()), st(&|r| Some(r.sid.as_str())), st(&|r| Some(r.ts.as_str())), st(&|r| Some(r.ts_th.as_str())),
        st(&|r| r.text.as_deref()), st(&|r| r.kind), st(&|r| r.call_id.as_deref()), st(&|r| r.name.as_deref()),
        st(&|r| r.body.as_deref()), Arc::new(rows.iter().map(|r| r.err).collect::<BooleanArray>()), st(&|r| r.red.as_deref()),
        st(&|r| Some(&*r.file)), Arc::new(rows.iter().map(|r| r.line).collect::<Int64Array>()),
    ];
    RecordBatch::try_new(raw_schema(), cols).expect("raw batch")
}

struct Spiller {
    dir: PathBuf,
    tid: usize,
    n: usize,
    buf: HashMap<String, Vec<Raw>>,
}

impl Spiller {
    fn push(&mut self, r: Raw) {
        let full = match self.buf.get_mut(&r.month) {
            Some(v) => {
                v.push(r);
                (v.len() >= SPILL_ROWS).then(|| std::mem::take(v))
            }
            None => {
                self.buf.insert(r.month.clone(), vec![r]);
                None
            }
        };
        if let Some(rows) = full {
            let m = rows[0].month.clone();
            self.write(&m, &rows);
        }
    }
    fn write(&mut self, month: &str, rows: &[Raw]) {
        let d = self.dir.join(month);
        fs::create_dir_all(&d).unwrap();
        let p = d.join(format!("t{:02}-{:05}.arrow", self.tid, self.n));
        self.n += 1;
        let f = fs::File::create(&p).unwrap();
        let mut w = FileWriter::try_new(f, &raw_schema()).unwrap();
        w.write(&raw_batch(rows)).unwrap();
        w.finish().unwrap();
    }
    fn flush(&mut self) {
        let all: Vec<(String, Vec<Raw>)> = self.buf.drain().collect();
        for (m, rows) in all {
            if !rows.is_empty() {
                self.write(&m, &rows);
            }
        }
    }
}

// ---------- merge: one month -> one file ----------

/// One candidate copy of a truth row, from the existing month file or from a new raw part.
struct Cand {
    s: String, h: i64, idx: i64, engine: Option<String>, role: Option<String>, typ: Option<String>, sid: Option<String>,
    ts: Option<String>, ts_th: Option<String>, text: Option<String>, kind: Option<String>, call_id: Option<String>,
    name: Option<String>, body: Option<String>, err: Option<bool>, red: Option<String>,
    first: (String, i64), // the copy's (file, line): the lowest wins, so incremental == cold
}

fn col_str<'a>(b: &'a RecordBatch, name: &str) -> &'a StringArray {
    b.column_by_name(name).unwrap().as_any().downcast_ref::<StringArray>().unwrap()
}

fn s_opt(a: &StringArray, i: usize) -> Option<String> {
    if a.is_null(i) { None } else { Some(a.value(i).to_string()) }
}

fn cands_from(b: &RecordBatch, srcs: &mut HashMap<i64, BTreeMap<String, i64>>, agent: &str, out: &mut Vec<Cand>) {
    let (h, idx) = (b.column_by_name("h").unwrap().as_any().downcast_ref::<Int64Array>().unwrap(),
                    b.column_by_name("idx").unwrap().as_any().downcast_ref::<Int64Array>().unwrap());
    let err = b.column_by_name("err").unwrap().as_any().downcast_ref::<BooleanArray>().unwrap();
    let c = |n: &str| col_str(b, n);
    let (s, eng, role, typ, sid, ts, tsth, text, kind, cid, name, body, red) =
        (c("s"), c("engine"), c("role"), c("type"), c("sid"), c("ts"), c("ts_th"), c("text"), c("kind"), c("call_id"), c("name"),
         c("body"), c("red"));
    let raw = b.column_by_name("file").is_some();
    let prefix = format!("{agent}:");
    for i in 0..b.num_rows() {
        let hv = h.value(i);
        let first = if raw {
            let (f, l) = (c("file").value(i).to_string(), b.column_by_name("line").unwrap().as_any().downcast_ref::<Int64Array>().unwrap().value(i));
            let e = srcs.entry(hv).or_default().entry(f.clone()).or_insert(l);
            if l < *e {
                *e = l;
            }
            (f, l)
        } else {
            // an existing month row: its sources list holds every copy; its own copy is the lowest (file, line)
            let list = b.column_by_name("sources").unwrap().as_any().downcast_ref::<arrow::array::ListArray>().unwrap().value(i);
            let list = list.as_any().downcast_ref::<StringArray>().unwrap();
            let mut best: Option<(String, i64)> = None;
            for j in 0..list.len() {
                let v = list.value(j);
                let Some(rest) = v.strip_prefix(&prefix) else { continue };
                let Some((f, l)) = rest.rsplit_once(':') else { continue };
                let l: i64 = l.parse().unwrap_or(i64::MAX);
                let e = srcs.entry(hv).or_default().entry(f.to_string()).or_insert(l);
                if l < *e {
                    *e = l;
                }
                if best.as_ref().is_none_or(|b| (f, l) < (b.0.as_str(), b.1)) {
                    best = Some((f.to_string(), l));
                }
            }
            best.unwrap_or((String::new(), i64::MAX))
        };
        out.push(Cand {
            s: s.value(i).to_string(), h: hv, idx: idx.value(i), engine: s_opt(eng, i), role: s_opt(role, i), typ: s_opt(typ, i),
            sid: s_opt(sid, i), ts: s_opt(ts, i), ts_th: s_opt(tsth, i), text: s_opt(text, i), kind: s_opt(kind, i),
            call_id: s_opt(cid, i), name: s_opt(name, i), body: s_opt(body, i), err: if err.is_null(i) { None } else { Some(err.value(i)) },
            red: s_opt(red, i), first,
        });
    }
}

/// Fold one month: the existing out/<month>.parquet (if any) + this run's raw parts -> out/<month>.parquet.
/// Returns (rows, new copies read).
pub fn merge_month(parts_dir: &Path, month: &str, out: &Path, agent: &str) -> (usize, usize) {
    let mut cands: Vec<Cand> = vec![];
    let mut srcs: HashMap<i64, BTreeMap<String, i64>> = HashMap::new();
    let existing = out.join(format!("{month}.parquet"));
    if existing.exists() {
        let r = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(fs::File::open(&existing).unwrap())
            .unwrap().build().unwrap();
        for b in r {
            cands_from(&b.unwrap(), &mut srcs, agent, &mut cands);
        }
    }
    let had = cands.len();
    if let Ok(rd) = fs::read_dir(parts_dir.join(month)) {
        for e in rd.flatten() {
            for b in FileReader::try_new(fs::File::open(e.path()).unwrap(), None).unwrap() {
                cands_from(&b.unwrap(), &mut srcs, agent, &mut cands);
            }
        }
    }
    let copies = cands.len() - had;
    // first copy wins: lowest (file, line) per (s, h, idx)
    let mut keep: HashMap<(String, i64, i64), usize> = HashMap::with_capacity(cands.len());
    for (i, c) in cands.iter().enumerate() {
        let k = (c.s.clone(), c.h, c.idx);
        match keep.get(&k) {
            Some(&o) if cands[o].first <= c.first => {}
            _ => {
                keep.insert(k, i);
            }
        }
    }
    let mut order: Vec<usize> = keep.into_values().collect();
    order.sort_by(|&a, &b| {
        let (x, y) = (&cands[a], &cands[b]);
        (x.ts.as_deref(), x.s.as_str(), x.h, x.idx).cmp(&(y.ts.as_deref(), y.s.as_str(), y.h, y.idx))
    });
    let strs = |f: &dyn Fn(&Cand) -> Option<&str>| -> ArrayRef { Arc::new(order.iter().map(|&i| f(&cands[i])).collect::<StringArray>()) };
    let mut hb = Int64Builder::new();
    let mut ib = Int64Builder::new();
    let mut eb = BooleanBuilder::new();
    let mut machines = ListBuilder::new(StringBuilder::new());
    let mut sources = ListBuilder::new(StringBuilder::new());
    for &i in &order {
        let c = &cands[i];
        hb.append_value(c.h);
        ib.append_value(c.idx);
        eb.append_option(c.err);
        machines.values().append_value(agent);
        machines.append(true);
        for (f, l) in &srcs[&c.h] {
            sources.values().append_value(format!("{agent}:{f}:{l}"));
        }
        sources.append(true);
    }
    let u = |n: &str| Field::new(n, DataType::Utf8, true);
    let list = |n: &str| Field::new(n, DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))), true);
    let schema = Arc::new(Schema::new(vec![
        u("s"), Field::new("h", DataType::Int64, false), Field::new("idx", DataType::Int64, false), u("engine"), u("role"),
        u("type"), u("sid"), u("ts"), u("ts_th"), u("text"), u("kind"), u("call_id"), u("name"), u("body"),
        Field::new("err", DataType::Boolean, true), u("red"), list("machines"), list("sources"),
    ]));
    let cols: Vec<ArrayRef> = vec![
        strs(&|c| Some(c.s.as_str())), Arc::new(hb.finish()), Arc::new(ib.finish()), strs(&|c| c.engine.as_deref()),
        strs(&|c| c.role.as_deref()), strs(&|c| c.typ.as_deref()), strs(&|c| c.sid.as_deref()), strs(&|c| c.ts.as_deref()),
        strs(&|c| c.ts_th.as_deref()), strs(&|c| c.text.as_deref()), strs(&|c| c.kind.as_deref()), strs(&|c| c.call_id.as_deref()),
        strs(&|c| c.name.as_deref()), strs(&|c| c.body.as_deref()), Arc::new(eb.finish()), strs(&|c| c.red.as_deref()),
        Arc::new(machines.finish()), Arc::new(sources.finish()),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols).unwrap();
    fs::create_dir_all(out).unwrap();
    let tmp = out.join(format!("{month}.parquet.part"));
    let props = WriterProperties::builder().set_compression(Compression::ZSTD(ZstdLevel::try_new(1).unwrap())).build();
    let mut w = ArrowWriter::try_new(fs::File::create(&tmp).unwrap(), schema, Some(props)).unwrap();
    w.write(&batch).unwrap();
    w.into_inner().unwrap().sync_all().unwrap(); // the month is on disk before _state.tsv can point past it
    fs::rename(&tmp, &existing).unwrap(); // atomic: readers see the old month or the new one, never half
    (order.len(), copies)
}

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
                eprint!("\r  {} {} {:>3.0}%  {}/{} files · {:.1}/{:.1} GB · {:.1} GB/s · {} rows   ", crate::ui::pink("parse"),
                        crate::ui::bar(frac, 28), frac * 100.0, st.files.load(Ordering::Relaxed), nj, b as f64 / 1e9, total as f64 / 1e9,
                        b as f64 / 1e9 / t0.elapsed().as_secs_f64(), st.rows.load(Ordering::Relaxed));
            }
        });
    });
    let t_parse = t0.elapsed().as_secs_f64();
    eprintln!("\nparse: {} files · {:.2} GB · {} lines ({} not JSON) · {} raw rows · {} stuck at a >8 MB line · {:.1}s ({:.0} MB/s) · {} threads",
              st.files.load(Ordering::Relaxed), st.bytes.load(Ordering::Relaxed) as f64 / 1e9, st.lines.load(Ordering::Relaxed),
              st.bad_json.load(Ordering::Relaxed), st.rows.load(Ordering::Relaxed), st.stuck.load(Ordering::Relaxed), t_parse,
              st.bytes.load(Ordering::Relaxed) as f64 / 1e6 / t_parse, n);
    if let Some(g) = range {
        eprintln!("parse: {} rows outside {} left out", st.out_of_range.load(Ordering::Relaxed), g.human());
    }
    DONE.store(total, Ordering::Relaxed);
    crate::PHASE.store(3, Ordering::Relaxed);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, empty directory under the system temp dir.
    pub fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("relic-scan-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// Claude-shaped lines: user text, assistant text + thinking + a tool call, a tool result, a line with no
    /// time, a broken line, CRLF and a lone CR ending.
    pub fn lines(day: u32, n: usize) -> String {
        let mut s = String::new();
        for i in 0..n {
            let ts = format!("2026-07-{day:02}T0{}:{:02}:00.000Z", i % 10, i % 60);
            s += &format!(r#"{{"type":"user","sessionId":"s1","timestamp":"{ts}","message":{{"role":"user","content":"hello {i}"}}}}"#);
            s += "\n";
            s += &format!(r#"{{"type":"assistant","sessionId":"s1","timestamp":"{ts}","message":{{"role":"assistant","content":[{{"type":"thinking","thinking":"hmm {i}"}},{{"type":"text","text":"hi {i}"}},{{"type":"tool_use","id":"t{day}-{i}","name":"Bash","input":{{"command":"ls"}}}}]}}}}"#);
            s += "\r\n";
            s += &format!(r#"{{"type":"user","sessionId":"s1","timestamp":"{ts}","message":{{"role":"user","content":[{{"type":"tool_result","tool_use_id":"t{day}-{i}","content":"ok {i}","is_error":{}}}]}}}}"#, i % 3 == 0);
            s += "\r";
            s += &format!(r#"{{"type":"user","sessionId":"s1","message":{{"role":"user","content":"no time {i}"}}}}"#);
            s += "\n{not json\n";
        }
        s
    }

    fn rows_of(path: &str, chunk: usize) -> (Vec<String>, Ctx) {
        let st = Stats::default();
        let mut v = vec![];
        let c = parse_file_chunked(path, &Ctx::default(), false, &st, &mut |r| {
            v.push(format!("{}|{}|{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}|{}", r.month, r.s, r.h, r.idx, r.role, r.text, r.kind, r.body, r.err, r.line))
        }, chunk).unwrap();
        (v, c)
    }

    #[test]
    fn split_lines_like_python() {
        assert_eq!(split_lines(b"a\nb\r\nc\rd"), vec![&b"a\n"[..], b"b\r\n", b"c\r", b"d"]);
        assert_eq!(split_lines(b""), Vec::<&[u8]>::new());
        assert_eq!(split_lines(b"\n\n"), vec![&b"\n"[..], b"\n"]);
    }

    #[test]
    fn complete_keeps_a_json_tail_only() {
        assert_eq!(complete(b"{}\n{\"a\":1}"), b"{}\n{\"a\":1}");
        assert_eq!(complete(b"{}\n{\"a\":"), b"{}\n");
        assert_eq!(complete(b"{\"a\":"), b"");
        assert_eq!(complete(b"{}"), b"{}");
    }

    #[test]
    fn any_chunk_size_gives_the_same_rows() {
        let d = tmp("chunks");
        let p = d.join("a.jsonl");
        // a half-written last line (not JSON): it must be left for the next run at every chunk size
        fs::write(&p, lines(1, 40) + r#"{"type":"user","sessionId":"s1""#).unwrap();
        let p = p.to_str().unwrap();
        let (whole, c0) = rows_of(p, usize::MAX >> 1);
        assert!(whole.len() > 100, "the fixture must produce rows, got {}", whole.len());
        for chunk in [1, 2, 3, 7, 64, 333, 4096] {
            let (v, c) = rows_of(p, chunk);
            assert_eq!(v, whole, "chunk {chunk}");
            assert_eq!((c.offset, c.lines, &c.session, c.last_ms), (c0.offset, c0.lines, &c0.session, c0.last_ms), "chunk {chunk}");
        }
        let _ = fs::remove_dir_all(&d);
    }

    /// Every row of a month file, as text, in file order.
    pub fn month_rows(p: &Path) -> Vec<String> {
        let r = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(fs::File::open(p).unwrap()).unwrap().build().unwrap();
        let mut out = vec![];
        for b in r {
            let b = b.unwrap();
            for i in 0..b.num_rows() {
                let cols: Vec<String> = b.columns().iter()
                    .map(|c| arrow::util::display::ArrayFormatter::try_new(c.as_ref(), &Default::default()).unwrap().value(i).to_string())
                    .collect();
                out.push(cols.join("|"));
            }
        }
        out
    }

    #[test]
    fn incremental_equals_cold() {
        let d = tmp("incr");
        let src = d.join("s.jsonl");
        let sp = src.to_str().unwrap().to_string();
        let job = |ctx: Ctx| Job { path: sp.clone(), size: fs::metadata(&src).unwrap().len(), verdict: String::new(), ctx };
        // incremental: three runs over a growing file (the second cut mid-line)
        let full = lines(2, 30) + &lines(3, 30);
        let cut = full.len() / 2;
        fs::write(&src, &full[..cut]).unwrap();
        let inc = d.join("inc");
        let c1 = run(vec![job(Ctx::default())], &inc, "t@x", 2, false, false, None)[&sp].clone();
        assert!(c1.offset <= cut as u64 && c1.offset > 0);
        fs::write(&src, &full[..cut + 1000]).unwrap();
        let c2 = run(vec![job(c1)], &inc, "t@x", 2, false, false, None)[&sp].clone();
        fs::write(&src, &full).unwrap();
        let c3 = run(vec![job(c2)], &inc, "t@x", 2, false, false, None)[&sp].clone();
        // cold: the whole file at once
        let cold = d.join("cold");
        let c = run(vec![job(Ctx::default())], &cold, "t@x", 2, false, false, None)[&sp].clone();
        assert_eq!((c3.offset, c3.lines, c3.last_ms), (c.offset, c.lines, c.last_ms));
        let mut months: Vec<String> = fs::read_dir(&cold).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".parquet")).collect();
        months.sort();
        assert!(!months.is_empty());
        for m in months {
            let (a, b) = (month_rows(&inc.join(&m)), month_rows(&cold.join(&m)));
            assert!(!b.is_empty());
            assert_eq!(a, b, "{m}");
        }
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn a_range_keeps_exactly_its_days() {
        let d = tmp("range-rows");
        let src = d.join("s.jsonl");
        // each day's lines unique: two identical lines are one row (the first copy), so a day would lose its own
        let day_lines = |d: u32| lines(d, 30).replace("no time ", &format!("no time {d}-"));
        fs::write(&src, day_lines(2) + &day_lines(3) + &day_lines(4)).unwrap();
        let sp = src.to_str().unwrap().to_string();
        let job = || Job { path: sp.clone(), size: fs::metadata(&src).unwrap().len(), verdict: String::new(), ctx: Ctx::default() };
        let (all, day) = (d.join("all"), d.join("day"));
        let c_all = run(vec![job()], &all, "t@x", 2, false, false, None)[&sp].clone();
        let g = crate::range::Range::from_args(None, Some("2026-07-03"), None, None).unwrap().unwrap();
        let c_day = run(vec![job()], &day, "t@x", 2, false, false, Some(&g))[&sp].clone();
        assert_eq!(c_day.offset, c_all.offset, "a range still reads the whole file, so the next run starts at its end");
        let want: Vec<String> = month_rows(&all.join("2026-07.parquet")).into_iter().filter(|r| r.contains("2026-07-03T")).collect();
        assert!(!want.is_empty());
        assert_eq!(month_rows(&day.join("2026-07.parquet")), want);
        let _ = fs::remove_dir_all(&d);
    }
}
