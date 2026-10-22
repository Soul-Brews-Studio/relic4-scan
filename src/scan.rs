//! Phase 1, `relic4-scan scan`: find every agent home under the roots, then index every .jsonl inside them.
//!
//! A home is a directory whose NAME says it holds agent transcripts, whatever the copy was called:
//!   .claude .claude2 .claude_bak .claudebak .claude-old …   (any `.claude*`)
//!   claude_bak claude-backup claude-rescue …                (no dot: `claude` + bak/backup/rescue/mirror/old/copy)
//!   .codex* .omp* .omx* .thclaws* .gemini* .pi
//! A home is also found by SHAPE, for copies whose folder name says nothing (relic mirrors, rescues,
//! ~/Documents/Codex): a transcript outside every named home makes its root a home —
//!   <root>/<project-dir>/<uuid>.jsonl            -> <root> (a claude projects folder)
//!   <root>/[YYYY/MM/DD/]rollout-*.jsonl          -> <root> (a codex sessions folder)
//! The walk does not descend into a home it found (a home's own subfolders are not homes), and skips heavy
//! trees that never hold one (node_modules, .git, Library, caches, toolchains).
//!
//! Then every *.jsonl under each home is listed with size, mtime and (dev, inode). One inode reachable twice
//! (a symlinked or hard-linked copy) counts once: the first home in sorted order keeps it, like jt does.

use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, UNIX_EPOCH};

use ignore::{WalkBuilder, WalkState};
use serde::Serialize;

/// Directory names never worth entering while looking for homes (or while listing a home's files).
const SKIP: &[&str] = &[
    "node_modules", ".git", "target", "dist", "build", ".venv", "venv", "__pycache__", ".next", "vendor",
    "Library", ".Trash", ".cache", ".npm", ".bun", ".cargo", ".rustup", ".pyenv", ".gradle", ".m2", "go",
    ".docker", ".colima", ".orbstack", "Pictures", "Movies", "Music", ".vscode", ".cursor",
];
/// Inside a home: folders jt also skips (plugin caches, shell snapshots, file history), never transcripts.
const SKIP_IN_HOME: &[&str] = &["plugins", "cache", "shell-snapshots", "statsig", "file-history"];

/// The agent a directory name stands for, if it is an agent home.
pub fn agent_kind(name: &str) -> Option<&'static str> {
    let n = name.to_lowercase();
    if n.starts_with(".omx-runs") {
        return Some("omx-ops"); // omx run logs, not conversation (relic v1 sources.ts:122)
    }
    let dotted = [(".claude", "claude"), (".codex", "codex"), (".omp", "omp"), (".omx", "omx"), (".thclaws", "thclaws"),
                  (".gemini", "gemini")];
    for (p, k) in dotted {
        if n.starts_with(p) {
            return Some(k);
        }
    }
    if n == ".pi" {
        return Some("pi");
    }
    // undotted copies: claude_bak, claude-backup, claude-rescue-2026 … (a bare "claude" dir is usually a repo)
    if n.starts_with("claude") && ["bak", "backup", "rescue", "mirror", "old", "copy"].iter().any(|w| n.contains(w)) {
        return Some("claude");
    }
    None
}

/// Where a home's files came from: this account's own agent, or a copy (mirrors go stale, rescues are frozen).
pub fn origin(path: &str) -> &'static str {
    let p = path.to_lowercase();
    if p.contains("mirror") {
        "mirror"
    } else if p.contains("rescue") || p.contains("_bak") || p.contains("backup") || p.contains("bak/") || p.ends_with("bak") {
        "rescue"
    } else if p.contains("/_archive/") || p.contains("/archive/") {
        "archive"
    } else if p.contains("/downloads/") || p.contains("/documents/") && !p.ends_with("/documents/codex") {
        "copy"
    } else {
        "own"
    }
}

#[derive(Serialize, Clone)]
pub struct Home {
    pub path: String,
    pub kind: String,
    pub origin: String,
    pub files: u64,
    pub bytes: u64,
    pub dup_files: u64,
    pub newest: Option<String>,
    pub main: u64,
    pub subagent: u64,
    pub workflow_agent: u64,
    pub transcripts: u64,
    pub empty_sessions: u64,
    pub not_transcripts: u64,
    pub found_by: String, // name | shape — and every kept home is confirmed by content
}

#[derive(Serialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub home: String,
    pub kind: String,
    pub agent: String,   // by content (sniff.rs), "-" when no signature
    pub verdict: String, // transcript | transcript-empty | not-transcript | empty | unreadable
    pub size: u64,
    pub mtime: i64,
    pub dev: u64,
    pub ino: u64,
}

/// main / subagent / workflow-agent / other, from the file path (same rules as relic4's sql.tier_of).
pub fn tier(path: &str) -> &'static str {
    if path.contains("/subagents/workflows/") && path.ends_with(".jsonl") && file_name(path).starts_with("agent-") {
        "workflow-agent"
    } else if path.contains("/subagents/") && file_name(path).starts_with("agent-") {
        "subagent"
    } else if is_uuid_jsonl(file_name(path)) || (file_name(path).starts_with("rollout-") && path.ends_with(".jsonl")) {
        "main"
    } else {
        "other"
    }
}

