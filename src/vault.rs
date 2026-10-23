//! ψ vaults: every oracle's `ψ` folder the scan walked past, real or a link, with its notes -> OUT/meta/vaults.json
//! (one line per vault) and OUT/meta/vaults/<id>.json (that vault's notes, read by the page when the vault is opened).
//!
//! A vault is listed once, by its real path; the `ψ` links that reach it are its `links` (a code repo whose memory lives
//! in another oracle's vault). A vault in a git worktree (its repo's `.git` is a file) is a `worktree` copy of the main
//! checkout, named as its `origin`; the page hides those copies unless asked.
//!
//! A note is a markdown file under the vault: its path inside it, its section (the first two folders: "memory/learnings",
//! "inbox"), its title (the first `# ` heading in the first 4 KB, else the file name), size and mtime. A folder inside the
//! vault that is a git repository of its own (a cloned repo kept for study) is not the vault's writing and is skipped,
//! like .git and node_modules. A title is read again only when the note changed since the last run.
//!
//! Plain JSON on purpose: the page reads it, and so can jq, DuckDB or any script, with nothing to install.

use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::Serialize;

/// One note: (path inside the vault, section, title, bytes, mtime ms).
pub type Note = (String, String, String, u64, i64);

#[derive(Serialize, Default)]
pub struct Vault {
    pub id: usize,
    /// the folder holding the vault (an oracle or repo name)
    pub name: String,
    /// the vault as the walk met it
    pub path: String,
    /// "own" (a repo's own vault), "worktree" (a git worktree's copy of one) or "plain" (not in a git repo)
    pub kind: String,
    /// a worktree copy: the main checkout it was made from
    pub origin: String,
    /// `ψ` links elsewhere that reach this vault
    pub links: Vec<String>,
    pub files: u64,
    pub bytes: u64,
    pub notes: u64,
    pub newest_ms: i64,
    /// notes per section
    pub sections: BTreeMap<String, u64>,
    /// nested git repositories skipped inside it
    pub skipped_repos: u64,
    #[serde(skip)]
    pub items: Vec<Note>,
}

pub struct Summary {
    pub vaults: usize,
    pub worktrees: usize,
    pub notes: u64,
    pub bytes: u64,
}

const SKIP: [&str; 4] = [".git", "node_modules", ".obsidian", ".trash"];

/// The section of a note: its first two folders inside the vault ("memory/learnings"), one when there is only one,
/// "(top)" for a note in the vault's own folder.
pub fn section(rel: &str) -> String {
    let dirs: Vec<&str> = rel.split('/').collect();
    match dirs.len() {
        0 | 1 => "(top)".into(),
        2 => dirs[0].into(),
        _ => format!("{}/{}", dirs[0], dirs[1]),
    }
}

/// The first `# ` heading in the first 4 KB (after any front matter), else the file name without `.md`.
pub fn title(path: &Path) -> String {
    let mut buf = vec![0u8; 4096];
    let n = std::fs::File::open(path).and_then(|mut f| f.read(&mut buf)).unwrap_or(0);
    let text = String::from_utf8_lossy(&buf[..n]);
    for line in text.lines() {
        if let Some(h) = line.strip_prefix("# ") {
            let h = h.trim();
            if !h.is_empty() {
                return h.chars().take(160).collect();
            }
        }
    }
    stem(path)
}
fn stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
}

/// own / worktree (with the main checkout) / plain, from the `.git` beside the vault.
fn kind_of(vault: &Path) -> (&'static str, String) {
    let Some(repo) = vault.parent() else { return ("plain", String::new()) };
    let git = repo.join(".git");
    match std::fs::symlink_metadata(&git) {
        Ok(m) if m.is_dir() => ("own", String::new()),
        Ok(m) if m.is_file() => {
            // "gitdir: /path/main/.git/worktrees/<name>" -> /path/main
            let t = std::fs::read_to_string(&git).unwrap_or_default();
            let main = t.trim().strip_prefix("gitdir:").map(str::trim).and_then(|g| g.split("/.git/").next()).unwrap_or("").to_string();
            ("worktree", main)
        }
        _ => ("plain", String::new()),
    }
}

