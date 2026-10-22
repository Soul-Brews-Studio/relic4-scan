//! `relic-scan meta` (and the last step of `run`): the month files + _state.tsv -> OUT/meta/files.parquet and
//! OUT/meta/sessions.parquet. A Rust port of agent-relic-v4 `relic4 meta --from-scan` (src/relic4/meta.py): the same
//! columns, the same path rules (sql.py MACROS), with no truth import and no DuckDB.
//!
//!   per month (cached):  every row's sources -> per file:    n_rows, text_rows, rows by type, first_ts, last_ts
//!                                            -> per session: distinct rows, text rows, text messages, first/last ts
//!                        OUT/meta/cache/YYYY-MM.tsv, redone only when YYYY-MM.parquet changed (size or mtime)
//!   whole (every run):   _state.tsv files + the month caches -> files.parquet (one row per file on disk)
//!                                                            -> sessions.parquet (one row per file NAME, copies folded)
//!
//! It lives in OUT/meta/, not beside the months: `relic4 truth import --from-scan` reads every OUT/*.parquet as a month.
//! One difference from meta.py: a session's counts include rows whose only source file has since been deleted
//! (truth is append-only, so those rows still belong to the session); meta.py drops them as orphans.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use arrow::array::{Array, ArrayRef, BooleanArray, Int64Array, ListArray, ListBuilder, RecordBatch, StringArray, StringBuilder};
use arrow::datatypes::{Field, Schema};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::{ArrowWriter, ProjectionMask};
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;
use regex::Regex;

use crate::state::FileState;

// ---------- path rules: sql.py MACROS, one for one ----------

const UUID: &str = "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";

struct Rx {
    wf: Regex,
    sub: Regex,
    main_uuid: Regex,
    main_rollout: Regex,
    main_omp: Regex,
    main_other: Regex,
    history: Regex,
    proj: Regex,
    parent: Regex,
    home: Regex,
    agent_sid: Regex,
    gh: Regex,
    gh_org: Regex,
    gh_wt: Regex,
    dashes: Regex,
    wts: Regex,
    wts_wt: Regex,
}

fn rx() -> &'static Rx {
    static R: OnceLock<Rx> = OnceLock::new();
    R.get_or_init(|| {
        let r = |p: &str| Regex::new(p).expect("meta rule");
        Rx {
            wf: r(r"/subagents/workflows/.*agent-[^/]+\.jsonl$"),
            sub: r(r"/subagents/agent-[^/]+\.jsonl$"),
            main_uuid: r(&format!(r"/{UUID}\.jsonl$")),
            main_rollout: r(&format!(r"rollout-.*{UUID}\.jsonl$")),
            main_omp: r(r"/\.(omp|pi)/.*_[0-9a-f-]+\.jsonl$"),
            main_other: r(r"/\.(omx|thclaws|gemini)/"),
            history: r(r"history\.jsonl$"),
            proj: r(&format!(r"/([^/]+)/(?:{UUID}\.jsonl|{UUID}/subagents/)")),
            parent: r(&format!(r"/({UUID})/subagents/")),
            home: r(r"^(.*?/\.?(?:claude|codex|omp|omx|pi|thclaws|gemini)[^/]*)/"),
            agent_sid: r(r"^agent-(.*)\.jsonl$"),
            gh: r(r".*github-com-(.*)$"),
            gh_org: r(r"^(soul-brews-studio|laris-co|dustboy-pm25|floodboy-cm|nazt|nat-weerawan)-"),
            gh_wt: r(r"(-worktree-|-wt-|--claude-worktrees-).*$"),
            dashes: r(r"^-+|-+$"),
            wts: r(r".*-worktrees-(.*)$"),
            wts_wt: r(r"(-worktree-|-wt-).*$"),
        }
    })
}

/// DuckDB regexp_extract: group `g` of the first match, "" when there is none.
fn extract(re: &Regex, s: &str, g: usize) -> String {
    re.captures(s).and_then(|c| c.get(g)).map(|m| m.as_str().to_string()).unwrap_or_default()
}

