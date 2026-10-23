//! OUT/meta/months.json: one entry per month file, so a hub (relic5) can tell a month it holds from one that changed.
//!
//!   rows, first_ts, max_ts   what the month file holds now
//!   keys                     blake2b-16 over every row's (s, h, idx) in file order: equal manifests, equal rows
//!   late_rows                rows this run added that sit at or before the month's previous newest time. A transcript
//!                            only grows at its end, so this is normally 0; more means an import, a mirror of another
//!                            machine, or a file whose past changed (the ledger's "past changed")
//!   late_total               late_rows summed over every run since the manifest began
//!   sealed                   the month is before the current Bangkok month: no new line can be written into it
//!
//! A month is read again only when its file changed (size or mtime); the other entries are copied over.

use std::fs;
use std::path::Path;

use arrow::array::{Array, Int64Array, StringArray};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ProjectionMask;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Entry {
    pub month: String,
    pub rows: u64,
    pub first_ts: String,
    pub max_ts: String,
    pub keys: String,
    pub bytes: u64,
    pub mtime_ms: i64,
    pub late_rows: u64,
    pub late_total: u64,
    pub sealed: bool,
    pub updated_at: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct File {
    version: u32,
    out: String,
    months: Vec<Entry>,
}

/// The month file's entry, read from the file; `prev` is what the manifest held before.
fn read_month(path: &Path, month: &str, prev: Option<&Entry>) -> Entry {
    let b = ParquetRecordBatchReaderBuilder::try_new(fs::File::open(path).unwrap()).unwrap();
    let cols: Vec<usize> = ["s", "h", "idx", "ts"].iter().map(|n| b.schema().index_of(n).unwrap()).collect();
    let mask = ProjectionMask::roots(b.parquet_schema(), cols);
    let rd = b.with_projection(mask).with_batch_size(65_536).build().unwrap();
    let mut st = blake2b_simd::Params::new().hash_length(16).to_state();
    let (mut rows, mut upto_prev) = (0u64, 0u64);
    let (mut first, mut max) = (String::new(), String::new());
    let before = prev.map(|p| p.max_ts.as_str()).filter(|t| !t.is_empty());
    for batch in rd {
        let batch = batch.unwrap();
        let s = batch.column_by_name("s").unwrap().as_any().downcast_ref::<StringArray>().unwrap();
        let ts = batch.column_by_name("ts").unwrap().as_any().downcast_ref::<StringArray>().unwrap();
        let h = batch.column_by_name("h").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
        let idx = batch.column_by_name("idx").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
        for i in 0..batch.num_rows() {
            rows += 1;
            st.update(s.value(i).as_bytes()).update(&h.value(i).to_le_bytes()).update(&idx.value(i).to_le_bytes());
            if ts.is_null(i) || ts.value(i).is_empty() {
                continue; // a row with no time (the month "undated")
            }
            let t = ts.value(i);
            if first.is_empty() || t < first.as_str() {
                first = t.to_string();
            }
            if t > max.as_str() {
                max = t.to_string();
            }
            if before.is_some_and(|p| t <= p) {
                upto_prev += 1;
            }
        }
    }
    let prev_rows = prev.map(|p| p.rows).unwrap_or(0);
    // rows now at or before the old newest time, minus the rows the old manifest held: what arrived out of order
    let late = if prev.is_some() && before.is_some() { upto_prev.saturating_sub(prev_rows) } else { 0 };
    let md = fs::metadata(path).unwrap();
    let mtime_ms = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64).unwrap_or(0);
    Entry {
        month: month.to_string(), rows, first_ts: first, max_ts: max, keys: hex(&st.finalize().as_bytes()[..16]),
        bytes: md.len(), mtime_ms, late_rows: late, late_total: prev.map(|p| p.late_total).unwrap_or(0) + late,
        sealed: false, updated_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The current Bangkok month, "YYYY-MM".
fn bangkok_month() -> String {
    (chrono::Utc::now() + chrono::Duration::hours(7)).format("%Y-%m").to_string()
}

/// Write OUT/meta/months.json for every month file in OUT. `now` is the current Bangkok month (a parameter for tests).
pub fn build_at(out: &Path, now: &str) -> Vec<Entry> {
    let path = out.join("meta").join("months.json");
    let old: File = fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    let mut names: Vec<String> = fs::read_dir(out).into_iter().flatten().flatten()
        .filter_map(|e| e.file_name().to_string_lossy().strip_suffix(".parquet").map(str::to_string))
        .filter(|m| crate::meta::month_file(out, m).is_some() || m == "undated").collect();
    names.sort();
    let mut entries = vec![];
    for m in names {
        let f = out.join(format!("{m}.parquet"));
        let prev = old.months.iter().find(|e| e.month == m);
        let md = fs::metadata(&f).unwrap();
        let mtime = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64).unwrap_or(0);
        let mut e = match prev {
            Some(p) if p.bytes == md.len() && p.mtime_ms == mtime => Entry { late_rows: 0, ..p.clone() },
            _ => read_month(&f, &m, prev),
        };
        e.sealed = m.as_str() < now && m != "undated";
        entries.push(e);
    }
    let _ = fs::create_dir_all(out.join("meta"));
    let tmp = path.with_extension("json.part");
    if fs::write(&tmp, serde_json::to_vec_pretty(&File { version: 1, out: out.display().to_string(), months: entries.clone() }).unwrap()).is_ok() {
        let _ = fs::rename(&tmp, &path);
    }
    entries
}

pub fn build(out: &Path) -> Vec<Entry> {
    build_at(out, &bangkok_month())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::tests::{lines, month_rows, tmp};

    #[test]
    fn manifest_follows_the_months_and_flags_late_rows() {
        let d = tmp("manifest");
        let (src, out) = (d.join("s.jsonl"), d.join("out"));
        let sp = src.to_str().unwrap().to_string();
        let job = |ctx| crate::parse::Job { path: sp.clone(), size: fs::metadata(&src).unwrap().len(), verdict: String::new(), ctx };
        fs::write(&src, lines(2, 10)).unwrap();
        let c1 = crate::parse::run(vec![job(Default::default())], &out, "t@x", 2, false, false, None)[&sp].clone();
        let a = build_at(&out, "2026-08");
        let m = a.iter().find(|e| e.month == "2026-07").unwrap();
        assert_eq!(m.rows as usize, month_rows(&out.join("2026-07.parquet")).len());
        assert!(m.sealed && m.late_rows == 0 && !m.max_ts.is_empty() && m.first_ts <= m.max_ts);
        assert!(!build_at(&out, "2026-07").iter().find(|e| e.month == "2026-07").unwrap().sealed, "the current month is open");
        // same file again: the entry is copied, not read
        let again = build_at(&out, "2026-08");
        assert_eq!(again, a);
        // the file grows at its end: new rows, none late
        fs::write(&src, lines(2, 10) + &lines(3, 10)).unwrap();
        crate::parse::run(vec![job(c1.clone())], &out, "t@x", 2, false, false, None);
        let b = build_at(&out, "2026-08");
        let n = b.iter().find(|e| e.month == "2026-07").unwrap();
        assert!(n.rows > m.rows && n.max_ts > m.max_ts && n.keys != m.keys && n.late_rows == 0, "{n:?}");
        // another file holding an older day arrives: its rows sit before the newest time, so they are late
        let old = d.join("o.jsonl");
        fs::write(&old, lines(1, 6)).unwrap();
        let op = old.to_str().unwrap().to_string();
        let j2 = crate::parse::Job { path: op.clone(), size: fs::metadata(&old).unwrap().len(), verdict: String::new(), ctx: Default::default() };
        crate::parse::run(vec![j2], &out, "t@x", 2, false, false, None);
        let c = build_at(&out, "2026-08");
        let l = c.iter().find(|e| e.month == "2026-07").unwrap();
        assert!(l.rows > n.rows && l.late_rows > 0 && l.late_total == l.late_rows, "{l:?}");
        let _ = fs::remove_dir_all(&d);
    }
}
