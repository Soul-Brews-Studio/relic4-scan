//! relic4-scan — one binary that finds every AI-agent transcript on this machine and (next phases) parses it
//! into one file per Bangkok month. See RESEARCH.md for the design and its sources.
//!
//!   relic4-scan scan [--root DIR]... [--depth 8] [--json] [--index index.tsv]
//!       phase 1: find every agent home (.claude, .claude_bak, .claude2, .codex …) and index its .jsonl files

mod scan;

use std::io::Write;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Find and index every AI-agent transcript on this machine")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
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

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Scan { roots, depth, threads, json, index, exclude } => {
            let roots = if roots.is_empty() { vec!["~".to_string(), "/opt/Code".to_string()] } else { roots };
            let r = scan::run(&roots, depth, threads, &exclude);
            let (files, bytes, dups): (u64, u64, u64) =
                r.homes.iter().fold((0, 0, 0), |a, h| (a.0 + h.files, a.1 + h.bytes, a.2 + h.dup_files));
            if let Some(p) = &index {
                let mut f = std::io::BufWriter::new(std::fs::File::create(p).expect("create index"));
                writeln!(f, "path\thome\tkind\torigin\ttier\tsize\tmtime_ms\tdev\tino").unwrap();
                for e in &r.files {
                    writeln!(f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", e.path, e.home, e.kind, scan::origin(&e.home), scan::tier(&e.path), e.size, e.mtime, e.dev, e.ino).unwrap();
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
            println!("{:<w$}  {:<7} {:<7} {:>8} {:>8} {:>7} {:>7} {:>7} {:>6}  {}", "HOME", "KIND", "ORIGIN", "FILES", "GB", "MAIN", "SUB", "WF",
                     "DUP", "NEWEST (BKK)");
            for h in homes.iter().filter(|h| h.files > 0 || h.dup_files > 0) {
                let mut p = tilde(&h.path);
                if p.chars().count() > w {
                    p = format!("…{}", p.chars().skip(p.chars().count() - w + 1).collect::<String>());
                }
                println!("{:<w$}  {:<7} {:<7} {:>8} {:>8} {:>7} {:>7} {:>7} {:>6}  {}", p, h.kind, h.origin, num(h.files), gb(h.bytes), num(h.main),
                         num(h.subagent), num(h.workflow_agent), num(h.dup_files), h.newest.clone().unwrap_or_default());
            }
            let mut by: std::collections::BTreeMap<&str, (u64, u64)> = Default::default();
            for h in &homes {
                let e = by.entry(h.origin.as_str()).or_default();
                e.0 += h.files;
                e.1 += h.bytes;
            }
            eprintln!("by origin: {}", by.iter().map(|(k, (f, b))| format!("{k} {} files {} GB", num(*f), gb(*b))).collect::<Vec<_>>().join(" · "));
            let empty = homes.iter().filter(|h| h.files == 0 && h.dup_files == 0).count();
            eprintln!("{} homes ({} with no .jsonl) · {} files · {} GB · {} duplicate inodes folded · homes found in {:.2}s · files indexed in {:.2}s{}",
                      homes.len(), empty, num(files), gb(bytes), num(dups), r.secs_homes, r.secs_files,
                      index.map(|p| format!(" · index -> {p}")).unwrap_or_default());
        }
    }
}