/// The titles of the last run, by "<vault path>\t<note>": (size, mtime, title), so an unchanged note is not opened again.
type Titles = HashMap<String, (u64, i64, String)>;
fn last_titles(dir: &Path) -> Titles {
    let mut t = Titles::new();
    let Ok(rd) = std::fs::read_dir(dir.join("vaults")) else { return t };
    for e in rd.flatten() {
        let Ok(j) = std::fs::read_to_string(e.path()).map(|s| serde_json::from_str::<serde_json::Value>(&s)) else { continue };
        let Ok(j) = j else { continue };
        let vp = j["path"].as_str().unwrap_or_default();
        for n in j["items"].as_array().into_iter().flatten() {
            if let (Some(rel), Some(title), Some(size), Some(ms)) = (n[0].as_str(), n[2].as_str(), n[3].as_u64(), n[4].as_i64()) {
                t.insert(format!("{vp}\t{rel}"), (size, ms, title.to_string()));
            }
        }
    }
    t
}

fn read_vault(shown: &Path, real: &Path, titles: &Titles) -> Vault {
    let (kind, origin) = kind_of(shown);
    let mut v = Vault {
        name: shown.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        path: shown.display().to_string(),
        kind: kind.into(),
        origin,
        ..Default::default()
    };
    let skipped = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let sk = skipped.clone();
    let walk = ignore::WalkBuilder::new(real).hidden(false).ignore(false).git_ignore(false).git_global(false).git_exclude(false)
        .parents(false).follow_links(false)
        .filter_entry(move |e| {
            let n = e.file_name().to_str().unwrap_or("");
            if SKIP.contains(&n) {
                return false;
            }
            // a cloned repository kept inside the vault: its README and docs are not the vault's notes
            let nested = e.depth() > 0 && e.file_type().is_some_and(|t| t.is_dir()) && e.path().join(".git").exists();
            if nested {
                sk.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            !nested
        })
        .build();
    for e in walk.flatten() {
        if !e.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let Ok(md) = e.metadata() else { continue };
        v.files += 1;
        v.bytes += md.len();
        if !e.path().extension().is_some_and(|x| x == "md" || x == "markdown") {
            continue;
        }
        let rel = e.path().strip_prefix(real).map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let mtime = md.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64).unwrap_or(0);
        let sec = section(&rel);
        // a worktree copy shows file names only; a note unchanged since the last run keeps its title
        let t = match titles.get(&format!("{}\t{rel}", v.path)) {
            Some((s, m, t)) if *s == md.len() && *m == mtime => t.clone(),
            _ if v.kind == "worktree" => stem(e.path()),
            _ => title(e.path()),
        };
        *v.sections.entry(sec.clone()).or_default() += 1;
        v.notes += 1;
        v.newest_ms = v.newest_ms.max(mtime);
        v.items.push((rel, sec, t, md.len(), mtime));
    }
    v.skipped_repos = skipped.load(std::sync::atomic::Ordering::Relaxed);
    // newest first: what the page shows when a vault is opened
    v.items.sort_by(|a, b| b.4.cmp(&a.4).then_with(|| a.0.cmp(&b.0)));
    v
}

