//! `--month`, `--day`, `--from`, `--to`: index only the rows of some Bangkok days, into a folder of their own.
//!
//! A row is kept when its Bangkok date (the first 10 characters of ts_th) lies within [from, to]; a row with no
//! time at all is dropped. The rest of each file is still read, and the state moves past it, so those rows never
//! come back into that folder. That is why a range never writes into a folder indexed without one, or with
//! another range: OUT/_range holds the range a folder was made with, and a run that disagrees with it stops.

use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct Range {
    pub from: String, // "YYYY-MM-DD", inclusive; "" = no lower bound
    pub to: String,   // "YYYY-MM-DD", inclusive; "" = no upper bound
    pub label: String, // what the user typed: "2026-10", "2026-10-22", "2026-09-01..2026-10-15"
}

/// "2026-10-22" as it is; "2026-10" as its first day, or with `end` its last possible one ("-31" compares right
/// for every month, since the bound is only compared as text).
fn day(s: &str, end: bool) -> Result<String, String> {
    let bad = || format!("{s:?} is not a date: use YYYY-MM-DD or YYYY-MM");
    match s.len() {
        10 => chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map(|_| s.to_string()).map_err(|_| bad()),
        7 => chrono::NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d").map(|_| format!("{s}-{}", if end { "31" } else { "01" }))
            .map_err(|_| bad()),
        _ => Err(bad()),
    }
}

impl Range {
    /// The range the flags ask for, or None when no date flag was given.
    pub fn from_args(month: Option<&str>, day_: Option<&str>, from: Option<&str>, to: Option<&str>) -> Result<Option<Range>, String> {
        if let Some(m) = month {
            if m.len() != 7 {
                return Err(format!("--month {m:?}: use YYYY-MM"));
            }
            return Ok(Some(Range { from: day(m, false)?, to: day(m, true)?, label: m.to_string() }));
        }
        if let Some(d) = day_ {
            if d.len() != 10 {
                return Err(format!("--day {d:?}: use YYYY-MM-DD"));
            }
            return Ok(Some(Range { from: day(d, false)?, to: day(d, true)?, label: d.to_string() }));
        }
        if from.is_none() && to.is_none() {
            return Ok(None);
        }
        let f = from.map(|s| day(s, false)).transpose()?.unwrap_or_default();
        let t = to.map(|s| day(s, true)).transpose()?.unwrap_or_default();
        if !f.is_empty() && !t.is_empty() && f > t {
            return Err(format!("--from {} is after --to {}", from.unwrap_or(""), to.unwrap_or("")));
        }
        Ok(Some(Range { from: f, to: t, label: format!("{}..{}", from.unwrap_or(""), to.unwrap_or("")) }))
    }

    /// Does a row with this ts_th ("2026-10-22T09:15:00.000+07:00", or "" for no time) belong in the range?
    pub fn keeps(&self, ts_th: &str) -> bool {
        let Some(d) = ts_th.get(..10) else { return false };
        (self.from.is_empty() || d >= self.from.as_str()) && (self.to.is_empty() || d <= self.to.as_str())
    }

    /// Can a file last modified at `mtime_ms` hold a row on or after `from`? A line is written when it happens, so a
    /// file untouched since before `from` cannot; one day of slack covers clocks and late writes.
    pub fn may_hold(&self, mtime_ms: i64) -> bool {
        self.from.is_empty() || crate::engine::stamp(Some(mtime_ms + 86_400_000)).ts_th.get(..10).is_some_and(|d| d >= self.from.as_str())
    }

    /// "2026-10-01 to 2026-10-31", for people.
    pub fn human(&self) -> String {
        match (self.from.is_empty(), self.to.is_empty()) {
            (false, false) if self.from == self.to => self.from.clone(),
            (false, false) => format!("{} to {}", self.from, self.to),
            (false, true) => format!("{} onwards", self.from),
            (true, false) => format!("up to {}", self.to),
            (true, true) => "every day".into(),
        }
    }

    /// The range `out` was made with, if any.
    pub fn read(out: &Path) -> Option<Range> {
        let t = fs::read_to_string(out.join("_range")).ok()?;
        let c: Vec<&str> = t.trim_end_matches('\n').split('\t').collect();
        (c.len() == 3).then(|| Range { from: c[0].into(), to: c[1].into(), label: c[2].into() })
    }