pub fn tier_of(path: &str) -> &'static str {
    let r = rx();
    if r.wf.is_match(path) {
        "workflow-agent"
    } else if r.sub.is_match(path) {
        "subagent"
    } else if r.main_uuid.is_match(path) || r.main_rollout.is_match(path) || r.main_omp.is_match(path)
        || (r.main_other.is_match(path) && !r.history.is_match(path)) {
        "main"
    } else {
        "other"
    }
}

pub fn engine_of(path: &str) -> &'static str {
    if path.contains("rollout-") || path.contains("/.codex") {
        "codex"
    } else if path.contains("/.omp/") {
        "omp"
    } else if path.contains("/.omx/") {
        "omx"
    } else if path.contains("/.thclaws/") {
        "thclaws"
    } else if path.contains("/.gemini/") {
        "gemini"
    } else if path.contains("/.pi/") {
        "pi"
    } else {
        "claude"
    }
}

fn base_of(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn proj_of(path: &str) -> Option<String> {
    Some(extract(&rx().proj, path, 1)).filter(|s| !s.is_empty())
}

fn parent_of(path: &str) -> Option<String> {
    Some(extract(&rx().parent, path, 1)).filter(|s| !s.is_empty())
}

fn home_of(path: &str) -> String {
    extract(&rx().home, path, 1)
}

pub fn cat_of(engine: &str, proj: Option<&str>) -> String {
    let r = rx();
    if engine != "claude" {
        return format!("{engine} (no cwd in path)");
    }
    let Some(p) = proj.filter(|p| !p.is_empty()) else { return "(no project dir)".into() };
    let lp = p.to_lowercase();
    if lp.contains("oracle-probe-cwd") {
        "probe (automated)".into()
    } else if lp.contains("github-com-") {
        let x = extract(&r.gh, &lp, 1);
        let x = r.gh_org.replace(&x, "");
        let x = r.gh_wt.replace(&x, "");
        r.dashes.replace_all(&x, "").into_owned()
    } else if lp.contains("-worktrees-") {
        let x = extract(&r.wts, &lp, 1);
        r.wts_wt.replace(&x, "").into_owned()
    } else {
        "home / other".into()
    }
}

// ---------- per month: one pass over the rows' sources ----------

/// Rows by type, in this order: user text, assistant text, thinking, tool calls, tool results, tool errors
/// (errors are results with err = true, so they are also counted in results).
pub const TYPES: [&str; 6] = ["user_text", "assistant_text", "thinking", "tool_calls", "tool_results", "tool_errors"];

fn types_of(s: &str, role: Option<&str>, kind: Option<&str>, err: bool) -> [i64; 6] {
    let mut t = [0; 6];
    match (s, role, kind) {
        ("text", Some("user"), _) => t[0] = 1,
        ("text", Some("assistant"), _) => t[1] = 1,
        ("thinking", _, _) => t[2] = 1,
        ("tools", _, Some("call")) => t[3] = 1,
        ("tools", _, Some("result")) => {
            t[4] = 1;
            t[5] = err as i64;
        }
        _ => {}
    }
    t
}

#[derive(Default, Clone)]
struct Agg {
    n_rows: i64,
    text_rows: i64,
    text_msgs: i64,
    by: [i64; 6],
    first_ts: Option<String>,
    last_ts: Option<String>,
}

impl Agg {
    fn ts(&mut self, ts: Option<&str>) {
        let Some(t) = ts else { return };
        if self.first_ts.as_deref().is_none_or(|f| t < f) {
            self.first_ts = Some(t.to_string());
        }
        if self.last_ts.as_deref().is_none_or(|l| t > l) {
            self.last_ts = Some(t.to_string());
        }
    }
    fn add(&mut self, o: &Agg) {
        self.n_rows += o.n_rows;
        self.text_rows += o.text_rows;
        self.text_msgs += o.text_msgs;
        for (a, b) in self.by.iter_mut().zip(o.by) {
            *a += b;
        }
        self.ts(o.first_ts.as_deref());
        self.ts(o.last_ts.as_deref());
    }
}

/// One month's aggregates: per file path, and per session (base, tier).
#[derive(Default)]
struct MonthAgg {
    files: HashMap<String, Agg>,
    sessions: HashMap<(String, String), Agg>,
}

/// msgkey for a text row is md5(to_json([s, role, ts, text])); count(DISTINCT msgkey) only needs a stable key of
/// (role, ts, text) with NULL kept apart from any string, so a length-prefixed blake2b-8 does the same job.
fn msg_key(role: Option<&str>, ts: Option<&str>, text: Option<&str>) -> u64 {
    let mut st = blake2b_simd::Params::new().hash_length(8).to_state();
    for v in [role, ts, text] {
        match v {
            None => {
                st.update(&[0]);
            }
            Some(s) => {
                st.update(&[1]);
                st.update(&(s.len() as u64).to_le_bytes());
                st.update(s.as_bytes());
            }
        }
    }
    u64::from_le_bytes(st.finalize().as_bytes().try_into().unwrap())
}

fn str_col<'a>(b: &'a RecordBatch, n: &str) -> &'a StringArray {
    b.column_by_name(n).unwrap().as_any().downcast_ref::<StringArray>().unwrap()
}

