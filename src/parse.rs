//! Phase 2, `relic4-scan parse`: every indexed file -> rows -> ONE file per Bangkok month.
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
    pub file: String,
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

/// store.parse_chunk for a whole file, from line 0 with an empty context.
pub fn parse_file(path: &str, jt_compat: bool, st: &Stats, emit: &mut dyn FnMut(Raw)) {
    let mut data = Vec::new();
    if fs::File::open(path).and_then(|mut f| f.read_to_end(&mut data)).is_err() {
        return;
    }
    st.bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
    let data = complete(&data);
    let eng = engine::engine_of(path);
    let (mut nline, mut session, mut last): (i64, String, Option<i64>) = (0, String::new(), None);
    for line in split_lines(data) {
        if jt_compat && line.len() > JT_CHUNK {
            st.stuck.fetch_add(1, Ordering::Relaxed); // jt never ships a line longer than its chunk: the file stops here
            break;
        }
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
                name: None, body: None, err: None, red: (!kinds.is_empty()).then_some(kinds), file: path.to_string(), line: nline,
            });
        }
        for (idx, t) in trs.into_iter().enumerate() {
            let (clean, kinds) = engine::redact(&t.body);
            emit(Raw {
                month: stp.month.clone(), s: "tools", h, idx: idx as i64, engine: eng, role: None, typ: None,
                sid: sid.clone(), ts: stp.ts.clone(), ts_th: stp.ts_th.clone(), text: None, kind: Some(t.kind),
                call_id: Some(t.call_id), name: (!t.name.is_empty()).then_some(t.name), body: Some(clean), err: Some(t.err),
                red: (!kinds.is_empty()).then_some(kinds), file: path.to_string(), line: nline,
            });
        }
    }
    st.lines.fetch_add(nline as u64, Ordering::Relaxed);
    st.files.fetch_add(1, Ordering::Relaxed);
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
        st(&|r| Some(r.file.as_str())), Arc::new(rows.iter().map(|r| r.line).collect::<Int64Array>()),
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
        let m = r.month.clone();
        let v = self.buf.entry(m.clone()).or_default();
        v.push(r);
        if v.len() >= SPILL_ROWS {
            let rows = std::mem::take(v);
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

struct Kept {
    row: usize, // index into the month's rows of the copy that wins (first by file, line)
}

fn col_str<'a>(b: &'a RecordBatch, i: usize) -> &'a StringArray {
    b.column(i).as_any().downcast_ref::<StringArray>().unwrap()
}

fn s_opt(a: &StringArray, i: usize) -> Option<&str> {
    if a.is_null(i) { None } else { Some(a.value(i)) }
}

/// Fold one month's raw parts into the truth rows and write `out/<month>.parquet`. Returns (rows, copies).
pub fn merge_month(parts_dir: &Path, month: &str, out: &Path, agent: &str) -> (usize, usize) {
    let mut batches = vec![];
    for e in fs::read_dir(parts_dir.join(month)).unwrap().flatten() {
        let r = FileReader::try_new(fs::File::open(e.path()).unwrap(), None).unwrap();
        for b in r {
            batches.push(b.unwrap());
        }
    }
    let cat = arrow::compute::concat_batches(&raw_schema(), &batches).unwrap();
    drop(batches);
    let n = cat.num_rows();
    let (s, h, idx) = (col_str(&cat, 0), cat.column(1).as_any().downcast_ref::<Int64Array>().unwrap(),
                       cat.column(2).as_any().downcast_ref::<Int64Array>().unwrap());
    let (file, line) = (col_str(&cat, 16), cat.column(17).as_any().downcast_ref::<Int64Array>().unwrap());
    // first copy wins: lowest (file, line) per (s, h, idx)
    let mut keep: HashMap<(&str, i64, i64), Kept> = HashMap::with_capacity(n);
    // sources: one per (h, agent, file), its first line
    let mut srcs: HashMap<i64, BTreeMap<&str, i64>> = HashMap::new();
    for i in 0..n {
        let k = (s.value(i), h.value(i), idx.value(i));
        let better = match keep.get(&k) {
            None => true,
            Some(o) => (file.value(i), line.value(i)) < (file.value(o.row), line.value(o.row)),
        };
        if better {
            keep.insert(k, Kept { row: i });
        }
        let e = srcs.entry(h.value(i)).or_default().entry(file.value(i)).or_insert(line.value(i));
        if line.value(i) < *e {
            *e = line.value(i);
        }
    }
    let ts = col_str(&cat, 7);
    let mut order: Vec<usize> = keep.values().map(|k| k.row).collect();
    order.sort_by(|&a, &b| (ts.value(a), s.value(a), h.value(a), idx.value(a)).cmp(&(ts.value(b), s.value(b), h.value(b), idx.value(b))));

    // build the truth columns
    let strs = |c: usize| -> ArrayRef {
        let a = col_str(&cat, c);
        Arc::new(order.iter().map(|&i| s_opt(a, i)).collect::<StringArray>())
    };
    let mut hb = Int64Builder::new();
    let mut ib = Int64Builder::new();
    let mut eb = BooleanBuilder::new();
    let errs = cat.column(14).as_any().downcast_ref::<BooleanArray>().unwrap();
    for &i in &order {
        hb.append_value(h.value(i));
        ib.append_value(idx.value(i));
        if errs.is_null(i) { eb.append_null() } else { eb.append_value(errs.value(i)) }
    }
    let mut machines = ListBuilder::new(StringBuilder::new());
    let mut sources = ListBuilder::new(StringBuilder::new());
    for &i in &order {
        machines.values().append_value(agent);
        machines.append(true);
        for (f, l) in &srcs[&h.value(i)] {
            sources.values().append_value(format!("{agent}:{f}:{l}"));
        }
        sources.append(true);
    }
    let null_str: ArrayRef = Arc::new(StringArray::from(vec![None::<&str>; order.len()]));
    let _ = null_str;
    let u = |n: &str| Field::new(n, DataType::Utf8, true);
    let list = |n: &str| Field::new(n, DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))), true);
    let schema = Arc::new(Schema::new(vec![
        u("s"), Field::new("h", DataType::Int64, false), Field::new("idx", DataType::Int64, false), u("engine"), u("role"),
        u("type"), u("sid"), u("ts"), u("ts_th"), u("text"), u("kind"), u("call_id"), u("name"), u("body"),
        Field::new("err", DataType::Boolean, true), u("red"), list("machines"), list("sources"),
    ]));
    let cols: Vec<ArrayRef> = vec![
        strs(0), Arc::new(hb.finish()), Arc::new(ib.finish()), strs(3), strs(4), strs(5), strs(6), strs(7), strs(8), strs(9),
        strs(10), strs(11), strs(12), strs(13), Arc::new(eb.finish()), strs(15), Arc::new(machines.finish()), Arc::new(sources.finish()),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols).unwrap();
    fs::create_dir_all(out).unwrap();
    let tmp = out.join(format!("{month}.parquet.part"));
    let props = WriterProperties::builder().set_compression(Compression::ZSTD(ZstdLevel::try_new(1).unwrap())).build();
    let mut w = ArrowWriter::try_new(fs::File::create(&tmp).unwrap(), schema, Some(props)).unwrap();
    w.write(&batch).unwrap();
    w.close().unwrap();
    fs::rename(&tmp, out.join(format!("{month}.parquet"))).unwrap();
    (order.len(), n)
}

