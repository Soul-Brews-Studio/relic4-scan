//! `relic-scan run`: scan, diff against OUT/_state.tsv, parse only what changed, merge the touched months, then
//! meta, map and the scan log. A dry run (`--dry-run`, the page's Check changes) stops after the diff.

use crate::*;

/// What a run found (and, unless it was a dry run, read): the diff between the disk and OUT/_state.tsv.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Report {
    pub dry_run: bool,
    pub new: u64,
    pub grown: u64,
    pub rewritten: u64,
    pub unchanged: u64,
    pub gone: u64,
    pub before_range: u64,
    /// files and bytes an index reads (for a dry run: would read)
    pub files: u64,
    pub bytes: u64,
    pub scan_secs: f64,
    pub secs: f64,
    /// unix seconds when it finished
    pub at: u64,
    /// folders the walk was refused (no permission)
    pub unreadable: u64,
    /// stopped on the page before anything was written
    pub stopped: bool,
    /// its line in OUT/_scans.jsonl (None when the log could not be written, e.g. a check of a folder not made yet)
    pub log: Option<usize>,
}


/// Runs `f` while a line on stderr shows the scan's live counters, so the walk never looks frozen.
fn with_ticker<T>(f: impl FnOnce() -> T) -> T {
    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
    let done = AtomicBool::new(false);
    let t0 = std::time::Instant::now();
    std::thread::scope(|sc| {
        sc.spawn(|| {
            let spin = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let mut i = 0;
            while !done.load(Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(200));
                if done.load(Relaxed) {
                    break;
                }
                ui::live(&format!("  {} {} ", ui::pink("scan"), ui::cyan(&spin[i % 10].to_string())), 9,
                         &format!("{} entries walked · {} homes · {} .jsonl listed · {:.0}s", num(scan::WALKED.load(Relaxed)),
                                  num(scan::HOMES.load(Relaxed)), num(scan::LISTED.load(Relaxed)), t0.elapsed().as_secs_f64()));
                i += 1;
            }
        });
        let r = f();
        done.store(true, Relaxed);
        if t0.elapsed().as_millis() >= 200 {
            ui::live("", 0, "");
            eprint!("\r");
        }
        r
    })
}

pub fn run_cmd(a: RunArgs) {
    let _ = run_with(a, None);
}

/// What a run is doing now, for /api/status: 0 idle, 1 scan, 2 parse, 3 merge, 4 meta + map.
pub static PHASE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Set by the page's Stop button. The walk and the parse check it; a run that sees it before the merge stops with
/// nothing written (the next run reads the same bytes again). Once the merge starts the run finishes.
pub static CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `run` with the OUT lock already taken (the server takes it before it answers, so a busy OUT is a 409, not an exit).
/// A dry run takes no lock and writes nothing. `serving` makes a bad argument panic (the server reports it) instead of exit.
pub fn run_with(a: RunArgs, lock: Option<std::fs::File>) -> Report {
    let serving = lock.is_some();
    run_inner(a, lock, serving)
}