fn opt(a: &StringArray, i: usize) -> Option<&str> {
    if a.is_null(i) { None } else { Some(a.value(i)) }
}

fn month_agg(path: &Path, agent: &str) -> MonthAgg {
    let b = ParquetRecordBatchReaderBuilder::try_new(fs::File::open(path).unwrap()).unwrap();
    let want = ["s", "role", "kind", "err", "ts", "text", "sources"];
    let idx: Vec<usize> = want.iter().map(|n| b.schema().index_of(n).unwrap()).collect();
    let mask = ProjectionMask::roots(b.parquet_schema(), idx);
    let rd = b.with_projection(mask).with_batch_size(16_384).build().unwrap();
    let prefix = format!("{agent}:");
    let mut out = MonthAgg::default();
    let mut class: HashMap<String, Option<(String, String)>> = HashMap::new(); // path -> session key (None = tier other)
    let mut msgs: HashMap<(String, String), HashSet<u64>> = HashMap::new();
    let mut touched: Vec<(String, String)> = vec![];
    for batch in rd {
        let batch = batch.unwrap();
        let (s, role, ts, text) = (str_col(&batch, "s"), str_col(&batch, "role"), str_col(&batch, "ts"), str_col(&batch, "text"));
        let kind = str_col(&batch, "kind");
        let err = batch.column_by_name("err").unwrap().as_any().downcast_ref::<arrow::array::BooleanArray>().unwrap();
        let src = batch.column_by_name("sources").unwrap().as_any().downcast_ref::<ListArray>().unwrap();
        let vals = src.values().as_any().downcast_ref::<StringArray>().unwrap();
        let offs = src.value_offsets();
        for i in 0..batch.num_rows() {
            let is_text = s.value(i) == "text";
            let t = opt(ts, i);
            let ty = types_of(s.value(i), opt(role, i), opt(kind, i), !err.is_null(i) && err.value(i));
            touched.clear();
            for j in offs[i] as usize..offs[i + 1] as usize {
                let Some(rest) = vals.value(j).strip_prefix(&prefix) else { continue };
                let Some((p, _line)) = rest.rsplit_once(':') else { continue };
                let fa = out.files.entry(p.to_string()).or_default();
                fa.n_rows += 1;
                fa.text_rows += is_text as i64;
                for (a, b) in fa.by.iter_mut().zip(ty) {
                    *a += b;
                }
                fa.ts(t);
                let k = class.entry(p.to_string()).or_insert_with(|| {
                    let tier = tier_of(p);
                    (tier != "other").then(|| (base_of(p).to_string(), tier.to_string()))
                });
                if let Some(k) = k
                    && !touched.contains(k) {
                    touched.push(k.clone()); // a row counts once per session, however many of its copies it holds
                }
            }
            for k in &touched {
                let sa = out.sessions.entry(k.clone()).or_default();
                sa.n_rows += 1;
                sa.text_rows += is_text as i64;
                for (a, b) in sa.by.iter_mut().zip(ty) {
                    *a += b;
                }
                sa.ts(t);
                if is_text {
                    msgs.entry(k.clone()).or_default().insert(msg_key(opt(role, i), t, opt(text, i)));
                }
            }
        }
    }
    for (k, m) in msgs {
        out.sessions.get_mut(&k).unwrap().text_msgs = m.len() as i64;
    }
    out
}

// ---------- the month cache: OUT/meta/cache/YYYY-MM.tsv ----------

