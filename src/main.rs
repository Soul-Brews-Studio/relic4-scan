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
//!   relic-scan up [--kind claude] [--port 8787]
//!       run (into ~/.relic-scan unless --out), then serve http://127.0.0.1:8787/map and /tree
//!   relic-scan serve [--out DIR] [--port 8787]
//!   relic-scan map --out DIR
//!       DIR/meta/{sessions,files}.parquet -> DIR/meta/map.html (session star map + query, one offline page)
//!   relic-scan meta --out DIR [--agent user@host]
//!       DIR/YYYY-MM.parquet + DIR/_state.tsv -> DIR/meta/files.parquet + DIR/meta/sessions.parquet

mod engine;
mod map;
mod meta;
mod parse;
mod scan;
mod serve;
mod sniff;
mod state;

use std::io::Write;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Find and index every AI-agent transcript on this machine")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

/// Where run/meta/map/serve read and write when --out is not given.
const DEFAULT_OUT: &str = "~/.relic-scan";

#[derive(clap::Args)]
struct RunArgs {
    /// output directory: YYYY-MM.parquet per month, _state.tsv, meta/
    #[arg(long, default_value = DEFAULT_OUT)]
    out: String,
    #[arg(long = "root")]
    roots: Vec<String>,
    #[arg(long, default_value_t = 8)]
    depth: usize,
    #[arg(long, default_value_t = 0)]
    threads: usize,
    #[arg(long)]
    exclude: Vec<String>,
    /// content-sniff NEW files (unchanged ones keep their verdict from the light scan)
    #[arg(long)]
    full: bool,
    #[arg(long)]
    agent: Option<String>,
    #[arg(long)]
    jt_compat: bool,
    /// only files whose path contains this text (repeatable)
    #[arg(long)]
    only: Vec<String>,
    /// only homes of this kind: claude, codex, omp, omx, thclaws, gemini, pi (repeatable); loose .jsonl are kept
    /// when their content says the same agent. Without --out it writes ~/.relic-scan/<kind> (kinds joined by +),
    /// so it never mixes with a run over every kind.
    #[arg(long)]
    kind: Vec<String>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Find every agent home (.claude, .claude_bak, .claudebak, .claude2, .codex …) and index its .jsonl files
    Scan {
        /// where to look; repeatable (default: ~ and /opt/Code)
        #[arg(long = "root")]
        roots: Vec<String>,
        /// how deep below a root a home may sit
        #[arg(long, default_value_t = 8)]
        depth: usize,
        /// worker threads (0 = one per core)
        #[arg(long, default_value_t = 0)]
        threads: usize,
        /// print JSON (homes + totals) instead of the table
        #[arg(long)]
        json: bool,
        /// also write the file index (one line per file) for phase 2
        #[arg(long)]
        index: Option<String>,
        /// skip any path containing this text; repeatable (e.g. --exclude fixture-home)
        #[arg(long)]
        exclude: Vec<String>,
        /// full scan: read each file's first complete lines and prove it is a transcript (slower, exact).
        /// Default is the light scan: names, folder shapes and path rules only, no file contents.
        #[arg(long)]
        full: bool,
    },
    /// Incremental: scan, then parse only new/grown/rewritten files from where they were left, then meta + map
    Run(RunArgs),
    /// One command: `run`, then serve the map and the tree on 127.0.0.1 (http://127.0.0.1:PORT/map · /tree)
    Up {
        #[command(flatten)]
        run: RunArgs,
        #[arg(long, default_value_t = 8787)]
        port: u16,
    },
    /// Serve OUT/meta/map.html on 127.0.0.1: /map (star map) and /tree (homes -> repos -> sessions)
    Serve {
        #[arg(long, default_value = DEFAULT_OUT)]
        out: String,
        #[arg(long, default_value_t = 8787)]
        port: u16,
        /// the --kind the run used: picks ~/.relic-scan/<kind> when --out is not given
        #[arg(long)]
        kind: Vec<String>,
    },
    /// The month files + _state.tsv -> meta/files.parquet + meta/sessions.parquet (`run` does this as its last step)
    Meta {
        /// the directory `run` wrote
        #[arg(long, default_value = DEFAULT_OUT)]
        out: String,
        /// the --agent `run` used (default: user@host)
        #[arg(long)]
        agent: Option<String>,
        #[arg(long, default_value_t = 0)]
        threads: usize,
        /// the --kind the run used: picks ~/.relic-scan/<kind> when --out is not given
        #[arg(long)]
        kind: Vec<String>,
    },
    /// meta/sessions.parquet + meta/files.parquet -> meta/map.html: every session as a star, queryable in the browser
    Map {
        /// the directory `run` wrote
        #[arg(long, default_value = DEFAULT_OUT)]
        out: String,
        #[arg(long)]
        agent: Option<String>,
        /// the --kind the run used: picks ~/.relic-scan/<kind> when --out is not given
        #[arg(long)]
        kind: Vec<String>,
    },
    /// Parse every indexed file -> one Parquet file per Bangkok month (truth schema, copies folded)
    Parse {
        /// the file index from `scan --index`
        #[arg(long)]
        index: String,
        /// output directory: YYYY-MM.parquet per month
        #[arg(long)]
        out: String,
        /// the label written into machines/sources (default: user@host)
        #[arg(long)]
        agent: Option<String>,
        /// worker threads (0 = one per core)
        #[arg(long, default_value_t = 0)]
        threads: usize,
        /// behave like jt: a file stops at a line longer than jt's 8 MB chunk
        #[arg(long)]
        jt_compat: bool,
        /// only files whose path contains this text (repeatable)
        #[arg(long)]
        only: Vec<String>,
        /// keep the per-thread raw parts (out/_parts)
        #[arg(long)]
        keep_parts: bool,
        /// also parse files the content sniff called not-transcript (prompt logs, journals, omx logs)
        #[arg(long)]
        all: bool,
    },
}

