//! relic-scan — one binary that finds every AI-agent transcript on this machine and (next phases) parses it
//! into one file per Bangkok month. See RESEARCH.md for the design and its sources.
//!
//!   relic-scan scan [--root DIR]... [--depth 8] [--json] [--index index.tsv]
//!       phase 1: find every agent home (.claude, .claude_bak, .claude2, .codex …) and index its .jsonl files
//!   relic-scan parse --index index.tsv --out DIR [--agent user@host] [--threads N] [--jt-compat]
//!       phase 2: parse every indexed file in parallel -> DIR/YYYY-MM.parquet, one file per Bangkok month
//!   relic-scan run --out DIR [--full] [--root DIR]... [--agent user@host]
//!       incremental: scan, diff against DIR/_state.tsv, parse only new bytes, rewrite only touched months,
//!       then meta (below)
//!   relic-scan run --month 2026-10 | --day 2026-10-22 | --from 2026-09-15 [--to 2026-10-15]
//!       only rows of those Bangkok days, into ~/.relic-scan/<kinds or all>@<range> (a folder that keeps its range)
//!   relic-scan up [--kind claude] [--port 8787]
//!       run (into ~/.relic-scan unless --out), then serve http://127.0.0.1:8787/map and /tree
//!   relic-scan serve [--out DIR] [--port 8787]
//!   relic-scan map --out DIR
//!       DIR/meta/{sessions,files}.parquet -> DIR/meta/map.html (session star map + query, one offline page)
//!   relic-scan meta --out DIR [--agent user@host]
//!       DIR/YYYY-MM.parquet + DIR/_state.tsv -> DIR/meta/files.parquet + DIR/meta/sessions.parquet

pub mod engine;
pub mod export;
pub mod map;
pub mod meta;
pub mod parse;
pub mod range;
pub mod scanlog;
pub mod scan;
pub mod serve;
pub mod sniff;
pub mod state;
pub mod tee;
pub mod tui;
pub mod ui;



/// Where run/meta/map/serve read and write when --out is not given.
pub const DEFAULT_OUT: &str = "~/.relic-scan";

#[derive(clap::Args)]
pub struct RunArgs {
    /// output directory: YYYY-MM.parquet per month, _state.tsv, meta/
    #[arg(long, default_value = DEFAULT_OUT)]
    pub out: String,
    #[arg(long = "root")]
    pub roots: Vec<String>,
    #[arg(long, default_value_t = 8)]
    pub depth: usize,
    #[arg(long, default_value_t = 0)]
    pub threads: usize,
    #[arg(long)]
    pub exclude: Vec<String>,
    /// content-sniff NEW files (unchanged ones keep their verdict from the light scan)
    #[arg(long)]
    pub full: bool,
    #[arg(long)]
    pub agent: Option<String>,
    #[arg(long)]
    pub jt_compat: bool,
    /// only files whose path contains this text (repeatable)
    #[arg(long)]
    pub only: Vec<String>,
    /// only homes of this kind: claude, codex, omp, omx, thclaws, gemini, pi (repeatable); loose .jsonl are kept
    /// when their content says the same agent. Without --out it writes ~/.relic-scan/<kind> (kinds joined by +),
    /// so it never mixes with a run over every kind.
    #[arg(long)]
    pub kind: Vec<String>,
    /// only rows of this Bangkok month, YYYY-MM. Without --out it writes ~/.relic-scan/<kinds or all>@<month>; the
    /// rest of each file is read and left behind, so a range never writes into a folder indexed without one
    #[arg(long, conflicts_with_all = ["day", "from", "to"])]
    pub month: Option<String>,
    /// only rows of this Bangkok day, YYYY-MM-DD (same folder rule as --month)
    #[arg(long, conflicts_with_all = ["from", "to"])]
    pub day: Option<String>,
    /// only rows on or after this Bangkok day, YYYY-MM-DD or YYYY-MM (same folder rule as --month)
    #[arg(long)]
    pub from: Option<String>,
    /// only rows on or before this Bangkok day, YYYY-MM-DD or YYYY-MM (same folder rule as --month)
    #[arg(long)]
    pub to: Option<String>,
    /// only scan and compare with the state: print what an index would read (new, grown, rewritten, gone, bytes),
    /// write nothing and take no lock
    #[arg(long)]
    pub dry_run: bool,
}

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
    /// stopped on the page before anything was written
    pub stopped: bool,
    /// its line in OUT/_scans.jsonl (None when the log could not be written, e.g. a check of a folder not made yet)
    pub log: Option<usize>,
}

impl RunArgs {
    /// The date range the flags ask for (None without --month/--day/--from/--to).
    pub fn range(&self) -> Result<Option<range::Range>, String> {
        range::Range::from_args(self.month.as_deref(), self.day.as_deref(), self.from.as_deref(), self.to.as_deref())
    }

    /// Where this run writes: --out, else ~/.relic-scan[/<kinds>], else with a range ~/.relic-scan/<kinds or all>@<range>.
    pub fn out_dir(&self) -> Result<String, String> {
        let Some(g) = self.range()? else { return Ok(out_dir(&self.out, &self.kind)) };
        if self.out != DEFAULT_OUT {
            return Ok(expand(&self.out));
        }
        let mut k = self.kind.clone();
        k.sort();
        k.dedup();
        let k = if k.is_empty() { "all".to_string() } else { k.join("+") };
        Ok(expand(&format!("{DEFAULT_OUT}/{k}@{}", g.label)))
    }
}