fn stamp_of(p: &Path) -> String {
    let m = fs::metadata(p).unwrap();
    format!("{}\t{}.{}", m.size(), m.mtime(), m.mtime_nsec())
}

// NULL is written as \N: an empty ts is a real value (the undated month holds rows with ts = "")
fn agg_tsv(a: &Agg) -> String {
    let by: Vec<String> = a.by.iter().map(|x| x.to_string()).collect();
    format!("{}\t{}\t{}\t{}\t{}\t{}", a.n_rows, a.text_rows, a.text_msgs, by.join("\t"), a.first_ts.as_deref().unwrap_or("\\N"),
            a.last_ts.as_deref().unwrap_or("\\N"))
}

fn agg_from(c: &[&str]) -> Agg {
    let n = |i: usize| c[i].parse::<i64>().unwrap_or(0);
    let t = |i: usize| Some(c[i].to_string()).filter(|s| s != "\\N");
    Agg { n_rows: n(0), text_rows: n(1), text_msgs: n(2), by: std::array::from_fn(|j| n(3 + j)), first_ts: t(9), last_ts: t(10) }
}

fn save_cache(p: &Path, stamp: &str, agent: &str, m: &MonthAgg) {
    let tmp = p.with_extension("tsv.part");
    let mut w = std::io::BufWriter::new(fs::File::create(&tmp).unwrap());
    writeln!(w, "#3\t{stamp}\t{agent}").unwrap();
    for (f, a) in &m.files {
        writeln!(w, "F\t{f}\t{}", agg_tsv(a)).unwrap();
    }
    for ((base, tier), a) in &m.sessions {
        writeln!(w, "S\t{base}\t{tier}\t{}", agg_tsv(a)).unwrap();
    }
    w.flush().unwrap();
    drop(w);
    fs::rename(tmp, p).unwrap();
}

/// The cached aggregates, or None when the cache was made from another version of the month file.
fn load_cache(p: &Path, stamp: &str, agent: &str) -> Option<MonthAgg> {
    let mut lines = BufReader::new(fs::File::open(p).ok()?).lines().map_while(Result::ok);
    if lines.next()? != format!("#3\t{stamp}\t{agent}") {
        return None;
    }
    let mut m = MonthAgg::default();
    for l in lines {
        let c: Vec<&str> = l.split('\t').collect();
        match c[0] {
            "F" if c.len() == 13 => {
                m.files.insert(c[1].to_string(), agg_from(&c[2..]));
            }
            "S" if c.len() == 14 => {
                m.sessions.insert((c[1].to_string(), c[2].to_string()), agg_from(&c[3..]));
            }
            _ => return None,
        }
    }
    Some(m)
}

// ---------- output ----------

fn utf8(rows: impl Iterator<Item = Option<String>>) -> ArrayRef {
    Arc::new(rows.collect::<StringArray>())
}

fn i64s(rows: impl Iterator<Item = Option<i64>>) -> ArrayRef {
    Arc::new(rows.collect::<Int64Array>())
}

fn lists(rows: impl Iterator<Item = Vec<String>>) -> ArrayRef {
    let mut b = ListBuilder::new(StringBuilder::new());
    for v in rows {
        for x in v {
            b.values().append_value(x);
        }
        b.append(true);
    }
    Arc::new(b.finish())
}

fn write_parquet(p: &Path, cols: Vec<(&str, ArrayRef)>) {
    let schema = Arc::new(Schema::new(cols.iter().map(|(n, a)| Field::new(*n, a.data_type().clone(), true)).collect::<Vec<_>>()));
    let batch = RecordBatch::try_new(schema.clone(), cols.into_iter().map(|(_, a)| a).collect()).unwrap();
    let tmp = p.with_extension("parquet.part");
    let props = WriterProperties::builder().set_compression(Compression::ZSTD(ZstdLevel::try_new(3).unwrap())).build();
    let mut w = ArrowWriter::try_new(fs::File::create(&tmp).unwrap(), schema, Some(props)).unwrap();
    w.write(&batch).unwrap();
    w.into_inner().unwrap().sync_all().unwrap();
    fs::rename(tmp, p).unwrap();
}

