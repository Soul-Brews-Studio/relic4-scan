//! `relic-scan map --out DIR`: meta/sessions.parquet + meta/files.parquet -> DIR/meta/map.html, one self-contained
//! page (data inlined, no network) that draws every session as a star and answers queries in the browser.
//!
//!   ring    = the Bangkok month the session started (inside = oldest)
//!   sector  = the repo (cat), wider for repos with more sessions
//!   star    = a main session, size = log(rows); subagents and workflow agents orbit their parent
//!   query   = words match repo / project / sid; tier:main  repo:neo  home:mirror  month:2026-07  rows>1000 …

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use arrow::array::{Array, Int64Array, RecordBatch, StringArray};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde_json::{Value, json};

const PAGE: &str = include_str!("map.html");

fn read(p: &Path) -> Vec<RecordBatch> {
    let f = fs::File::open(p).unwrap_or_else(|e| panic!("{}: {e} (run `relic-scan run` first)", p.display()));
    ParquetRecordBatchReaderBuilder::try_new(f).unwrap().with_batch_size(65_536).build().unwrap().map(|b| b.unwrap()).collect()
}

fn s<'a>(b: &'a RecordBatch, n: &str) -> &'a StringArray {
    b.column_by_name(n).unwrap().as_any().downcast_ref::<StringArray>().unwrap()
}

fn i<'a>(b: &'a RecordBatch, n: &str) -> &'a Int64Array {
    b.column_by_name(n).unwrap().as_any().downcast_ref::<Int64Array>().unwrap()
}

fn get<'a>(a: &'a StringArray, k: usize) -> Option<&'a str> {
    if a.is_null(k) { None } else { Some(a.value(k)) }
}

/// "2026-07-01T12:34:56.789Z" -> epoch seconds
fn epoch(ts: Option<&str>) -> Value {
    ts.and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok()).map(|d| json!(d.timestamp())).unwrap_or(Value::Null)
}

/// A string column as (dictionary, index per row): the page stays small with 40k+ rows.
#[derive(Default)]
struct Dict {
    words: Vec<String>,
    at: HashMap<String, usize>,
}

impl Dict {
    fn id(&mut self, w: Option<&str>) -> Value {
        let Some(w) = w else { return json!(-1) };
        if let Some(&k) = self.at.get(w) {
            return json!(k);
        }
        self.words.push(w.to_string());
        self.at.insert(w.to_string(), self.words.len() - 1);
        json!(self.words.len() - 1)
    }
}

