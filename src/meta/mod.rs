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

mod agg;
mod rules;
#[cfg(test)]
mod tests;

pub use agg::*;
pub use rules::*;

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

/// What `build` wrote, for the run's summary.
pub struct Summary {
    pub files: i64,
    pub sessions: i64,
    pub rows: i64,
}

/// Build OUT/meta/files.parquet and OUT/meta/sessions.parquet from the month files and the state.
pub fn build(out: &Path, agent: &str, st: &HashMap<String, FileState>, threads: usize) -> Summary {
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
    Summary { files: files.len() as i64, sessions: rows.len() as i64, rows: rows.iter().map(|r| r.2.agg.n_rows).sum() }
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