struct FileRow {
    path: String,
    home: String,
    size: i64,
    lines: i64,
    last_ms: Option<i64>,
    file_sid: Option<String>,
    base: String,
    tier: &'static str,
    engine: &'static str,
    proj: Option<String>,
    parent_sid: Option<String>,
    cat: String,
    agg: Agg,
}

/// Every OUT/*.parquet is a month, as `relic4 truth import --from-scan` reads them: YYYY-MM, and `undated`.
fn month_of(name: &str) -> Option<&str> {
    name.strip_suffix(".parquet").filter(|m| !m.is_empty() && !m.starts_with('.'))
}

/// Build OUT/meta/files.parquet and OUT/meta/sessions.parquet from the month files and the state.
pub fn build(out: &Path, agent: &str, st: &HashMap<String, FileState>, threads: usize) {
    let t0 = Instant::now();
    let dir = out.join("meta");
    let cache = dir.join("cache");
    fs::create_dir_all(&cache).unwrap();
    let mut months: Vec<String> = fs::read_dir(out).unwrap().flatten()
        .filter_map(|e| month_of(&e.file_name().to_string_lossy()).map(str::to_string)).collect();
    months.sort();
    // a month's cache is redone only when its month file changed since the cache was written
    let n = if threads == 0 { std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8) } else { threads };
    let next = AtomicUsize::new(0);
    let aggs: Mutex<BTreeMap<String, MonthAgg>> = Mutex::default();
    let redone = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..n.min(months.len()).max(1) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(m) = months.get(i) else { break };
                let src = out.join(format!("{m}.parquet"));
                let cp = cache.join(format!("{m}.tsv"));
                let stamp = stamp_of(&src);
                let a = match load_cache(&cp, &stamp, agent) {
                    Some(a) => a,
                    None => {
                        let a = month_agg(&src, agent);
                        save_cache(&cp, &stamp, agent, &a);
                        redone.fetch_add(1, Ordering::Relaxed);
                        a
                    }
                };
                aggs.lock().unwrap().insert(m.clone(), a);
            });
        }
    });
    let aggs = aggs.into_inner().unwrap();
    let t_months = t0.elapsed().as_secs_f64();
    // drop the caches of months that no longer exist
    for e in fs::read_dir(&cache).unwrap().flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.strip_suffix(".tsv").is_none_or(|m| !aggs.contains_key(m)) {
            let _ = fs::remove_file(e.path());
        }
    }

    // files: one row per file in the state, its rows summed over every month
    let mut per_file: HashMap<&str, Agg> = HashMap::new();
    for a in aggs.values() {
        for (p, x) in &a.files {
            per_file.entry(p.as_str()).or_default().add(x);
        }
    }
    let mut paths: Vec<&String> = st.keys().collect();
    paths.sort();
    let files: Vec<FileRow> = paths.iter().map(|p| {
        let s = &st[*p];
        let engine = engine_of(p);
        let proj = proj_of(p);
        FileRow {
            path: p.to_string(), home: home_of(p), size: s.size as i64, lines: s.ctx.lines, last_ms: s.ctx.last_ms,
            file_sid: Some(s.ctx.session.clone()).filter(|x| !x.is_empty()), base: base_of(p).to_string(), tier: tier_of(p),
            engine, cat: cat_of(engine, proj.as_deref()), proj, parent_sid: parent_of(p),
            agg: per_file.get(p.as_str()).cloned().unwrap_or_default(),
        }
    }).collect();
    let mut fcols = vec![
        ("agent", utf8(files.iter().map(|_| Some(agent.to_string())))),
        ("path", utf8(files.iter().map(|f| Some(f.path.clone())))),
        ("home", utf8(files.iter().map(|f| Some(f.home.clone())))),
        ("size", i64s(files.iter().map(|f| Some(f.size)))),
        ("lines", i64s(files.iter().map(|f| Some(f.lines)))),
        ("last_ms", i64s(files.iter().map(|f| f.last_ms))),
        ("file_sid", utf8(files.iter().map(|f| f.file_sid.clone()))),
        ("base", utf8(files.iter().map(|f| Some(f.base.clone())))),
        ("tier", utf8(files.iter().map(|f| Some(f.tier.to_string())))),
        ("engine", utf8(files.iter().map(|f| Some(f.engine.to_string())))),
        ("proj", utf8(files.iter().map(|f| f.proj.clone()))),
        ("parent_sid", utf8(files.iter().map(|f| f.parent_sid.clone()))),
        ("cat", utf8(files.iter().map(|f| Some(f.cat.clone())))),
        ("n_rows", i64s(files.iter().map(|f| Some(f.agg.n_rows)))),
        ("text_rows", i64s(files.iter().map(|f| Some(f.agg.text_rows)))),
        ("first_ts", utf8(files.iter().map(|f| f.agg.first_ts.clone()))),
        ("last_ts", utf8(files.iter().map(|f| f.agg.last_ts.clone()))),
    ];
    for (j, n) in TYPES.iter().enumerate() {
        fcols.push((n, i64s(files.iter().map(|f| Some(f.agg.by[j])))));
    }
    write_parquet(&dir.join("files.parquet"), fcols);

    // sessions: one row per file NAME (base, tier), every copy folded; counts from the month aggregates
    struct Sess<'a> {
        engine: &'static str,
        last_ms: Option<i64>,
        copies: i64,
        size: i64,
        proj: Option<&'a str>,
        file_sid: Option<&'a str>,
        parent_sid: Option<&'a str>,
        agg: Agg,
        months: BTreeSet<&'a str>,
    }
    let mut ss: HashMap<(&str, &str), Sess> = HashMap::new();
    for f in files.iter().filter(|f| f.tier != "other") {
        let e = ss.entry((f.base.as_str(), f.tier)).or_insert(Sess {
            engine: f.engine, last_ms: None, copies: 0, size: 0, proj: None, file_sid: None, parent_sid: None,
            agg: Agg::default(), months: BTreeSet::new(),
        });
        e.last_ms = e.last_ms.max(f.last_ms);
        e.copies += 1;
        e.size = e.size.max(f.size);
        e.proj = e.proj.or(f.proj.as_deref());
        e.file_sid = e.file_sid.or(f.file_sid.as_deref());
        e.parent_sid = e.parent_sid.or(f.parent_sid.as_deref());
    }
    for (m, a) in &aggs {
        for ((base, tier), x) in &a.sessions {
            if let Some(e) = ss.get_mut(&(base.as_str(), tier.as_str())) {
                e.agg.add(x);
                e.months.insert(m.as_str());
            }
        }
    }
    let mut keys: Vec<(&str, &str)> = ss.keys().copied().collect();
    keys.sort_by(|a, b| {
        let (x, y) = (&ss[a].agg.first_ts, &ss[b].agg.first_ts);
        (x.is_none(), x, a.0).cmp(&(y.is_none(), y, b.0)).then(a.1.cmp(b.1))
    });
    let rows: Vec<(&str, &str, &Sess)> = keys.iter().map(|k| (k.0, k.1, &ss[k])).collect();
    let cats: Vec<String> = rows.iter().map(|(_, _, s)| cat_of(s.engine, s.proj)).collect();
    let mut scols = vec![
        ("base", utf8(rows.iter().map(|r| Some(r.0.to_string())))),
        ("tier", utf8(rows.iter().map(|r| Some(r.1.to_string())))),
        ("sid", utf8(rows.iter().map(|(base, tier, s)| {
            if *tier == "main" { s.file_sid.map(str::to_string) } else { Some(extract(&rx().agent_sid, base, 1)) }
        }))),
        ("parent_sid", utf8(rows.iter().map(|(_, tier, s)| {
            s.parent_sid.or(if *tier != "main" { s.file_sid } else { None }).map(str::to_string)
        }))),
        ("engine", utf8(rows.iter().map(|r| Some(r.2.engine.to_string())))),
        ("proj", utf8(rows.iter().map(|r| r.2.proj.map(str::to_string)))),
        ("cat", utf8(cats.iter().map(|c| Some(c.clone())))),
        ("probe", Arc::new(cats.iter().map(|c| Some(c == "probe (automated)")).collect::<BooleanArray>())),
        ("first_ts", utf8(rows.iter().map(|r| r.2.agg.first_ts.clone()))),
        ("last_ts", utf8(rows.iter().map(|r| r.2.agg.last_ts.clone()))),
        ("last_ms", i64s(rows.iter().map(|r| r.2.last_ms))),
        ("copies", i64s(rows.iter().map(|r| Some(r.2.copies)))),
        ("machines", i64s(rows.iter().map(|_| Some(1)))),
        ("agents", lists(rows.iter().map(|_| vec![agent.to_string()]))),
        ("size", i64s(rows.iter().map(|r| Some(r.2.size)))),
        ("n_rows", i64s(rows.iter().map(|r| Some(r.2.agg.n_rows)))),
        ("text_rows", i64s(rows.iter().map(|r| Some(r.2.agg.text_rows)))),
        ("text_msgs", i64s(rows.iter().map(|r| Some(r.2.agg.text_msgs)))),
        ("months", lists(rows.iter().map(|r| r.2.months.iter().map(|m| m.to_string()).collect()))),
    ];
    for (j, n) in TYPES.iter().enumerate() {
        scols.push((n, i64s(rows.iter().map(|r| Some(r.2.agg.by[j])))));
    }
    write_parquet(&dir.join("sessions.parquet"), scols);
    let probe = cats.iter().filter(|c| *c == "probe (automated)").count();
    eprintln!("meta: {} months ({} re-aggregated) in {:.1}s · {} files · {} sessions ({} probe) · {:.1}s -> {}/meta/",
              months.len(), redone.load(Ordering::Relaxed), t_months, files.len(), rows.len(), probe,
              t0.elapsed().as_secs_f64(), out.display());
}