fn file_name(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

fn is_uuid_jsonl(n: &str) -> bool {
    let Some(stem) = n.strip_suffix(".jsonl") else { return false };
    let b = stem.as_bytes();
    b.len() == 36 && b.iter().enumerate().all(|(i, c)| match i {
        8 | 13 | 18 | 23 => *c == b'-',
        _ => c.is_ascii_hexdigit(),
    })
}

fn expand(p: &str) -> PathBuf {
    match p.strip_prefix("~/") {
        Some(rest) => PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(rest),
        None if p == "~" => PathBuf::from(std::env::var("HOME").unwrap_or_default()),
        None => PathBuf::from(p),
    }
}

fn walker(root: &Path, depth: Option<usize>, threads: usize) -> WalkBuilder {
    let mut w = WalkBuilder::new(root);
    w.hidden(false).ignore(false).git_ignore(false).git_global(false).git_exclude(false).parents(false)
        .follow_links(false).max_depth(depth).threads(threads);
    w
}

/// Every home under the roots, sorted by path.
/// The root a stray transcript belongs to, by its shape (see the module doc), or None.
fn shape_root(p: &Path) -> Option<(PathBuf, &'static str)> {
    let name = p.file_name()?.to_str()?;
    if name.starts_with("rollout-") {
        // climb over YYYY / MM / DD (or YYYY-MM-DD) folders to the sessions folder
        let mut d = p.parent()?;
        while d.file_name().and_then(|n| n.to_str()).is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit() || c == b'-')) {
            d = d.parent()?;
        }
        return Some((d.to_path_buf(), "codex"));
    }
    if is_uuid_jsonl(name) {
        return Some((p.parent()?.parent()?.to_path_buf(), "claude"));
    }
    None
}