pub fn run_inner(a: RunArgs, lock: Option<std::fs::File>, serving: bool) -> Report {
    use std::sync::atomic::Ordering::Relaxed;
    // the server catches a panic and reports it; the CLI says why and exits
    let fail = |e: String| -> ! {
        if serving {
            panic!("{e}");
        }
        eprintln!("relic-scan: {e}");
        std::process::exit(1);
    };
    let (asked, out_path) = match (a.range(), a.out_dir()) {
        (Ok(g), Ok(o)) => (g, o),
        (Err(e), _) | (_, Err(e)) => fail(e),
    };
    let RunArgs { roots, depth, threads, exclude, full, agent, jt_compat, only, kind, dry_run, .. } = a;
    let out = out_path;
    let t0 = std::time::Instant::now();
    let outp = std::path::Path::new(&out);
    let _lock = match lock {
        Some(l) => Some(l),
        None if dry_run => None,
        None => Some(lock_out(outp)),
    };
    let range = range::Range::settle(outp, asked, !dry_run).unwrap_or_else(|e| fail(e));
    let out_flag = if out != out_dir(DEFAULT_OUT, &kind) { format!(" --out {}", tilde(&out)) } else { String::new() };
    if let Some(g) = &range {
        eprintln!("run: only rows from {} (Bangkok days) -> {}", g.human(), tilde(&out));
    }
    PHASE.store(1, Relaxed);
    CANCEL.store(false, Relaxed);
    for c in [&scan::WALKED, &scan::HOMES, &scan::LISTED, &parse::DONE, &parse::TOTAL] {
        c.store(0, Relaxed);
    }
    // the Settings page's choices, unless a flag says otherwise (--root replaces the roots, --exclude adds, --depth wins)
    let cfg = config::load(outp);
    let roots = if roots.is_empty() { cfg.roots.clone() } else { roots };
    let exclude: Vec<String> = exclude.into_iter().chain(cfg.exclude.iter().cloned()).collect();
    let depth = if depth == 8 { cfg.depth } else { depth };
    let agent = agent.unwrap_or_else(default_agent);
    let _ = scan::take_unreadable();
    let _ = scan::take_vaults();
    let mut r = with_ticker(|| scan::run(&roots, depth, threads, &exclude, false));
    let unreadable = scan::take_unreadable();
    let vaults = scan::take_vaults();
    if !unreadable.is_empty() {
        eprintln!("run: {} folders could not be read (no permission); a transcript inside them is not seen", num(unreadable.len() as u64));
    }
    if !kind.is_empty() {
        r.files.retain(|f| kind.contains(&f.kind));
        r.loose.retain(|f| kind.iter().any(|k| *k == f.agent));
    }
    let t_scan = t0.elapsed().as_secs_f64();
    let mut st = state::load(outp);
    let (mut new, mut grown, mut rewritten, mut unchanged) = (0u64, 0u64, 0u64, 0u64);
    let mut jobs = vec![];
    let mut seen = std::collections::HashSet::new();
    let mut odd = 0u64;
    let mut early = 0u64;
    let mut changes: Vec<scanlog::Change> = vec![];
    for f in &r.files {
        if !only.is_empty() && !only.iter().any(|o| f.path.contains(o.as_str())) {
            continue;
        }
        if range.as_ref().is_some_and(|g| !g.may_hold(f.mtime)) {
            early += 1;
            continue;
        }
        if f.verdict == "not-transcript" {
            continue;
        }
        // _state.tsv is tab- and line-separated: such a path could not be saved, so it would be re-read every run
        if f.path.contains(['\t', '\n', '\r']) {
            odd += 1;
            continue;
        }
        seen.insert(f.path.clone());
        let mut status = "new";
        let start = match st.get(&f.path) {
            None => {
                if full && sniff::sniff(&f.path).verdict == "not-transcript" {
                    continue;
                }
                new += 1;
                Some(parse::Ctx::default())
            }
            // unchanged first: same inode, same size -> no read at all (the prefix is checked only on growth)
            Some(s) if s.dev == f.dev && s.ino == f.ino && f.size == s.ctx.offset => {
                unchanged += 1;
                None
            }
            Some(s) if s.dev != f.dev || s.ino != f.ino || f.size < s.ctx.offset
                || state::prefix_hash(&f.path, s.prefix_len) != Some(s.prefix_hash) => {
                rewritten += 1;
                status = "rewritten";
                Some(parse::Ctx::default())
            }
            Some(s) => {
                grown += 1;
                status = "grown";
                Some(s.ctx.clone())
            }
        };
        if let Some(ctx) = start {
            changes.push((status, f.path.clone(), f.size, ctx.offset));
            jobs.push(parse::Job { path: f.path.clone(), size: f.size, verdict: f.verdict.clone(), ctx });
        }
    }
    let filtered = !only.is_empty() || !kind.is_empty();
    let gone_paths: Vec<&String> = if !kind.is_empty() { vec![] } else {
        st.keys().filter(|k| !seen.contains(*k) && (only.is_empty() || only.iter().any(|o| k.contains(o.as_str())))).collect()
    };
    let gone = gone_paths.len();
    changes.extend(gone_paths.iter().map(|p| ("gone", p.to_string(), 0, st[*p].size)));
    if !filtered {
        st.retain(|k, _| seen.contains(k));
    }
    if early > 0 {
        eprintln!("run: skipped {} files last written before the range", num(early));
    }
    if odd > 0 {
        eprintln!("run: skipped {} files whose path holds a tab or a newline", num(odd));
    }
    let bytes: u64 = jobs.iter().map(|j| j.size.saturating_sub(j.ctx.offset)).sum();
    eprintln!("run: scan {:.1}s · new {} · grown {} · rewritten {} · unchanged {} · gone {} · {:.2} GB to read",
              t_scan, num(new), num(grown), num(rewritten), num(unchanged), num(gone as u64), bytes as f64 / 1e9);
    let stopped = |r: Report| -> Report {
        eprintln!("run: stopped before writing anything; the next run reads the same bytes again");
        PHASE.store(0, Relaxed);
        Report { secs: t0.elapsed().as_secs_f64(), stopped: true, ..r }
    };
    if CANCEL.load(Relaxed) {
        return stopped(Report { dry_run, ..Default::default() });
    }
    let report = Report {
        dry_run, new, grown, rewritten, unchanged, gone: gone as u64, before_range: early, files: jobs.len() as u64, bytes,
        unreadable: unreadable.len() as u64,
        scan_secs: t_scan, ..Default::default()
    };
    let range_label = range.as_ref().map(|g| g.label.clone());
    let (kind_l, only_l) = (kind.clone(), only.clone());
    // every run and every check leaves one line in OUT/_scans.jsonl, with the files it read (or would read)
    let finish = |mut r: Report, changes: Vec<scanlog::Change>, totals: Option<(i64, i64)>| {
        r.secs = t0.elapsed().as_secs_f64();
        r.at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let mut e = serde_json::to_value(&r).unwrap_or_default();
        if let Some(o) = e.as_object_mut() {
            o.remove("dry_run");
            o.remove("log");
            o.insert("job".into(), serde_json::json!(if r.dry_run { "check" } else { "index" }));
            o.insert("range".into(), serde_json::json!(range_label));
            o.insert("kind".into(), serde_json::json!(kind_l));
            o.insert("only".into(), serde_json::json!(only_l));
            if let Some((rows, sessions)) = totals {
                o.insert("rows".into(), serde_json::json!(rows));
                o.insert("sessions".into(), serde_json::json!(sessions));
            }
        }
        r.log = scanlog::append(outp, e, changes);
        PHASE.store(0, Relaxed);
        r
    };
    if dry_run {
        let what = if jobs.is_empty() { "nothing to read: the index is up to date".to_string() }
            else { format!("an index would read {} files, {:.2} GB", num(jobs.len() as u64), bytes as f64 / 1e9) };
        eprintln!("{}", ui::card(&ui::logo(), &[
            format!("{} · {}", ui::bold("dry run"), what),
            format!("new {} · grown {} · rewritten {} · unchanged {} · gone {}", num(new), num(grown), num(rewritten), num(unchanged), num(gone as u64)),
            ui::dim(&format!("scan {:.1}s · nothing written", t_scan)),
            format!("{} {}{}", ui::dim("→"), tilde(&out), range.as_ref().map(|g| ui::dim(&format!(" · only {}", g.human()))).unwrap_or_default()),
        ]));
        return finish(report, changes, None);
    }
    let sizes: std::collections::HashMap<String, (u64, u64, u64)> =
        r.files.iter().map(|f| (f.path.clone(), (f.dev, f.ino, f.size))).collect();
    PHASE.store(2, Relaxed);
    let done = if jobs.is_empty() { Default::default() } else { parse::run(jobs, outp, &agent, threads, jt_compat, false, range.as_ref()) };
    if CANCEL.load(Relaxed) && PHASE.load(Relaxed) == 2 {
        return stopped(report); // parse::run stopped before its merge: OUT and the state are as they were
    }
    for (p, c) in done {
        let (dev, ino, size) = sizes.get(&p).copied().unwrap_or_default();
        let plen = c.offset.min(state::PREFIX);
        let ph = state::prefix_hash(&p, plen).unwrap_or(0);
        st.insert(p, state::FileState { dev, ino, size, ctx: c, prefix_len: plen, prefix_hash: ph });
    }
    state::save(outp, &st);
    let t_parse = t0.elapsed().as_secs_f64() - t_scan;
    let t1 = std::time::Instant::now();
    PHASE.store(4, Relaxed);
    // the folders the walk was refused, for Health (the list may be long: the page shows the first ones)
    let _ = std::fs::create_dir_all(outp.join("meta"));
    let _ = std::fs::write(outp.join("meta").join("unreadable.json"),
                           serde_json::json!({"count": unreadable.len(), "folders": unreadable.iter().take(500).collect::<Vec<_>>()}).to_string());
    meta::jsonl(outp, &r, &st);
    // every ψ vault and its notes, as plain JSON (meta/vaults.json)
    let tv = std::time::Instant::now();
    let vs = crate::vault::index(vaults, outp, threads);
    eprintln!("vaults: {} ψ vaults · {} notes · {:.1} GB (+ {} worktree {}) in {:.1}s -> {}/meta/vaults.json", num(vs.vaults as u64),
              num(vs.notes), vs.bytes as f64 / 1e9, num(vs.worktrees as u64), if vs.worktrees == 1 { "copy" } else { "copies" },
              tv.elapsed().as_secs_f64(), out);
    let m = meta::build(outp, &agent, &st, threads);
    let t_meta = t1.elapsed().as_secs_f64();
    let t2 = std::time::Instant::now();
    map::build(outp, &agent);
    let t_map = t2.elapsed().as_secs_f64();
    eprintln!("run: done in {:.1}s · state {} files -> {}/_state.tsv", t0.elapsed().as_secs_f64(), num(st.len() as u64), out);
    let kind_flag = if !out_flag.is_empty() { out_flag } else { kind.iter().map(|k| format!(" --kind {k}")).collect() };
    let s = |x: f64| format!("{x:.1}s");
    eprintln!("{}", ui::card(&ui::logo(), &[
        format!("{} sessions · {} rows · {} files · {:.1} GB read", ui::bold(&num(m.sessions as u64)), ui::bold(&num(m.rows as u64)), num(m.files as u64),
                bytes as f64 / 1e9),
        format!("new {} · grown {} · rewritten {} · unchanged {}", num(new), num(grown), num(rewritten), num(unchanged)),
        ui::dim(&format!("scan {} · parse+merge {} · meta {} · map {} · total {}", s(t_scan), s(t_parse), s(t_meta), s(t_map),
                         s(t0.elapsed().as_secs_f64()))),
        format!("{} {}{}", ui::dim("→"), tilde(&out), range.as_ref().map(|g| ui::dim(&format!(" · only {}", g.human()))).unwrap_or_default()),
        format!("{} relic-scan serve{kind_flag}   {} relic-scan tui{kind_flag}", ui::pink("▸"), ui::pink("▸")),
    ]));
    finish(report, changes, Some((m.rows, m.sessions)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn args(root: &std::path::Path, out: &std::path::Path, dry_run: bool) -> RunArgs {
        RunArgs {
            out: out.display().to_string(), roots: vec![root.display().to_string()], depth: 8, threads: 2, exclude: vec![], full: false,
            agent: Some("t@x".into()), jt_compat: false, only: vec![], kind: vec![], month: None, day: None, from: None, to: None, dry_run,
        }
    }

    #[test]
    fn a_dry_run_reports_the_diff_and_writes_nothing() {
        let d = std::env::temp_dir().join(format!("relic-scan-test-{}-dry", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let (root, out) = (d.join("root"), d.join("out"));
        let proj = root.join(".claude/projects/-tmp-x");
        std::fs::create_dir_all(&proj).unwrap();
        let f = proj.join("0b867d73-405b-4b03-8c46-aee7648f1bd7.jsonl");
        let line = r#"{"type":"user","sessionId":"s1","timestamp":"2026-07-01T00:00:00.000Z","message":{"role":"user","content":"hello"}}"#;
        std::fs::write(&f, format!("{line}\n")).unwrap();

        let r = run_inner(args(&root, &out, true), None, false);
        assert_eq!((r.new, r.files, r.unchanged), (1, 1, 0));
        assert!(r.dry_run && r.bytes > 0);
        assert!(!out.join("_state.tsv").exists() && !out.join(".lock").exists(), "a dry run writes nothing");

        let r = run_inner(args(&root, &out, false), None, false);
        assert_eq!((r.new, r.dry_run), (1, false));
        assert!(out.join("_state.tsv").exists());

        let r = run_inner(args(&root, &out, true), None, false);
        assert_eq!((r.new, r.grown, r.unchanged, r.files, r.bytes), (0, 0, 1, 0, 0));

        let more = format!("{}\n", line.replace("hello", "again"));
        std::fs::OpenOptions::new().append(true).open(&f).unwrap().write_all(more.as_bytes()).unwrap();
        let r = run_inner(args(&root, &out, true), None, false);
        assert_eq!((r.grown, r.files, r.bytes), (1, 1, more.len() as u64), "a check sees exactly the appended bytes");
        // the log: the index and the two checks after it (the first check had no folder to write into)
        let log = scanlog::read(&out);
        assert_eq!(log.iter().map(|e| e["job"].as_str().unwrap()).collect::<Vec<_>>(), ["index", "check", "check"]);
        assert_eq!(log[0]["diff"][0][0], "new");
        assert_eq!(log[2]["diff"], serde_json::json!([["grown", f.display().to_string(), std::fs::metadata(&f).unwrap().len(),
                                                       std::fs::metadata(&f).unwrap().len() - more.len() as u64]]));
        assert!(log[0]["rows"].as_i64().unwrap() > 0);
        let _ = std::fs::remove_dir_all(&d);
    }
}
