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

#[test]
fn mirrors_fold_empty_thinking_stays_and_omp_results_are_tools_only() {
    let d = tmp("fixes");
    // the same two lines in two homes, except the mirror recorded another cwd: the hashes differ, the messages do not
    let lines_in = |cwd: &str| {
        format!(r#"{{"type":"user","sessionId":"m1","cwd":"{cwd}","timestamp":"2026-07-05T01:00:00.000Z","message":{{"role":"user","content":"same words"}}}}"#)
            + "\n"
            + &format!(r#"{{"type":"assistant","sessionId":"m1","cwd":"{cwd}","timestamp":"2026-07-05T01:00:01.000Z","message":{{"role":"assistant","content":[{{"type":"thinking","thinking":"","signature":"abc"}},{{"type":"tool_use","id":"tu1","name":"Bash","input":{{"command":"ls"}}}}]}}}}"#)
            + "\n"
    };
    let (a, b, o) = (d.join("home/m1.jsonl"), d.join("mirror/m1.jsonl"), d.join(".omp/s1.jsonl"));
    for p in [&a, &b, &o] {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
    }
    fs::write(&a, lines_in("/home/x/repo")).unwrap();
    fs::write(&b, lines_in("/Users/x/repo")).unwrap();
    // omp writes a tool result as a message of its own
    fs::write(&o, r#"{"type":"message","id":"x1","timestamp":"2026-07-05T02:00:00.000Z","message":{"role":"toolResult","toolCallId":"c1","toolName":"glob","content":[{"type":"text","text":"found 3 files"}],"isError":false}}"#.to_string() + "\n").unwrap();
    let job = |p: &PathBuf| Job { path: p.to_str().unwrap().to_string(), size: fs::metadata(p).unwrap().len(), verdict: String::new(), ctx: Ctx::default() };
    let out = d.join("out");
    run(vec![job(&a), job(&b), job(&o)], &out, "t@x", 2, false, false, None);
    let rows = month_rows(&out.join("2026-07.parquet"));
    let user: Vec<&String> = rows.iter().filter(|r| r.contains("|same words|")).collect();
    assert_eq!(user.len(), 1, "the mirrored message is one row: {rows:#?}");
    assert!(user[0].contains("home/m1.jsonl:1") && user[0].contains("mirror/m1.jsonl:1"), "both copies stay in sources: {}", user[0]);
    assert_eq!(rows.iter().filter(|r| r.contains("|tu1|")).count(), 1, "the mirrored tool call is one row");
    let think: Vec<&String> = rows.iter().filter(|r| r.starts_with("thinking|")).collect();
    assert_eq!(think.len(), 1, "a signature-only thinking block is a thinking row");
    assert!(think[0].contains("|thinking|") && think[0].contains("|2026-07-05T01:00:01.000Z|"), "{}", think[0]);
    assert!(!rows.iter().any(|r| r.contains("toolResult")), "an omp tool result is not a text row: {rows:#?}");
    let omp: Vec<&String> = rows.iter().filter(|r| r.contains("found 3 files")).collect();
    assert_eq!(omp.len(), 1);
    assert!(omp[0].starts_with("tools|") && omp[0].contains("|result|c1|glob|"), "{}", omp[0]);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn model_msg_id_and_out_tokens_once_per_response() {
    let d = tmp("model");
    let (c, m, x, o) = (d.join(".claude/p/c1.jsonl"), d.join("mirror/.claude/p/c1.jsonl"), d.join(".codex/rollout-x.jsonl"), d.join(".omp/o1.jsonl"));
    for p in [&c, &m, &x, &o] {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
    }
    // Claude: one response written as two lines (thinking, then text and a tool call), usage repeated on both
    let msg = |content: &str, out: i64| format!(r#"{{"type":"assistant","sessionId":"c1","timestamp":"2026-07-06T01:00:01.000Z","message":{{"id":"msg_A","model":"claude-opus-4-8","role":"assistant","content":[{content}],"usage":{{"input_tokens":3,"output_tokens":{out},"cache_read_input_tokens":100,"cache_creation_input_tokens":5}}}}}}"#);
    let claude = format!(r#"{{"type":"user","sessionId":"c1","timestamp":"2026-07-06T01:00:00.000Z","message":{{"role":"user","content":"go"}}}}"#)
        + "\n" + &msg(r#"{"type":"thinking","thinking":"plan"}"#, 7) + "\n"
        + &msg(r#"{"type":"text","text":"done"},{"type":"tool_use","id":"tu9","name":"Bash","input":{}}"#, 40) + "\n";
    fs::write(&c, &claude).unwrap();
    fs::write(&m, &claude).unwrap(); // a mirror: the same response again, counted once
    // Codex: the model is named per turn; its messages and calls carry none
    fs::write(&x, [
        r#"{"type":"session_meta","timestamp":"2026-07-06T02:00:00.000Z","payload":{"id":"x1"}}"#,
        r#"{"type":"turn_context","timestamp":"2026-07-06T02:00:01.000Z","payload":{"model":"gpt-6-astra"}}"#,
        r#"{"type":"response_item","timestamp":"2026-07-06T02:00:02.000Z","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"ask"}]}}"#,
        r#"{"type":"response_item","timestamp":"2026-07-06T02:00:03.000Z","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"answer"}]}}"#,
        r#"{"type":"response_item","timestamp":"2026-07-06T02:00:04.000Z","payload":{"type":"function_call","call_id":"fc1","name":"shell","arguments":"{}"}}"#,
        r#"{"type":"response_item","timestamp":"2026-07-06T02:00:05.000Z","payload":{"type":"function_call_output","call_id":"fc1","output":"ok"}}"#,
    ].join("\n") + "\n").unwrap();
    // omp: responseId and usage.output
    fs::write(&o, r#"{"type":"message","id":"o1","timestamp":"2026-07-06T03:00:00.000Z","message":{"role":"assistant","model":"glm-5.3","responseId":"msg_O","usage":{"input":11,"output":692,"cacheRead":50,"cacheWrite":2},"content":[{"type":"text","text":"omp says"}]}}"#.to_string() + "\n").unwrap();
    let job = |p: &PathBuf| Job { path: p.to_str().unwrap().to_string(), size: fs::metadata(p).unwrap().len(), verdict: String::new(), ctx: Ctx::default() };
    let out = d.join("out");
    run(vec![job(&c), job(&m), job(&x), job(&o)], &out, "t@x", 2, false, false, None);
    let r = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(fs::File::open(out.join("2026-07.parquet")).unwrap()).unwrap().build().unwrap();
    let mut rows: Vec<(String, String, String, String, String, Option<i64>)> = vec![];
    for b in r {
        let b = b.unwrap();
        let s = |n: &str| merge::col_str(&b, n).clone();
        let (who, text, body, model, id) = (s("role"), s("text"), s("body"), s("model"), s("msg_id"));
        let kind = s("kind");
        let tok = b.column_by_name("out_tokens").unwrap().as_any().downcast_ref::<Int64Array>().unwrap().clone();
        for i in 0..b.num_rows() {
            let what = if kind.is_null(i) || kind.value(i).is_empty() { who.value(i).to_string() } else { kind.value(i).to_string() };
            let said = if text.is_null(i) { body.value(i).to_string() } else { text.value(i).to_string() };
            let v = |a: &StringArray| if a.is_null(i) { "-".to_string() } else { a.value(i).to_string() };
            rows.push((what, said, v(&model), v(&id), String::new(), (!tok.is_null(i)).then(|| tok.value(i))));
        }
    }
    let get = |said: &str| rows.iter().find(|r| r.1 == said).unwrap_or_else(|| panic!("no row {said}: {rows:#?}"));
    assert_eq!((get("go").2.as_str(), get("go").5), ("-", None), "the user's text names no model");
    for said in ["plan", "done"] {
        assert_eq!((get(said).2.as_str(), get(said).3.as_str()), ("claude-opus-4-8", "msg_A"), "{said}");
    }
    let claude_tokens: Vec<i64> = rows.iter().filter(|r| r.3 == "msg_A").filter_map(|r| r.5).collect();
    assert_eq!(claude_tokens, vec![40], "one row per response holds its tokens, the largest copy: {rows:#?}");
    // the input and cache tokens ride on the same single row as the output tokens
    let r = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(fs::File::open(out.join("2026-07.parquet")).unwrap()).unwrap().build().unwrap();
    let mut sums = [0i64; 4];
    for b in r {
        let b = b.unwrap();
        for (k, n) in ["out_tokens", "in_tokens", "cache_read", "cache_write"].iter().enumerate() {
            let a = b.column_by_name(n).unwrap().as_any().downcast_ref::<Int64Array>().unwrap().clone();
            sums[k] += a.iter().flatten().sum::<i64>();
        }
    }
    assert_eq!(sums, [40 + 692, 3 + 11, 100 + 50, 5 + 2], "each response once, all four counters");
    assert_eq!(get("answer").2, "gpt-6-astra");
    assert_eq!(rows.iter().find(|r| r.0 == "call" && r.2 == "gpt-6-astra").map(|r| r.3.as_str()), Some("-"), "a Codex call: model, no id");
    assert_eq!((get("ask").2.as_str(), get("ok").2.as_str()), ("-", "-"), "user text and tool results name no model");
    assert_eq!((get("omp says").2.as_str(), get("omp says").3.as_str(), get("omp says").5), ("glm-5.3", "msg_O", Some(692)));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn a_session_says_where_it_ran_what_it_is_called_and_on_which_branch() {
    let d = tmp("sessmeta");
    let (c, x) = (d.join("c1.jsonl"), d.join("rollout-x.jsonl"));
    fs::write(&c, [
        r#"{"type":"user","sessionId":"c1","cwd":"/opt/Code/repo-a","gitBranch":"main","version":"2.1.1","timestamp":"2026-07-06T01:00:00.000Z","message":{"role":"user","content":"hi"}}"#,
        r#"{"type":"custom-title","customTitle":"first name","sessionId":"c1"}"#,
        r#"{"type":"user","sessionId":"c1","cwd":"/opt/Code/repo-b","gitBranch":"feat/x","version":"2.1.2","timestamp":"2026-07-06T01:01:00.000Z","message":{"role":"user","content":"go"}}"#,
        r#"{"type":"custom-title","customTitle":"neo-oracle","sessionId":"c1"}"#,
    ].join("\n") + "\n").unwrap();
    fs::write(&x, [
        r#"{"type":"session_meta","timestamp":"2026-07-06T02:00:00.000Z","payload":{"id":"x1","cwd":"/home/nat/proj"}}"#,
        r#"{"type":"response_item","timestamp":"2026-07-06T02:00:03.000Z","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"answer"}]}}"#,
    ].join("\n") + "\n").unwrap();
    let st = Stats::default();
    let meta = |p: &PathBuf| parse_file(p.to_str().unwrap(), &Ctx::default(), false, &st, &mut |_| {}).unwrap().meta;
    let m = meta(&c);
    assert_eq!((m.cwd.as_str(), m.title.as_str(), m.branch.as_str(), m.version.as_str()), ("/opt/Code/repo-a", "neo-oracle", "feat/x", "2.1.2"),
               "the first cwd stays; title, branch and version follow the newest line");
    assert_eq!(meta(&x).cwd, "/home/nat/proj");
    // a file read in two goes on from where it was left, with the meta it had
    let half = parse_file(c.to_str().unwrap(), &Ctx::default(), false, &st, &mut |_| {}).unwrap();
    let again = parse_file(c.to_str().unwrap(), &half, false, &st, &mut |_| {}).unwrap();
    assert_eq!(again.meta, half.meta);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn a_zst_rollout_gives_the_rows_of_the_plain_one_and_is_read_once() {
    let d = tmp("zst");
    let body = [
        r#"{"type":"session_meta","timestamp":"2026-07-06T02:00:00.000Z","payload":{"id":"01a0f546-a949-74f3-bf81-5a2f61763cc7","cwd":"/home/nat/proj"}}"#,
        r#"{"type":"turn_context","timestamp":"2026-07-06T02:00:01.000Z","payload":{"model":"gpt-6-astra"}}"#,
        r#"{"type":"response_item","timestamp":"2026-07-06T02:00:02.000Z","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"ask"}]}}"#,
        r#"{"type":"response_item","timestamp":"2026-07-06T02:00:03.000Z","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"answer"}]}}"#,
    ].join("\n") + "\n";
    let name = "rollout-2026-07-06T09-00-00-01a0f546-a949-74f3-bf81-5a2f61763cc7.jsonl";
    let (plain, packed) = (d.join(name), d.join(format!("{name}.zst")));
    fs::write(&plain, &body).unwrap();
    fs::write(&packed, zstd::stream::encode_all(body.as_bytes(), 3).unwrap()).unwrap();
    assert!(crate::scan::is_jsonl(&plain) && crate::scan::is_jsonl(&packed) && !crate::scan::is_jsonl(&d.join("a.json")));
    let rows = |p: &PathBuf, ctx: &Ctx| {
        let mut v = vec![];
        let c = parse_file(p.to_str().unwrap(), ctx, false, &Stats::default(), &mut |r| v.push(format!("{}|{:?}|{:?}|{:?}|{:?}", r.s, r.role, r.text, r.model, r.sid))).unwrap();
        (v, c)
    };
    let (a, ca) = rows(&plain, &Ctx::default());
    let (b, cb) = rows(&packed, &Ctx::default());
    assert!(a.len() >= 2, "{a:?}");
    assert_eq!(a, b, "the packed file says what the plain one says");
    assert_eq!((ca.lines, &ca.session, &ca.meta.cwd), (cb.lines, &cb.session, &cb.meta.cwd));
    assert_eq!(cb.offset, fs::metadata(&packed).unwrap().len(), "a read archive's offset is its size on disk, so it counts as unchanged");
    assert_eq!(ca.chain, cb.chain, "the ledger sees the same lines");
    let (again, _) = rows(&packed, &cb);
    assert!(again.is_empty(), "a closed archive is not read twice");
    assert_eq!(crate::sniff::sniff(packed.to_str().unwrap()).verdict, "transcript");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn a_month_written_in_small_batches_holds_the_same_rows() {
    // one Arrow string array holds at most 2 GiB: a big month goes out in batches (RELIC_SCAN_BATCH_ROWS rows each)
    let d = tmp("batches");
    let src = d.join("s.jsonl");
    fs::write(&src, lines(2, 40)).unwrap();
    let sp = src.to_str().unwrap().to_string();
    let job = || Job { path: sp.clone(), size: fs::metadata(&src).unwrap().len(), verdict: String::new(), ctx: Ctx::default() };
    let (big, small) = (d.join("big"), d.join("small"));
    run(vec![job()], &big, "t@x", 2, false, false, None);
    // safe here: the other tests read the variable too, and any batch size gives the same rows
    unsafe { std::env::set_var("RELIC_SCAN_BATCH_ROWS", "7") };
    run(vec![job()], &small, "t@x", 2, false, false, None);
    unsafe { std::env::remove_var("RELIC_SCAN_BATCH_ROWS") };
    let (a, b) = (month_rows(&big.join("2026-07.parquet")), month_rows(&small.join("2026-07.parquet")));
    assert!(a.len() > 100, "{}", a.len());
    assert_eq!(a, b);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn a_channel_message_names_the_sessions_channel() {
    let m = engine::channel_of(r#"<channel source="plugin:discord:discord" chat_id="1551543230984757350" message_id="1" user="emraccoon" user_id="2" ts="2026-10-01T05:44:15.702Z"> hello </channel>"#);
    assert_eq!((m.via.as_str(), m.chat.as_str(), m.from_user.as_str()), ("plugin:discord:discord", "1551543230984757350", "emraccoon"));
    assert_eq!(engine::channel_of("no tag here"), engine::SessMeta::default());
    let d = tmp("channel");
    let p = d.join("c1.jsonl");
    fs::write(&p, [
        r#"{"type":"user","sessionId":"c1","timestamp":"2026-07-06T01:00:00.000Z","message":{"role":"user","content":"typed by hand"}}"#,
        r#"{"type":"user","sessionId":"c1","timestamp":"2026-07-06T01:00:05.000Z","message":{"role":"user","content":"<channel source=\"plugin:discord:discord\" chat_id=\"77\" user=\"nat\"> hi </channel>"}}"#,
        r#"{"type":"user","sessionId":"c1","timestamp":"2026-07-06T01:00:09.000Z","message":{"role":"user","content":"<channel source=\"plugin:telegram:telegram\" chat_id=\"88\" user=\"other\"> later </channel>"}}"#,
    ].join("\n") + "\n").unwrap();
    let c = parse_file(p.to_str().unwrap(), &Ctx::default(), false, &Stats::default(), &mut |_| {}).unwrap();
    assert_eq!((c.meta.via.as_str(), c.meta.chat.as_str(), c.meta.from_user.as_str()), ("plugin:discord:discord", "77", "nat"), "the first channel message names it");
    let _ = fs::remove_dir_all(&d);
}