/// Every home under the roots, sorted by path: named homes, then shape homes not inside a named one.
pub fn find_homes(roots: &[String], depth: usize, threads: usize, exclude: &[String]) -> Vec<(PathBuf, &'static str)> {
    let found: Arc<Mutex<Vec<(PathBuf, &'static str)>>> = Arc::default();
    let shaped: Arc<Mutex<Vec<(PathBuf, &'static str)>>> = Arc::default();
    for r in roots {
        let root = expand(r);
        if !root.is_dir() {
            continue;
        }
        let (f, sh) = (found.clone(), shaped.clone());
        let ex: Vec<String> = exclude.to_vec();
        walker(&root, Some(depth), threads)
            .filter_entry(move |e| {
                let name = e.file_name().to_str().unwrap_or("");
                // like jt: below the root's own children, a hidden folder is entered only if it is an agent home
                // (~/.relic-mirrors is walked, a repo's .tmp test fixtures are not)
                let hidden_deep = e.depth() > 1 && name.starts_with('.') && agent_kind(name).is_none()
                    && e.file_type().is_some_and(|t| t.is_dir());
                !SKIP.contains(&name) && !hidden_deep && !ex.iter().any(|x| e.path().to_string_lossy().contains(x.as_str()))
            })
            .build_parallel()
            .run(|| {
                let (f, sh) = (f.clone(), sh.clone());
                Box::new(move |res| {
                    let Ok(e) = res else { return WalkState::Continue };
                    if e.depth() == 0 {
                        return WalkState::Continue;
                    }
                    if e.file_type().is_some_and(|t| t.is_dir()) {
                        if let Some(kind) = e.file_name().to_str().and_then(agent_kind) {
                            f.lock().unwrap().push((e.path().to_path_buf(), kind));
                            return WalkState::Skip; // a home's own subfolders are not homes
                        }
                    } else if e.path().extension().is_some_and(|x| x == "jsonl") {
                        if let Some(h) = shape_root(e.path()) {
                            sh.lock().unwrap().push(h);
                        }
                    }
                    WalkState::Continue
                })
            });
    }
    let mut v = std::mem::take(&mut *found.lock().unwrap());
    let mut extra = std::mem::take(&mut *shaped.lock().unwrap());
    extra.sort();
    extra.dedup_by(|a, b| a.0 == b.0);
    // keep the outermost shape root when one sits inside another
    let mut outer: Vec<(PathBuf, &'static str)> = Vec::new();
    for h in extra {
        if !outer.iter().any(|o| h.0.starts_with(&o.0)) {
            outer.push(h);
        }
    }
    v.extend(outer);
    v.sort();
    v.dedup_by(|a, b| a.0 == b.0);
    v
}

/// Every *.jsonl under one home (no depth limit).
fn list_home(home: &Path, kind: &'static str, threads: usize, by_shape: bool) -> Vec<FileEntry> {
    let out: Arc<Mutex<Vec<FileEntry>>> = Arc::default();
    let label = home.to_string_lossy().to_string();
    let o = out.clone();
    walker(home, None, threads)
        .filter_entry(move |e| {
            !e.file_name().to_str().is_some_and(|n| {
                SKIP_IN_HOME.contains(&n) || n == "node_modules" || n == ".git"
                    // a shape home lists named homes inside it on their own (they are homes too); a named home
                    // keeps what is nested in it (~/.claude/jobs/*/tmp/.thclaws was never seen by the home walk)
                    || (by_shape && e.depth() > 0 && e.file_type().is_some_and(|t| t.is_dir()) && agent_kind(n).is_some())
            })
        })
        .build_parallel()
        .run(|| {
            let o = o.clone();
            let label = label.clone();
            Box::new(move |res| {
                let Ok(e) = res else { return WalkState::Continue };
                if !e.path().extension().is_some_and(|x| x == "jsonl") {
                    return WalkState::Continue;
                }
                // follow a symlinked file (dead links drop here), like statSync in jt
                let Ok(md) = std::fs::metadata(e.path()) else { return WalkState::Continue };
                if !md.is_file() {
                    return WalkState::Continue;
                }
                let mtime = md.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64).unwrap_or(0);
                o.lock().unwrap().push(FileEntry {
                    path: e.path().to_string_lossy().to_string(),
                    home: label.clone(),
                    kind: kind.to_string(),
                    agent: String::new(),
                    verdict: String::new(),
                    size: md.len(),
                    mtime,
                    dev: md.dev(),
                    ino: md.ino(),
                });
                WalkState::Continue
            })
        });
    let mut v = std::mem::take(&mut *out.lock().unwrap());
    v.sort_by(|a, b| a.path.cmp(&b.path));
    v
}

pub struct ScanResult {
    pub homes: Vec<Home>,
    pub files: Vec<FileEntry>,
    pub secs_homes: f64,
    pub secs_files: f64,
}

pub fn run(roots: &[String], depth: usize, threads: usize, exclude: &[String], full: bool) -> ScanResult {
    let t0 = Instant::now();
    let homes = find_homes(roots, depth, threads, exclude);
    let secs_homes = t0.elapsed().as_secs_f64();
    let t1 = Instant::now();
    let mut seen: HashSet<(u64, u64)> = HashSet::new();
    let mut files = Vec::new();
    let mut out = Vec::new();
    for (path, kind) in &homes {
        let mut h = Home {
            path: path.to_string_lossy().to_string(), kind: kind.to_string(), origin: origin(&path.to_string_lossy()).to_string(), files: 0, bytes: 0, dup_files: 0,
            newest: None, main: 0, subagent: 0, workflow_agent: 0, transcripts: 0, empty_sessions: 0, not_transcripts: 0,
            found_by: if path.file_name().and_then(|n| n.to_str()).and_then(agent_kind).is_some() { "name".into() } else { "shape".into() },
        };
        let mut newest = 0i64;
        let by_shape = path.file_name().and_then(|n| n.to_str()).and_then(agent_kind).is_none();
        let mut listed = list_home(path, kind, threads, by_shape);
        sniff_all(&mut listed, threads, full);
        for f in listed {
            if !seen.insert((f.dev, f.ino)) {
                h.dup_files += 1; // the same inode under an earlier home
                continue;
            }
            h.files += 1;
            h.bytes += f.size;
            match f.verdict.as_str() {
                "transcript" | "transcript-shaped" | "transcript-unverified" => h.transcripts += 1,
                "transcript-empty" | "unknown" => h.empty_sessions += 1,
                _ => h.not_transcripts += 1,
            }
            newest = newest.max(f.mtime);
            match tier(&f.path) {
                "main" => h.main += 1,
                "subagent" => h.subagent += 1,
                "workflow-agent" => h.workflow_agent += 1,
                _ => {}
            }
            files.push(f);
        }
        if newest > 0 {
            h.newest = chrono::DateTime::from_timestamp_millis(newest + 7 * 3600 * 1000)
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string());
        }
        // a home found only by its shape must hold at least one real transcript
        if h.found_by == "shape" && h.transcripts == 0 {
            files.retain(|f: &FileEntry| f.home != h.path);
            continue;
        }
        out.push(h);
    }
    ScanResult { homes: out, files, secs_homes, secs_files: t1.elapsed().as_secs_f64() }
}

/// Content verdict for every file, in parallel (64 KB read + parse per file).
fn sniff_all(files: &mut [FileEntry], threads: usize, full: bool) {
    let n = if threads == 0 { std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8) } else { threads };
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: Vec<Mutex<Option<(String, String)>>> = (0..files.len()).map(|_| Mutex::new(None)).collect();
    std::thread::scope(|sc| {
        for _ in 0..n {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(f) = files.get(i) else { break };
                let s = if full { crate::sniff::sniff(&f.path) } else { crate::sniff::by_path(&f.path) };
                *results[i].lock().unwrap() = Some((s.agent.to_string(), s.verdict.to_string()));
            });
        }
    });
    for (f, r) in files.iter_mut().zip(results) {
        if let Some((a, v)) = r.into_inner().unwrap() {
            f.agent = a;
            f.verdict = v;
        }
    }
}
