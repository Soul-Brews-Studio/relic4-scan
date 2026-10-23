//! One session's rows out of the month files, for the page's export (`GET /api/export`).
//!
//! The months to read come from the caller (the page knows when a session started and ended), so an export opens a
//! few month files, not all of them. Each file is read with a row filter on `sid`: only the matching rows have their
//! text decoded. Rows come out in the month file's own order (time, then stream). A subagent's lines carry its
//! parent's session id, so a main session's export holds its subagents' rows too.

use std::fs;
use std::path::Path;

use arrow::array::{Array, BooleanArray, RecordBatch, StringArray};
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::{ArrowPredicateFn, ParquetRecordBatchReaderBuilder, RowFilter};
use serde_json::json;

const COLS: [&str; 9] = ["s", "role", "type", "ts_th", "text", "kind", "name", "body", "err"];

#[derive(Debug, Default)]
pub struct Row {
    pub s: String,
    pub role: String,
    pub typ: String,
    pub ts: String,
    pub text: String,
    pub kind: String,
    pub name: String,
    pub body: String,
    pub err: bool,
}

/// A month name the caller may ask for: YYYY-MM or "undated" (never a path).
fn month_ok(m: &str) -> bool {
    m == "undated" || (m.len() == 7 && m.as_bytes()[4] == b'-' && m.chars().enumerate().all(|(i, c)| i == 4 || c.is_ascii_digit()))
}

pub fn session(out: &Path, sid: &str, months: &[String]) -> Result<Vec<Row>, String> {
    let mut rows = vec![];
    for m in months.iter().filter(|m| month_ok(m)) {
        let p = out.join(format!("{m}.parquet"));
        let Ok(f) = fs::File::open(&p) else { continue };
        let b = ParquetRecordBatchReaderBuilder::try_new(f).map_err(|e| format!("{}: {e}", p.display()))?;
        let leaves: Vec<String> = b.parquet_schema().columns().iter().map(|c| c.path().string()).collect();
        let at = |n: &str| leaves.iter().position(|l| l == n);
        let Some(sid_at) = at("sid") else { continue };
        let want: Vec<usize> = COLS.iter().filter_map(|c| at(c)).collect();
        let key = sid.to_string();
        let pred = ArrowPredicateFn::new(ProjectionMask::leaves(b.parquet_schema(), [sid_at]), move |batch: RecordBatch| {
            let a = batch.column(0).as_any().downcast_ref::<StringArray>().unwrap();
            Ok((0..a.len()).map(|i| Some(!a.is_null(i) && a.value(i) == key)).collect::<BooleanArray>())
        });
        let mask = ProjectionMask::leaves(b.parquet_schema(), want);
        let r = b.with_projection(mask).with_row_filter(RowFilter::new(vec![Box::new(pred)])).build().map_err(|e| e.to_string())?;
        for batch in r {
            let batch = batch.map_err(|e| e.to_string())?;
            let col = |n: &str| batch.column_by_name(n).and_then(|c| c.as_any().downcast_ref::<StringArray>().cloned());
            let (s, role, typ, ts, text, kind, name, body) =
                (col("s"), col("role"), col("type"), col("ts_th"), col("text"), col("kind"), col("name"), col("body"));
            let err = batch.column_by_name("err").and_then(|c| c.as_any().downcast_ref::<BooleanArray>().cloned());
            let g = |a: &Option<StringArray>, i: usize| a.as_ref().filter(|a| !a.is_null(i)).map(|a| a.value(i).to_string()).unwrap_or_default();
            for i in 0..batch.num_rows() {
                rows.push(Row {
                    s: g(&s, i), role: g(&role, i), typ: g(&typ, i), ts: g(&ts, i), text: g(&text, i), kind: g(&kind, i),
                    name: g(&name, i), body: g(&body, i), err: err.as_ref().is_some_and(|e| !e.is_null(i) && e.value(i)),
                });
            }
        }
    }
    Ok(rows)
}

pub fn jsonl(rows: &[Row]) -> String {
    let mut out = String::new();
    for r in rows {
        let v = json!({"ts": r.ts, "stream": r.s, "role": r.role, "type": r.typ, "text": (!r.text.is_empty()).then_some(&r.text),
                       "kind": (!r.kind.is_empty()).then_some(&r.kind), "name": (!r.name.is_empty()).then_some(&r.name),
                       "body": (!r.body.is_empty()).then_some(&r.body), "error": r.err.then_some(true)});
        out.push_str(&v.to_string());
        out.push('\n');
    }
    out
}

pub fn markdown(title: &str, sid: &str, rows: &[Row]) -> String {
    let (first, last) = (rows.first().map(|r| r.ts.as_str()).unwrap_or(""), rows.last().map(|r| r.ts.as_str()).unwrap_or(""));
    let mut out = format!("# {title}\n\n`{sid}` · {} rows · {} → {} (Bangkok)\n", rows.len(), first.get(..16).unwrap_or(first).replace('T', " "),
                          last.get(..16).unwrap_or(last).replace('T', " "));
    let fence = |t: &str| { let mut f = "```".to_string(); while t.contains(&f) { f.push('`'); } f };
    for r in rows {
        let when = r.ts.get(..16).unwrap_or(&r.ts).replace('T', " ");
        match r.s.as_str() {
            "tools" => {
                let what = if r.kind == "result" { if r.err { "tool error" } else { "tool result" } } else { "tool call" };
                let f = fence(&r.body);
                out += &format!("\n<details><summary>{what}{} · {when}</summary>\n\n{f}\n{}\n{f}\n\n</details>\n",
                                if r.name.is_empty() { String::new() } else { format!(" · {}", r.name) }, r.body);
            }
            "thinking" => out += &format!("\n### thinking · {when}\n\n> {}\n", r.text.replace('\n', "\n> ")),
            _ => out += &format!("\n### {} · {when}\n\n{}\n", if r.role == "assistant" { "AI" } else if r.role.is_empty() { "row" } else { &r.role }, r.text),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn months_are_names_not_paths() {
        assert!(super::month_ok("2026-10") && super::month_ok("undated"));
        for bad in ["../x", "2026-1", "2026/10", "meta/sessions", "2026-10.parquet"] {
            assert!(!super::month_ok(bad), "{bad}");
        }
    }
}
