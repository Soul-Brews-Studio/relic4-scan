//! The `relic-scan` binary: the command line over the library (src/lib.rs has the commands' work).

use std::io::Write;

use clap::{Parser, Subcommand};
use relic_scan::*;

#[derive(Parser)]
#[command(version, about = "Find and index every AI-agent transcript on this machine")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
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
    /// Closed months and their seals (read-only files with the time, rows and key hash they were sealed at); --verify checks each again
    Seals {
        #[arg(long, default_value = DEFAULT_OUT)]
        out: String,
        /// the --kind the run used: picks ~/.relic-scan/<kind> when --out is not given
        #[arg(long)]
        kind: Vec<String>,
        /// read every sealed month again and compare it with its seal
        #[arg(long)]
        verify: bool,
    },
    /// The scan log: every index run and check (dry run) with what it found; --diff N lists one entry's files
    Log {
        #[arg(long, default_value = DEFAULT_OUT)]
        out: String,
        /// the --kind the run used: picks ~/.relic-scan/<kind> when --out is not given
        #[arg(long)]
        kind: Vec<String>,
        /// how many entries, newest first
        #[arg(short, long, default_value_t = 20)]
        n: usize,
        /// list the files of entry N (the number in the first column)
        #[arg(long)]
        diff: Option<u64>,
    },
    /// Browse the sessions, repos and every .jsonl in the terminal (reads OUT/meta/*.parquet)
    Tui {
        #[arg(long, default_value = DEFAULT_OUT)]
        out: String,
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

fn main() {
    let cli = Cli::parse();
    let Some(cmd) = cli.cmd else { return banner() };
    match cmd {
        Cmd::Tui { out, kind } => tui::tui(std::path::Path::new(&out_dir(&out, &kind))),
        Cmd::Run(a) => run_cmd(a),
        Cmd::Seals { out, kind, verify } => seals_cmd(std::path::Path::new(&out_dir(&out, &kind)), verify),
        Cmd::Log { out, kind, n, diff } => log_cmd(std::path::Path::new(&out_dir(&out, &kind)), n, diff),
        Cmd::Up { run, port } => {
            let out = run.out_dir().unwrap_or_else(|e| {
                eprintln!("relic-scan: {e}");
                std::process::exit(1);
            });
            let kind = run.kind.clone();
            run_cmd(run);
            serve::serve(std::path::Path::new(&out), port, kind);
        }
        Cmd::Serve { out, port, kind } => serve::serve(std::path::Path::new(&out_dir(&out, &kind)), port, kind),
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
            let _ = meta::build(outp, &agent.unwrap_or_else(default_agent), &st, threads);
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
            let _ = parse::run(jobs, std::path::Path::new(&out), &agent, threads, jt_compat, keep_parts, None);
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

fn log_cmd(out: &std::path::Path, n: usize, diff: Option<u64>) {
    let all = scanlog::read(out);
    if all.is_empty() {
        eprintln!("log: no {} yet (every `run` and `run --dry-run` adds a line)", scanlog::path(out).display());
        std::process::exit(1);
    }
    let u = |e: &serde_json::Value, k: &str| e[k].as_u64().unwrap_or(0);
    let when = |e: &serde_json::Value| chrono::DateTime::from_timestamp(u(e, "at") as i64, 0)
        .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default();
    if let Some(i) = diff {
        let Some(e) = all.iter().find(|e| e["i"].as_u64() == Some(i)) else {
            eprintln!("log: no entry {i}");
            std::process::exit(1);
        };
        eprintln!("{} {} {} · {} files read · {} GB · ledger: {} copies, {} resumed, {} past changed", ui::bold(&format!("#{i}")), when(e),
                  e["job"].as_str().unwrap_or(""), num(u(e, "files")), gb(u(e, "bytes")), num(u(e, "copies")), num(u(e, "resumed")),
                  num(u(e, "past_changed")));
        for c in e["diff"].as_array().into_iter().flatten() {
            let (st, p, size, from) = (c[0].as_str().unwrap_or(""), c[1].as_str().unwrap_or(""), c[2].as_u64().unwrap_or(0), c[3].as_u64().unwrap_or(0));
            let what = match st {
                "grown" | "resumed" => format!("+{}", human(size.saturating_sub(from))),
                "gone" => format!("was {}", human(from)),
                "copy" => "not read".to_string(),
                _ => human(size),
            };
            println!("{:<12} {:>12}  {}", st, what, tilde(p));
        }
        let (shown, total) = (e["diff"].as_array().map_or(0, |a| a.len()), u(e, "diff_total") as usize);
        if total > shown {
            eprintln!("… {} more (the log keeps the {} biggest reads)", num((total - shown) as u64), num(shown as u64));
        }
        return;
    }
    // copies, resumed and past changed: what the ledger answered for the new and rewritten paths (ledger.rs)
    println!("{:>5}  {:<16}  {:<6} {:>7} {:>7} {:>9} {:>6} {:>10} {:>7} {:>7} {:>6} {:>9} {:>7}  {}", "#", "when", "job", "new", "grown",
             "rewritten", "gone", "unchanged", "copies", "resumed", "past", "read", "secs", "range");
    for e in all.iter().rev().take(n) {
        println!("{:>5}  {:<16}  {:<6} {:>7} {:>7} {:>9} {:>6} {:>10} {:>7} {:>7} {:>6} {:>9} {:>7.1}  {}", u(e, "i"), when(e),
                 e["job"].as_str().unwrap_or(""), num(u(e, "new")), num(u(e, "grown")), num(u(e, "rewritten")), num(u(e, "gone")),
                 num(u(e, "unchanged")), num(u(e, "copies")), num(u(e, "resumed")), num(u(e, "past_changed")), human(u(e, "bytes")),
                 e["secs"].as_f64().unwrap_or(0.0), e["range"].as_str().unwrap_or(""));
    }
    eprintln!("{}", ui::dim("relic-scan log --diff N lists the files of entry N"));
}

fn human(b: u64) -> String {
    if b >= 1_000_000_000 { format!("{:.2} GB", b as f64 / 1e9) } else if b >= 1_000_000 { format!("{:.1} MB", b as f64 / 1e6) } else { format!("{:.0} KB", b as f64 / 1e3) }
}

/// `relic-scan seals`: the closed months with their seals; with --verify each is read again and checked.
fn seals_cmd(out: &std::path::Path, check: bool) {
    let months = relic_scan::manifest::build(out);
    let checked: std::collections::HashMap<String, bool> = if check { relic_scan::manifest::verify(out).into_iter().map(|v| (v.0, v.3)).collect() } else { Default::default() };
    println!("{:<9} {:>10}  {:<20} {:>8}  {}", "month", "rows", "sealed at", "resealed", "status");
    for e in &months {
        let status = match (e.sealed_at.is_empty(), checked.get(&e.month)) {
            (true, _) => "open".to_string(),
            (false, Some(true)) => "sealed, matches".to_string(),
            (false, Some(false)) => "CHANGED since the seal".to_string(),
            (false, None) => "sealed".to_string(),
        };
        println!("{:<9} {:>10}  {:<20} {:>8}  {}", e.month, e.rows, if e.sealed_at.is_empty() { "-" } else { &e.sealed_at }, e.resealed, status);
    }
    if checked.values().any(|ok| !ok) {
        std::process::exit(1);
    }
}