fn gb(b: u64) -> String {
    format!("{:.2}", b as f64 / 1e9)
}

fn num(n: u64) -> String {
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

fn tilde(p: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    match p.strip_prefix(&home) {
        Some(rest) if !home.is_empty() => format!("~{rest}"),
        _ => p.to_string(),
    }
}

fn default_agent() -> String {
    let user = std::env::var("USER").unwrap_or_default();
    let host = std::process::Command::new("hostname").arg("-s").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    format!("{user}@{host}")
}

/// A leading ~ means $HOME (clap defaults are not expanded by a shell).
fn expand(p: &str) -> String {
    match p.strip_prefix('~') {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("{}{rest}", std::env::var("HOME").unwrap_or_default()),
        _ => p.to_string(),
    }
}

/// The output directory. With --kind and no --out, each kind set gets its own folder (~/.relic-scan/claude), so a
/// kind-only run never shares a state with a run over every kind.
fn out_dir(out: &str, kind: &[String]) -> String {
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
fn lock_out(outp: &std::path::Path) -> std::fs::File {
    std::fs::create_dir_all(outp).unwrap_or_else(|e| {
        eprintln!("relic-scan: cannot create {}: {e}", outp.display());
        std::process::exit(1);
    });
    let f = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(outp.join(".lock")).unwrap_or_else(|e| {
        eprintln!("relic-scan: cannot open {}/.lock: {e}", outp.display());
        std::process::exit(1);
    });
    if f.try_lock().is_err() {
        eprintln!("relic-scan: another relic-scan is writing {} (wait for it, or use another --out)", outp.display());
        std::process::exit(1);
    }
    f
}

fn run_cmd(a: RunArgs) {
    let RunArgs { out, roots, depth, threads, exclude, full, agent, jt_compat, only, kind } = a;
    let out = out_dir(&out, &kind);
    let t0 = std::time::Instant::now();
    let outp = std::path::Path::new(&out);
    let _lock = lock_out(outp);
    let roots = if roots.is_empty() { vec!["~".to_string(), "/opt/Code".to_string()] } else { roots };
    let agent = agent.unwrap_or_else(default_agent);
    let mut r = scan::run(&roots, depth, threads, &exclude, false);
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
    for f in &r.files {
        if !only.is_empty() && !only.iter().any(|o| f.path.contains(o.as_str())) {
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
                Some(parse::Ctx::default())
            }
            Some(s) => {
                grown += 1;
                Some(s.ctx.clone())
            }
        };
        if let Some(ctx) = start {
            jobs.push(parse::Job { path: f.path.clone(), size: f.size, verdict: f.verdict.clone(), ctx });
        }
    }
    let filtered = !only.is_empty() || !kind.is_empty();
    let gone = if !kind.is_empty() { 0 } else {
        st.keys().filter(|k| !seen.contains(*k) && (only.is_empty() || only.iter().any(|o| k.contains(o.as_str())))).count()
    };
    if !filtered {
        st.retain(|k, _| seen.contains(k));
    }
    if odd > 0 {
        eprintln!("run: skipped {} files whose path holds a tab or a newline", num(odd));
    }
    let bytes: u64 = jobs.iter().map(|j| j.size.saturating_sub(j.ctx.offset)).sum();
    eprintln!("run: scan {:.1}s · new {} · grown {} · rewritten {} · unchanged {} · gone {} · {:.2} GB to read",
              t_scan, num(new), num(grown), num(rewritten), num(unchanged), num(gone as u64), bytes as f64 / 1e9);
    let sizes: std::collections::HashMap<String, (u64, u64, u64)> =
        r.files.iter().map(|f| (f.path.clone(), (f.dev, f.ino, f.size))).collect();
    let done = if jobs.is_empty() { Default::default() } else { parse::run(jobs, outp, &agent, threads, jt_compat, false) };
    for (p, c) in done {
        let (dev, ino, size) = sizes.get(&p).copied().unwrap_or_default();
        let plen = c.offset.min(state::PREFIX);
        let ph = state::prefix_hash(&p, plen).unwrap_or(0);
        st.insert(p, state::FileState { dev, ino, size, ctx: c, prefix_len: plen, prefix_hash: ph });
    }
    state::save(outp, &st);
    meta::jsonl(outp, &r, &st);
    meta::build(outp, &agent, &st, threads);
    map::build(outp, &agent);
    eprintln!("run: done in {:.1}s · state {} files -> {}/_state.tsv", t0.elapsed().as_secs_f64(), num(st.len() as u64), out);
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Run(a) => run_cmd(a),
        Cmd::Up { run, port } => {
            let out = out_dir(&run.out, &run.kind);
            run_cmd(run);
            serve::serve(std::path::Path::new(&out), port);
        }
        Cmd::Serve { out, port, kind } => serve::serve(std::path::Path::new(&out_dir(&out, &kind)), port),
        Cmd::Map { out, agent, kind } => {
            let out = out_dir(&out, &kind);
            let _lock = lock_out(std::path::Path::new(&out));
            map::build(std::path::Path::new(&out), &agent.unwrap_or_else(default_agent));
        }
        Cmd::Meta { out, agent, threads, kind } => {
            let out = out_dir(&out, &kind);
            let outp = std::path::Path::new(&out);
            let _lock = lock_out(outp);
            let st = state::load(outp);
            if st.is_empty() {
                eprintln!("meta: no {out}/_state.tsv (run `relic-scan run --out {out}` first)");
                std::process::exit(1);
            }
            meta::build(outp, &agent.unwrap_or_else(default_agent), &st, threads);
        }
        Cmd::Parse { index, out, agent, threads, jt_compat, only, keep_parts, all } => {
            let agent = agent.unwrap_or_else(default_agent);
            let mut jobs = parse::read_index(&index);
            if !only.is_empty() {
                jobs.retain(|j| only.iter().any(|o| j.path.contains(o.as_str())));
            }
            if !all {
                let before = jobs.len();
                jobs.retain(|j| !matches!(j.verdict.as_str(), "not-transcript" | "empty" | "unreadable"));
                if before != jobs.len() {
                    eprintln!("parse: skipping {} files the content sniff called not-transcript (--all to include)", before - jobs.len());
                }
            }
            eprintln!("parse: {} files from {index} · agent {agent}", jobs.len());
            let _ = parse::run(jobs, std::path::Path::new(&out), &agent, threads, jt_compat, keep_parts);
        }
        Cmd::Scan { roots, depth, threads, json, index, exclude, full } => {
            let roots = if roots.is_empty() { vec!["~".to_string(), "/opt/Code".to_string()] } else { roots };
            let r = scan::run(&roots, depth, threads, &exclude, full);
            let (files, bytes, dups): (u64, u64, u64) =
                r.homes.iter().fold((0, 0, 0), |a, h| (a.0 + h.files, a.1 + h.bytes, a.2 + h.dup_files));
            if let Some(p) = &index {
                let mut f = std::io::BufWriter::new(std::fs::File::create(p).expect("create index"));
                writeln!(f, "path\thome\tkind\torigin\ttier\tagent\tverdict\tsize\tmtime_ms\tdev\tino").unwrap();
                for e in &r.files {
                    writeln!(f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", e.path, e.home, e.kind, scan::origin(&e.home), scan::tier(&e.path),
                             e.agent, e.verdict, e.size, e.mtime, e.dev, e.ino).unwrap();
                }
            }
            if json {
                let v = serde_json::json!({
                    "roots": roots, "homes": r.homes, "files": files, "bytes": bytes, "dup_files": dups,
                    "secs_homes": r.secs_homes, "secs_files": r.secs_files,
                });
                println!("{}", serde_json::to_string_pretty(&v).unwrap());
                return;
            }
            let mut homes = r.homes.clone();
            homes.sort_by(|a, b| b.bytes.cmp(&a.bytes));
            let w = homes.iter().map(|h| tilde(&h.path).chars().count()).max().unwrap_or(4).clamp(4, 90);
            println!("{:<w$}  {:<7} {:<7} {:<5} {:>8} {:>8} {:>8} {:>6} {:>6} {:>8} {:>6}  {}", "HOME", "KIND", "ORIGIN", "BY", "FILES",
                     "CHATS", "EMPTY", "NOT", "DUP", "GB", "", "NEWEST (BKK)");
            for h in homes.iter().filter(|h| h.files > 0 || h.dup_files > 0) {
                let mut p = tilde(&h.path);
                if p.chars().count() > w {
                    p = format!("…{}", p.chars().skip(p.chars().count() - w + 1).collect::<String>());
                }
                println!("{:<w$}  {:<7} {:<7} {:<5} {:>8} {:>8} {:>8} {:>6} {:>6} {:>8} {:>6}  {}", p, h.kind, h.origin, h.found_by, num(h.files),
                         num(h.transcripts), num(h.empty_sessions), num(h.not_transcripts), num(h.dup_files), gb(h.bytes), "",
                         h.newest.clone().unwrap_or_default());
            }
            let mut by: std::collections::BTreeMap<&str, (u64, u64)> = Default::default();
            for h in &homes {
                let e = by.entry(h.origin.as_str()).or_default();
                e.0 += h.files;
                e.1 += h.bytes;
            }
            let (ch, em, nt) = homes.iter().fold((0, 0, 0), |a, h| (a.0 + h.transcripts, a.1 + h.empty_sessions, a.2 + h.not_transcripts));
            eprintln!("{} scan: {} transcripts · {} {} · {} not transcripts (prompt logs, runner journals, omx logs, data)",
                      if full { "full (content)" } else { "light (names + paths)" }, num(ch), num(em),
                      if full { "empty sessions" } else { "unknown shape" }, num(nt));
            eprintln!("by origin: {}", by.iter().map(|(k, (f, b))| format!("{k} {} files {} GB", num(*f), gb(*b))).collect::<Vec<_>>().join(" · "));
            let in_vault = r.loose.iter().filter(|f| f.kind == "vault").count();
            let vaults: std::collections::BTreeSet<&str> = r.loose.iter().filter(|f| f.kind == "vault").map(|f| f.home.as_str()).collect();
            let mut lv: std::collections::BTreeMap<&str, u64> = Default::default();
            for f in &r.loose {
                *lv.entry(f.verdict.as_str()).or_default() += 1;
            }
            eprintln!("loose .jsonl (outside every home, not parsed): {} · in ψ vaults {} ({} vaults) · elsewhere {} · {}",
                      num(r.loose.len() as u64), num(in_vault as u64), vaults.len(), num((r.loose.len() - in_vault) as u64),
                      lv.iter().map(|(k, v)| format!("{k} {}", num(*v))).collect::<Vec<_>>().join(" · "));
            let empty = homes.iter().filter(|h| h.files == 0 && h.dup_files == 0).count();
            eprintln!("{} homes ({} with no .jsonl) · {} files · {} GB · {} duplicate inodes folded · homes found in {:.2}s · files indexed in {:.2}s{}",
                      homes.len(), empty, num(files), gb(bytes), num(dups), r.secs_homes, r.secs_files,
                      index.map(|p| format!(" · index -> {p}")).unwrap_or_default());
        }
    }
}