    /// The range a run over `out` uses: the one asked for, checked against the folder's own; or, when none was
    /// asked for, the folder's own (so the index button and a plain `run --out` keep a ranged folder ranged).
    /// A new range is saved in the folder only when `write` is set (a dry run checks and writes nothing).
    pub fn settle(out: &Path, asked: Option<Range>, write: bool) -> Result<Option<Range>, String> {
        let saved = Self::read(out);
        match (asked, saved) {
            (None, saved) => Ok(saved),
            (Some(a), Some(s)) if a.from == s.from && a.to == s.to => Ok(Some(s)),
            (Some(a), Some(s)) => Err(format!("{} holds only {}; this run asks for {} (use another --out)", out.display(), s.human(), a.human())),
            (Some(_), None) if out.join("_state.tsv").exists() => Err(format!(
                "{} was indexed without a date range; a range would leave rows behind in it for good (use another --out)", out.display())),
            (Some(a), None) if !write => Ok(Some(a)),
            (Some(a), None) => {
                fs::create_dir_all(out).map_err(|e| format!("cannot create {}: {e}", out.display()))?;
                fs::write(out.join("_range"), format!("{}\t{}\t{}\n", a.from, a.to, a.label))
                    .map_err(|e| format!("cannot write {}/_range: {e}", out.display()))?;
                Ok(Some(a))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(m: Option<&str>, d: Option<&str>, f: Option<&str>, t: Option<&str>) -> Range {
        Range::from_args(m, d, f, t).unwrap().unwrap()
    }

    #[test]
    fn flags_become_days() {
        assert_eq!(Range::from_args(None, None, None, None).unwrap(), None);
        let m = r(Some("2026-02"), None, None, None);
        assert_eq!((m.from.as_str(), m.to.as_str(), m.label.as_str()), ("2026-02-01", "2026-02-31", "2026-02"));
        let d = r(None, Some("2026-10-22"), None, None);
        assert_eq!((d.from.as_str(), d.to.as_str()), ("2026-10-22", "2026-10-22"));
        let ft = r(None, None, Some("2026-09"), Some("2026-10-15"));
        assert_eq!((ft.from.as_str(), ft.to.as_str(), ft.label.as_str()), ("2026-09-01", "2026-10-15", "2026-09..2026-10-15"));
        let open = r(None, None, Some("2026-10-01"), None);
        assert_eq!((open.from.as_str(), open.to.as_str()), ("2026-10-01", ""));
        for bad in [(Some("2026-13"), None, None, None), (Some("2026-10-01"), None, None, None), (None, Some("2026-10"), None, None),
                    (None, Some("2026-02-30"), None, None), (None, None, Some("2026-10-02"), Some("2026-10-01")), (None, None, Some("yesterday"), None)] {
            assert!(Range::from_args(bad.0, bad.1, bad.2, bad.3).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn keeps_by_bangkok_day() {
        let m = r(Some("2026-10"), None, None, None);
        assert!(m.keeps("2026-10-01T00:00:00.000+07:00"));
        assert!(m.keeps("2026-10-31T23:59:59.999+07:00"));
        assert!(!m.keeps("2026-09-30T23:59:59.999+07:00"));
        assert!(!m.keeps("2026-11-01T00:00:00.000+07:00"));
        assert!(!m.keeps(""), "a row with no time is never in a range");
        // 2026-09-30T17:30Z is already 2026-10-01 in Bangkok
        assert!(m.keeps(&crate::engine::stamp(Some(chrono::DateTime::parse_from_rfc3339("2026-09-30T17:30:00Z").unwrap().timestamp_millis())).ts_th));
        let oct1 = chrono::DateTime::parse_from_rfc3339("2026-10-01T00:00:00+07:00").unwrap().timestamp_millis();
        assert!(m.may_hold(oct1));
        assert!(m.may_hold(oct1 - 3_600_000), "an hour before: within the day of slack");
        assert!(!m.may_hold(oct1 - 3 * 86_400_000));
    }

    #[test]
    fn a_folder_keeps_its_range() {
        let d = std::env::temp_dir().join(format!("relic-scan-test-{}-range-folder", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        let oct = r(Some("2026-10"), None, None, None);
        assert_eq!(Range::settle(&d, Some(oct.clone()), false).unwrap(), Some(oct.clone()));
        assert!(!d.join("_range").exists(), "a dry run writes nothing");
        assert_eq!(Range::settle(&d, Some(oct.clone()), true).unwrap(), Some(oct.clone()));
        assert_eq!(Range::settle(&d, None, true).unwrap(), Some(oct.clone()), "no flag: the folder's own range");
        assert_eq!(Range::settle(&d, Some(oct.clone()), true).unwrap(), Some(oct.clone()));
        assert!(Range::settle(&d, Some(r(Some("2026-09"), None, None, None)), true).is_err(), "another range: refused");
        let full = d.join("full");
        fs::create_dir_all(&full).unwrap();
        fs::write(full.join("_state.tsv"), "path\n").unwrap();
        assert_eq!(Range::settle(&full, None, true).unwrap(), None);
        assert!(Range::settle(&full, Some(oct), true).is_err(), "a full index never takes a range");
        let _ = fs::remove_dir_all(&d);
    }
}
