//! OUT/_scans.jsonl: one line per index run or check (dry run), oldest first, never rewritten.
//!
//!   {"at": unix s, "job": "index" | "check", "range": "2026-10" | null, "kind": [...], "only": [...],
//!    "new", "grown", "rewritten", "unchanged", "gone", "before_range", "files", "bytes", "scan_secs", "secs",
//!    "rows", "sessions" (index only: the totals after it),
//!    "diff": [[status, path, size, from], ...], "diff_total"}
//!
//! `diff` lists the files the run read (or a check would read) and the files gone from disk: status is new,
//! grown, rewritten or gone; `from` is the offset the read starts at (grown) or the old size (gone). The first index
//! of a machine lists tens of thousands of files, so `diff` keeps the biggest MAX_DIFF by bytes to read and
//! `diff_total` says how many there were. An entry is one write of one line, appended, so a crash leaves the log whole
//! up to the last full line, and a reader skips a torn one.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::{json, Value};

pub const MAX_DIFF: usize = 2000;

/// One changed file: (status, path, size now, offset the read starts at / size before).
pub type Change = (&'static str, String, u64, u64);

pub fn path(out: &Path) -> std::path::PathBuf {
    out.join("_scans.jsonl")
}

/// Append one entry; returns its index (its line number, from 0), which the page and `relic-scan log` use as its id.
pub fn append(out: &Path, mut entry: Value, mut diff: Vec<Change>) -> Option<usize> {
    let total = diff.len();
    // the biggest reads first: the files that cost the time are the ones worth seeing
    diff.sort_by(|a, b| (b.2.saturating_sub(b.3)).cmp(&(a.2.saturating_sub(a.3))).then_with(|| a.1.cmp(&b.1)));
    diff.truncate(MAX_DIFF);
    entry["diff"] = json!(diff.iter().map(|(s, p, size, from)| json!([s, p, size, from])).collect::<Vec<_>>());
    entry["diff_total"] = json!(total);
    let n = fs::read(path(out)).map(|b| b.iter().filter(|&&c| c == b'\n').count()).unwrap_or(0);
    let mut line = entry.to_string();
    line.push('\n');
    let mut f = fs::OpenOptions::new().create(true).append(true).open(path(out)).ok()?;
    f.write_all(line.as_bytes()).ok()?;
    Some(n)
}

/// Every entry, oldest first, each with its index as "i". A line that does not parse (torn by a crash) is skipped
/// but keeps its number, so ids never shift.
pub fn read(out: &Path) -> Vec<Value> {
    let Ok(t) = fs::read_to_string(path(out)) else { return vec![] };
    t.lines().enumerate().filter_map(|(i, l)| {
        let mut v: Value = serde_json::from_str(l).ok()?;
        v["i"] = json!(i);
        Some(v)
    }).collect()
}

/// An entry without its file list (what a list of runs needs).
pub fn summary(e: &Value) -> Value {
    let mut s = e.clone();
    if let Some(o) = s.as_object_mut() {
        o.remove("diff");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_numbers_and_skips_torn_lines() {
        let d = std::env::temp_dir().join(format!("relic-scan-test-{}-scanlog", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        assert_eq!(append(&d, json!({"job": "check"}), vec![("new", "/a".into(), 10, 0)]), Some(0));
        // a torn line, as a crash mid-write would leave it
        fs::OpenOptions::new().append(true).open(path(&d)).unwrap().write_all(b"{\"job\":\"ind\n").unwrap();
        let many: Vec<Change> = (0..MAX_DIFF + 5).map(|k| ("grown", format!("/f{k}"), 100 + k as u64, 100)).collect();
        assert_eq!(append(&d, json!({"job": "index"}), many), Some(2));
        let all = read(&d);
        assert_eq!(all.len(), 2, "the torn line is skipped");
        assert_eq!((all[0]["i"].as_u64(), all[1]["i"].as_u64()), (Some(0), Some(2)), "ids are line numbers");
        assert_eq!(all[1]["diff_total"], MAX_DIFF + 5);
        assert_eq!(all[1]["diff"].as_array().unwrap().len(), MAX_DIFF);
        assert_eq!(all[1]["diff"][0][1], format!("/f{}", MAX_DIFF + 4), "the biggest read first");
        assert!(summary(&all[1]).get("diff").is_none());
        let _ = fs::remove_dir_all(&d);
    }
}