// ---------- driver ----------

pub struct Job {
    pub path: String,
    pub size: u64,
    pub verdict: String,
}

pub fn read_index(p: &str) -> Vec<Job> {
    let text = fs::read_to_string(p).expect("read index");
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().unwrap_or("").split('\t').collect();
    let (ip, is) = (head.iter().position(|c| *c == "path").unwrap(), head.iter().position(|c| *c == "size").unwrap());
    let iv = head.iter().position(|c| *c == "verdict");
    lines.map(|l| {
        let c: Vec<&str> = l.split('\t').collect();
        Job { path: c[ip].to_string(), size: c[is].parse().unwrap_or(0), verdict: iv.map(|i| c[i].to_string()).unwrap_or_default() }
    }).collect()
}

pub fn run(mut jobs: Vec<Job>, out: &Path, agent: &str, threads: usize, jt_compat: bool, keep_parts: bool) {
    let t0 = Instant::now();
    jobs.sort_by(|a, b| b.size.cmp(&a.size)); // LPT: the biggest files first, so none is left for last
    let parts = out.join("_parts");
    let _ = fs::remove_dir_all(&parts);
    fs::create_dir_all(&parts).unwrap();
    let n = if threads == 0 { std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8) } else { threads };
    let next = AtomicUsize::new(0);
    let st = Stats::default();
    let total: u64 = jobs.iter().map(|j| j.size).sum();
    let months: Mutex<std::collections::BTreeSet<String>> = Mutex::default();
    std::thread::scope(|sc| {
        for tid in 0..n {
            let (jobs, next, st, parts, months) = (&jobs, &next, &st, &parts, &months);
            sc.spawn(move || {
                let mut sp = Spiller { dir: parts.clone(), tid, n: 0, buf: HashMap::new() };
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(j) = jobs.get(i) else { break };
                    parse_file(&j.path, jt_compat, st, &mut |r| {
                        st.rows.fetch_add(1, Ordering::Relaxed);
                        sp.push(r)
                    });
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
                eprint!("\r  parse {}/{} files · {:.1}/{:.1} GB · {:.0} MB/s · {} rows   ",
                        st.files.load(Ordering::Relaxed), nj, b as f64 / 1e9, total as f64 / 1e9,
                        b as f64 / 1e6 / t0.elapsed().as_secs_f64(), st.rows.load(Ordering::Relaxed));
            }
        });
    });
    let t_parse = t0.elapsed().as_secs_f64();
    eprintln!("\nparse: {} files · {:.2} GB · {} lines ({} not JSON) · {} raw rows · {} stuck at a >8 MB line · {:.1}s ({:.0} MB/s) · {} threads",
              st.files.load(Ordering::Relaxed), st.bytes.load(Ordering::Relaxed) as f64 / 1e9, st.lines.load(Ordering::Relaxed),
              st.bad_json.load(Ordering::Relaxed), st.rows.load(Ordering::Relaxed), st.stuck.load(Ordering::Relaxed), t_parse,
              st.bytes.load(Ordering::Relaxed) as f64 / 1e6 / t_parse, n);
    // merge: months in parallel (each holds only its own rows)
    let t1 = Instant::now();
    let ms: Vec<String> = fs::read_dir(&parts).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
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
        eprintln!("  {m}: {rows} rows from {copies} copies -> {}/{m}.parquet", out.display());
    }
    eprintln!("merge: {} months in {:.1}s · total {:.1}s", done.len(), t1.elapsed().as_secs_f64(), t0.elapsed().as_secs_f64());
    if !keep_parts {
        let _ = fs::remove_dir_all(&parts);
    }
    let _ = months;
}