pub fn gb(b: u64) -> String {
    format!("{:.2}", b as f64 / 1e9)
}

pub fn num(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

pub fn tilde(p: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    match p.strip_prefix(&home) {
        Some(rest) if !home.is_empty() => format!("~{rest}"),
        _ => p.to_string(),
    }
}

pub fn default_agent() -> String {
    let user = std::env::var("USER").unwrap_or_default();
    let host = std::process::Command::new("hostname").arg("-s").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    format!("{user}@{host}")
}

/// A leading ~ means $HOME (clap defaults are not expanded by a shell).
pub fn expand(p: &str) -> String {
    match p.strip_prefix('~') {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("{}{rest}", std::env::var("HOME").unwrap_or_default()),
        _ => p.to_string(),
    }
}

/// The output directory. With --kind and no --out, each kind set gets its own folder (~/.relic-scan/claude), so a
/// kind-only run never shares a state with a run over every kind.
pub fn out_dir(out: &str, kind: &[String]) -> String {
    if kind.is_empty() || out != DEFAULT_OUT {
        return expand(out);
    }
    let mut k = kind.to_vec();
    k.sort();
    k.dedup();
    expand(&format!("{DEFAULT_OUT}/{}", k.join("+")))
}

/// One writer per output directory: a second `run` (cron, another shell) on the same OUT would delete this one's
/// raw parts and race it on _state.tsv. The lock is released when the returned file is dropped (or the process ends).
pub fn lock_out(outp: &std::path::Path) -> std::fs::File {
    try_lock_out(outp).unwrap_or_else(|e| {
        eprintln!("relic-scan: {e}");
        std::process::exit(1);
    })
}

/// The OUT lock, or why it cannot be had.
pub fn try_lock_out(outp: &std::path::Path) -> Result<std::fs::File, String> {
    std::fs::create_dir_all(outp).map_err(|e| format!("cannot create {}: {e}", outp.display()))?;
    let f = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(outp.join(".lock"))
        .map_err(|e| format!("cannot open {}/.lock: {e}", outp.display()))?;
    f.try_lock().map_err(|_| format!("another relic-scan is writing {} (wait for it, or use another --out)", outp.display()))?;
    Ok(f)
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
    let roots = if roots.is_empty() { vec!["~".to_string(), "/opt/Code".to_string()] } else { roots };
    let agent = agent.unwrap_or_else(default_agent);
    let mut r = with_ticker(|| scan::run(&roots, depth, threads, &exclude, false));
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
    meta::jsonl(outp, &r, &st);
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

/// `relic-scan` with no command: the logo, what is already indexed, and the commands that matter.
pub fn banner() {
    eprintln!("\n  {}  {}\n", ui::logo(), ui::dim(&format!("v{} · every AI-agent transcript on this machine", env!("CARGO_PKG_VERSION"))));
    let base = expand(DEFAULT_OUT);
    let mut outs: Vec<String> = vec![base.clone()];
    if let Ok(rd) = std::fs::read_dir(&base) {
        let mut subs: Vec<String> = rd.flatten().filter(|e| e.path().join("meta").is_dir()).map(|e| e.path().display().to_string()).collect();
        subs.sort();
        outs.extend(subs);
    }
    let mut any = false;
    for o in outs {
        let st = std::path::Path::new(&o).join("_state.tsv");
        let Ok(md) = std::fs::metadata(&st) else { continue };
        let files = std::fs::read_to_string(&st).map(|t| t.lines().count().saturating_sub(1)).unwrap_or(0);
        let age = md.modified().ok().and_then(|m| m.elapsed().ok()).map(|d| d.as_secs()).unwrap_or(0);
        let ago = if age < 3600 { format!("{} min ago", age / 60) } else if age < 86400 { format!("{} h ago", age / 3600) } else { format!("{} d ago", age / 86400) };
        eprintln!("  {} {}  {}", ui::cyan("●"), tilde(&o), ui::dim(&format!("{} files indexed · last run {ago}", num(files as u64))));
        any = true;
    }
    if !any {
        eprintln!("  {} nothing indexed yet", ui::dim("○"));
    }
    let cmd = |c: &str, what: &str| eprintln!("  {}  {}", ui::pink(&format!("{c:<34}")), ui::dim(what));
    eprintln!();
    cmd("relic-scan up --kind claude", "index Claude transcripts, then serve the map on 127.0.0.1:8787");
    cmd("relic-scan up", "the same over every agent (claude, codex, omp, omx, …)");
    cmd("relic-scan tui [--kind claude]", "browse sessions, repos and paths in the terminal");
    cmd("relic-scan serve [--kind claude]", "serve /map /tree /paths without indexing again");
    cmd("relic-scan run [--kind claude]", "index only (incremental: new bytes only)");
    cmd("relic-scan run --dry-run", "only check: what an index would read, nothing written");
    cmd("relic-scan run --month 2026-10", "only one month (or --day, --from/--to), in a folder of its own");
    cmd("relic-scan --help", "every command and flag");
    eprintln!();
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
