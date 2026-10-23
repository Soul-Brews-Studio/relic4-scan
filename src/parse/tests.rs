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