/// OUT/meta/jsonl.parquet: every .jsonl the scan saw, one row each. place = home (an agent home; parsed into the
/// months unless the sniff called it not-transcript) | vault (in a ψ vault) | other (anywhere else under the roots).
pub fn jsonl(out: &Path, r: &crate::scan::ScanResult, st: &HashMap<String, FileState>) {
    let t0 = Instant::now();
    let dir = out.join("meta");
    fs::create_dir_all(&dir).unwrap();
    let mut all: Vec<(&crate::scan::FileEntry, &str)> = r.files.iter().map(|f| (f, "home"))
        .chain(r.loose.iter().map(|f| (f, f.kind.as_str()))).collect();
    all.sort_by(|a, b| a.0.path.cmp(&b.0.path));
    let home = |f: &crate::scan::FileEntry, place: &str| (place == "home").then(|| f.home.clone());
    write_parquet(&dir.join("jsonl.parquet"), vec![
        ("path", utf8(all.iter().map(|(f, _)| Some(f.path.clone())))),
        ("place", utf8(all.iter().map(|(_, p)| Some(p.to_string())))),
        ("home", utf8(all.iter().map(|(f, p)| home(f, p)))),
        ("home_kind", utf8(all.iter().map(|(f, p)| (*p == "home").then(|| f.kind.clone())))),
        ("origin", utf8(all.iter().map(|(f, p)| (*p == "home").then(|| crate::scan::origin(&f.home).to_string())))),
        ("vault", utf8(all.iter().map(|(f, _)| crate::scan::vault_of(&f.path).map(str::to_string)))),
        ("agent", utf8(all.iter().map(|(f, _)| Some(f.agent.clone()).filter(|a| !a.is_empty() && a != "-")))),
        ("verdict", utf8(all.iter().map(|(f, _)| Some(f.verdict.clone())))),
        ("tier", utf8(all.iter().map(|(f, _)| Some(tier_of(&f.path).to_string())))),
        ("size", i64s(all.iter().map(|(f, _)| Some(f.size as i64)))),
        ("mtime_ms", i64s(all.iter().map(|(f, _)| Some(f.mtime)))),
        ("parsed", Arc::new(all.iter().map(|(f, _)| Some(st.contains_key(&f.path))).collect::<BooleanArray>())),
    ]);
    let n = |p: &str| all.iter().filter(|x| x.1 == p).count();
    let vaults: BTreeSet<&str> = r.loose.iter().filter(|f| f.kind == "vault").map(|f| f.home.as_str()).collect();
    eprintln!("meta: jsonl {} (home {} · ψ vault {} in {} vaults · other {}) in {:.1}s -> {}/meta/jsonl.parquet",
              all.len(), n("home"), n("vault"), vaults.len(), n("other"), t0.elapsed().as_secs_f64(), out.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: &str = "0b867d73-405b-4b03-8c46-aee7648f1bd7";

    #[test]
    fn tiers() {
        let h = "/Users/x/.claude/projects/-opt-Code-github-com-laris-co-neo";
        assert_eq!(tier_of(&format!("{h}/{U}.jsonl")), "main");
        assert_eq!(tier_of(&format!("{h}/{U}/subagents/agent-a1.jsonl")), "subagent");
        assert_eq!(tier_of(&format!("{h}/{U}/subagents/workflows/wf_1/agent-a2.jsonl")), "workflow-agent");
        assert_eq!(tier_of(&format!("{h}/{U}/subagents/workflows/wf_1/journal.jsonl")), "other");
        assert_eq!(tier_of(&format!("/Users/x/.codex/sessions/2026/07/01/rollout-2026-07-01T00-00-00-{U}.jsonl")), "main");
        assert_eq!(tier_of("/Users/x/.codex/sessions/rollout-x.jsonl"), "other");
        assert_eq!(tier_of("/Users/x/.omp/agent/sessions/p/2026_ab12-cd.jsonl"), "main");
        assert_eq!(tier_of("/Users/x/.omx/logs/run.jsonl"), "main");
        assert_eq!(tier_of("/Users/x/.omx/history.jsonl"), "other");
        assert_eq!(tier_of("/Users/x/.claude/history.jsonl"), "other");
        // the scan uses the very same rules
        assert_eq!(crate::scan::tier("/Users/x/.omp/agent/sessions/p/2026_ab12-cd.jsonl"), "main");
    }

    #[test]
    fn repos_and_projects() {
        let p = format!("/h/.claude/projects/-opt-Code-github-com-Soul-Brews-Studio-relic4-scan/{U}.jsonl");
        assert_eq!(proj_of(&p).as_deref(), Some("-opt-Code-github-com-Soul-Brews-Studio-relic4-scan"));
        assert_eq!(cat_of("claude", proj_of(&p).as_deref()), "relic4-scan");
        assert_eq!(cat_of("claude", Some("-opt-Code-github-com-laris-co-neo-oracle-wt-fix-x")), "neo-oracle");
        assert_eq!(cat_of("claude", Some("-Users-x-proj-worktrees-thing-wt-2")), "thing");
        assert_eq!(cat_of("claude", Some("-Users-x-oracle-probe-cwd-1")), "probe (automated)");
        assert_eq!(cat_of("claude", Some("-Users-x")), "home / other");
        assert_eq!(cat_of("claude", None), "(no project dir)");
        assert_eq!(cat_of("codex", None), "codex (no cwd in path)");
        assert_eq!(parent_of(&format!("/h/p/{U}/subagents/agent-1.jsonl")).as_deref(), Some(U));
        assert_eq!(home_of(&p), "/h/.claude");
    }

    #[test]
    fn msg_key_keeps_null_apart_from_empty() {
        assert_ne!(msg_key(Some("user"), None, Some("x")), msg_key(Some("user"), Some(""), Some("x")));
        assert_ne!(msg_key(Some("ab"), Some("c"), None), msg_key(Some("a"), Some("bc"), None));
        assert_eq!(msg_key(Some("user"), Some("t"), Some("x")), msg_key(Some("user"), Some("t"), Some("x")));
    }

    #[test]
    fn months_and_cache_round_trip() {
        assert_eq!(month_of("2026-07.parquet"), Some("2026-07"));
        assert_eq!(month_of("undated.parquet"), Some("undated"));
        assert_eq!(month_of(".parquet"), None);
        assert_eq!(month_of("_state.tsv"), None);
        let a = Agg { n_rows: 5, text_rows: 2, text_msgs: 1, by: [1, 1, 0, 2, 1, 1], first_ts: Some(String::new()), last_ts: None };
        let line = format!("F\tp\t{}", agg_tsv(&a));
        let c: Vec<&str> = line.split('\t').collect();
        assert_eq!(c.len(), 13);
        let b = agg_from(&c[2..]);
        assert_eq!((b.n_rows, b.by, b.first_ts, b.last_ts), (5, [1, 1, 0, 2, 1, 1], Some(String::new()), None));
    }
}