pub fn build(out: &Path, agent: &str) {
    let t0 = std::time::Instant::now();
    let dir = out.join("meta");
    // the home the scan found each file in (files.parquet's home is a path rule, empty for shape homes)
    let mut scan_home: HashMap<String, String> = HashMap::new();
    if dir.join("jsonl.parquet").exists() {
        for b in read(&dir.join("jsonl.parquet")) {
            let (path, home) = (s(&b, "path"), s(&b, "home"));
            for k in 0..b.num_rows() {
                if let Some(h) = get(home, k) {
                    scan_home.insert(path.value(k).to_string(), h.to_string());
                }
            }
        }
    }
    // homes, and which homes hold each session (base, tier)
    let mut homes: Vec<(String, u64, u64, i64)> = vec![]; // path, files, bytes, rows
    let mut home_at: HashMap<String, usize> = HashMap::new();
    let mut sess_homes: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for b in read(&dir.join("files.parquet")) {
        let (path, home, base, tier, size, rows) = (s(&b, "path"), s(&b, "home"), s(&b, "base"), s(&b, "tier"), i(&b, "size"), i(&b, "n_rows"));
        for k in 0..b.num_rows() {
            let h = scan_home.get(path.value(k)).cloned().or_else(|| get(home, k).filter(|h| !h.is_empty()).map(str::to_string)).unwrap_or_else(|| {
                let p = path.value(k);
                p[..p.rfind('/').unwrap_or(0)].to_string()
            });
            let hi = *home_at.entry(h.clone()).or_insert_with(|| {
                homes.push((h, 0, 0, 0));
                homes.len() - 1
            });
            homes[hi].1 += 1;
            homes[hi].2 += size.value(k) as u64;
            homes[hi].3 += rows.value(k);
            let v = sess_homes.entry((base.value(k).to_string(), tier.value(k).to_string())).or_default();
            if !v.contains(&hi) {
                v.push(hi);
            }
        }
    }
    let (mut cats, mut projs) = (Dict::default(), Dict::default());
    let mut cols: HashMap<&str, Vec<Value>> = HashMap::new();
    let mut sid_main: HashMap<String, usize> = HashMap::new();
    let mut parents: Vec<Option<String>> = vec![];
    let mut n = 0usize;
    for b in read(&dir.join("sessions.parquet")) {
        let (base, tier, sid, parent, cat, proj, f, l) = (s(&b, "base"), s(&b, "tier"), s(&b, "sid"), s(&b, "parent_sid"), s(&b, "cat"),
                                                       s(&b, "proj"), s(&b, "first_ts"), s(&b, "last_ts"));
        let (rows, text, msgs, copies, size) = (i(&b, "n_rows"), i(&b, "text_rows"), i(&b, "text_msgs"), i(&b, "copies"), i(&b, "size"));
        for k in 0..b.num_rows() {
            let t = tier.value(k);
            // a subagent's parent_sid is the folder its file sits in, <main-uuid>/subagents/: the main FILE's name,
            // not the sessionId inside it (a resumed session's first line carries the earlier session's id)
            if t == "main" {
                sid_main.insert(base.value(k).trim_end_matches(".jsonl").to_string(), n);
            }
            parents.push(get(parent, k).map(str::to_string));
            let mut push = |c: &'static str, v: Value| cols.entry(c).or_default().push(v);
            push("sid", json!(get(sid, k).unwrap_or(base.value(k))));
            push("tier", json!(match t { "main" => 0, "subagent" => 1, _ => 2 }));
            push("cat", cats.id(get(cat, k)));
            push("proj", projs.id(get(proj, k)));
            push("t0", epoch(get(f, k)));
            push("t1", epoch(get(l, k)));
            push("rows", json!(rows.value(k)));
            push("text", json!(text.value(k)));
            push("msgs", json!(msgs.value(k)));
            push("copies", json!(copies.value(k)));
            push("bytes", json!(size.value(k)));
            for (j, ty) in crate::meta::TYPES.iter().enumerate() {
                push(["user", "asst", "think", "calls", "results", "errors"][j], json!(i(&b, ty).value(k)));
            }
            push("homes", json!(sess_homes.get(&(base.value(k).to_string(), t.to_string())).cloned().unwrap_or_default()));
            n += 1;
        }
    }
    let parent: Vec<Value> = parents.iter().map(|p| json!(p.as_ref().and_then(|p| sid_main.get(p)).map(|&x| x as i64).unwrap_or(-1))).collect();
    cols.insert("parent", parent);
    let data = json!({
        "agent": agent,
        "built": chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        "cats": cats.words, "projs": projs.words,
        "homes": homes.iter().map(|(p, f, b, r)| json!({"path": p, "origin": crate::scan::origin(p), "files": f, "bytes": b, "rows": r}))
            .collect::<Vec<_>>(),
        "s": cols,
    });
    // inline: "</" would end the <script> early
    let js = serde_json::to_string(&data).unwrap().replace("</", "<\\/");
    let page = PAGE.replace("/*DATA*/null", &js);
    let p = dir.join("map.html");
    fs::write(&p, page).unwrap();
    eprintln!("map: {} sessions · {} repos · {} homes · {:.1} MB in {:.1}s -> {}", n, cats.words.len(), homes.len(),
              fs::metadata(&p).unwrap().len() as f64 / 1e6, t0.elapsed().as_secs_f64(), p.display());
}