/// Read every vault the walk found (each real folder once, in parallel) and write OUT/meta/vaults.json and
/// OUT/meta/vaults/<id>.json.
pub fn index(found: Vec<PathBuf>, out: &Path, threads: usize) -> Summary {
    // real path -> (the path to show: the vault itself as the walk met it, and the ψ links that reach it). A link is a
    // `ψ` that is itself a symlink; a path that only passes through one (/var on macOS is /private/var) is not.
    let mut by_real: BTreeMap<PathBuf, (Option<PathBuf>, Vec<String>)> = BTreeMap::new();
    for p in found {
        let Ok(real) = std::fs::canonicalize(&p) else { continue }; // a dead link
        if !real.is_dir() {
            continue;
        }
        let e = by_real.entry(real.clone()).or_default();
        if std::fs::symlink_metadata(&p).is_ok_and(|m| m.file_type().is_symlink()) {
            e.1.push(p.display().to_string());
        } else if e.0.is_none() {
            e.0 = Some(p);
        }
    }
    let reals: Vec<(PathBuf, PathBuf, Vec<String>)> =
        by_real.into_iter().map(|(real, (shown, links))| (shown.unwrap_or_else(|| real.clone()), real, links)).collect();
    let dir = out.join("meta");
    let titles = last_titles(&dir);
    let n = if threads == 0 { std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8) } else { threads }.min(reals.len().max(1));
    let next = std::sync::atomic::AtomicUsize::new(0);
    let done: std::sync::Mutex<Vec<Vault>> = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..n {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some((shown, real, links)) = reals.get(i) else { break };
                let mut v = read_vault(shown, real, &titles);
                v.links = links.clone();
                v.links.sort();
                done.lock().unwrap().push(v);
            });
        }
    });
    let mut vaults = done.into_inner().unwrap();
    // a repo's own vaults first, then plain folders, then worktree copies; most notes first within each
    let rank = |k: &str| match k { "own" => 0, "plain" => 1, _ => 2 };
    vaults.sort_by(|a, b| rank(&a.kind).cmp(&rank(&b.kind)).then(b.notes.cmp(&a.notes)).then_with(|| a.path.cmp(&b.path)));
    for (i, v) in vaults.iter_mut().enumerate() {
        v.id = i;
    }
    let own: Vec<&Vault> = vaults.iter().filter(|v| v.kind != "worktree").collect();
    let s = Summary { vaults: own.len(), worktrees: vaults.len() - own.len(), notes: own.iter().map(|v| v.notes).sum(), bytes: own.iter().map(|v| v.bytes).sum() };
    // the notes, one file per vault (written fresh: ids follow this run's order)
    let vd = dir.join("vaults");
    let _ = std::fs::remove_dir_all(&vd);
    let _ = std::fs::create_dir_all(&vd);
    for v in &vaults {
        let _ = std::fs::write(vd.join(format!("{}.json", v.id)), serde_json::json!({"path": v.path, "items": v.items}).to_string());
    }
    let body = serde_json::json!({ "built_ms": chrono::Utc::now().timestamp_millis(), "vaults": vaults });
    let tmp = dir.join("vaults.json.part");
    if std::fs::write(&tmp, body.to_string()).is_ok() {
        let _ = std::fs::rename(&tmp, dir.join("vaults.json"));
    }
    s
}

/// The vaults in OUT/meta/vaults.json: their paths and links (what /api/reveal may open).
pub fn known(out: &Path) -> Vec<String> {
    let Ok(t) = std::fs::read_to_string(out.join("meta").join("vaults.json")) else { return vec![] };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) else { return vec![] };
    v["vaults"].as_array().into_iter().flatten().flat_map(|x| {
        std::iter::once(x["path"].as_str().map(str::to_string)).chain(x["links"].as_array().into_iter().flatten().map(|l| l.as_str().map(str::to_string)))
    }).flatten().collect()
}

/// What /api/reveal may open in a vault: the vault itself, or a path inside it (by its path or a link), that exists.
pub fn reveal_target(out: &Path, p: &str) -> Option<(String, bool)> {
    let p = p.trim_end_matches('/');
    if p.is_empty() || p.contains("/../") || p.ends_with("/..") {
        return None;
    }
    let inside = known(out).iter().any(|v| p == v || p.starts_with(&format!("{v}/")));
    let md = std::fs::metadata(p).ok()?;
    inside.then(|| (p.to_string(), md.is_file()))
}

