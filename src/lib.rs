//! relic-scan — one binary that finds every AI-agent transcript on this machine and (next phases) parses it
//! into one file per Bangkok month. See docs/RESEARCH.md for the design and its sources.
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

pub mod config;
pub mod engine;
pub mod ledger;
pub mod links;
pub mod export;
pub mod export_s3;
pub mod manifest;
pub mod map;
pub mod meta;
pub mod ops;
pub mod parse;
pub mod range;
pub mod run;
pub mod s3job;
pub mod scanlog;
pub mod scan;
pub mod serve;
pub mod sniff;
pub mod state;
pub mod tee;
pub mod tui;
pub mod ui;
pub mod vault;



pub use run::{run_cmd, run_inner, run_with, Report, CANCEL, PHASE};

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
        if i > 0 && (s.len() - i).is_multiple_of(3) {
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