/// OUT/meta/vaults/<id>.json, for /vaults/<id>.json (the id is digits only).
pub fn notes_file(out: &Path, id: &str) -> Option<PathBuf> {
    (!id.is_empty() && id.len() < 8 && id.bytes().all(|b| b.is_ascii_digit())).then(|| out.join("meta").join("vaults").join(format!("{id}.json")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_vault_its_link_and_a_worktree_copy() {
        let d = std::env::temp_dir().join(format!("relic-scan-test-{}-vault", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let repo = d.join("neo-oracle");
        let v = repo.join("ψ");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        std::fs::create_dir_all(v.join("memory/learnings")).unwrap();
        std::fs::write(v.join("memory/learnings/a.md"), "---\nx: 1\n---\n# Lesson one\nbody").unwrap();
        std::fs::write(v.join("inbox.md"), "no heading").unwrap();
        std::fs::write(v.join("memory/data.jsonl"), "{}").unwrap();
        // a cloned repo kept in the vault: skipped
        std::fs::create_dir_all(v.join("learn/org/tool/.git")).unwrap();
        std::fs::write(v.join("learn/org/tool/README.md"), "# Tool").unwrap();
        // a repo whose memory is the same vault, through a link
        std::fs::create_dir_all(d.join("tool")).unwrap();
        std::os::unix::fs::symlink(&v, d.join("tool").join("ψ")).unwrap();
        // a git worktree of neo-oracle with its own copy of the vault
        let wt = d.join("wt").join("neo-x");
        std::fs::create_dir_all(wt.join("ψ")).unwrap();
        std::fs::write(wt.join(".git"), format!("gitdir: {}/.git/worktrees/neo-x\n", repo.display())).unwrap();
        std::fs::write(wt.join("ψ/note.md"), "# Copy").unwrap();
        let out = d.join("out");
        let s = index(vec![v.clone(), d.join("tool").join("ψ"), wt.join("ψ")], &out, 2);
        assert_eq!((s.vaults, s.worktrees, s.notes), (1, 1, 2), "one vault, one worktree copy; the clone's README is not a note");
        let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(out.join("meta/vaults.json")).unwrap()).unwrap();
        let x = &j["vaults"][0];
        assert_eq!((x["name"].as_str(), x["kind"].as_str(), x["id"].as_u64()), (Some("neo-oracle"), Some("own"), Some(0)));
        assert_eq!(x["path"], v.display().to_string(), "shown as the walk met it");
        assert_eq!(x["links"], serde_json::json!([d.join("tool").join("ψ").display().to_string()]), "only the ψ that is a link");
        assert_eq!((x["files"].as_u64(), x["skipped_repos"].as_u64()), (Some(3), Some(1)));
        assert_eq!(x["sections"]["memory/learnings"], 1);
        assert!(x.get("items").is_none(), "the notes live in vaults/<id>.json");
        let w = &j["vaults"][1];
        assert_eq!((w["kind"].as_str(), w["origin"].as_str()), (Some("worktree"), Some(repo.display().to_string().as_str())));
        let notes: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(notes_file(&out, "0").unwrap()).unwrap()).unwrap();
        let titles: Vec<&str> = notes["items"].as_array().unwrap().iter().map(|i| i[2].as_str().unwrap()).collect();
        assert!(titles.contains(&"Lesson one") && titles.contains(&"inbox"), "{titles:?}");
        // a second run keeps a title without opening the note again
        std::fs::write(v.join("memory/learnings/a.md"), "# Lesson one\nbody!").unwrap();
        index(vec![v.clone()], &out, 2);
        let notes: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(notes_file(&out, "0").unwrap()).unwrap()).unwrap();
        assert!(notes["items"].as_array().unwrap().iter().any(|i| i[2] == "Lesson one"));
        let note = v.join("memory/learnings/a.md").display().to_string();
        assert_eq!(reveal_target(&out, &note), Some((note.clone(), true)));
        assert!(reveal_target(&out, &d.join("tool").join("ψ").join("inbox.md").display().to_string()).is_none(), "the link is gone from this run");
        assert!(reveal_target(&out, &d.display().to_string()).is_none(), "outside every vault");
        assert!(reveal_target(&out, &format!("{}/../x", v.display())).is_none());
        assert!(notes_file(&out, "../x").is_none() && notes_file(&out, "").is_none());
        let _ = std::fs::remove_dir_all(&d);
    }
}
