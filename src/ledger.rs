//! OUT/_ledger.tsv: every version of every transcript the index has read, from any path, appended and never rewritten.
//!
//! A transcript is a ledger already: its lines are only ever added. Each file's lines are folded into a hash chain
//! (chain₀ = 0, chainₙ = blake2b-128(chainₙ₋₁ ‖ hₙ), where h is the line's hash, the same `h` its rows carry), and every
//! run appends, for each file it read or recognised, the version it reached:
//!
//!   first (h of line 1) · lines · bytes · chain · session · last_ms · path · at · how (read | copy | seed)
//!
//! Before a new or changed path is parsed, the ledger answers what it is, wherever it came from:
//!   same          its whole chain is a known version: a copy, nothing to read (its state says copy_of)
//!   update        a known version is a prefix of it: read only what comes after that version
//!   past-changed  its first line is known but no known version is a prefix: the past changed; read it all
//!                 (old rows stay, as everywhere in relic-scan)
//!   never         its first line was never seen: read it all
//! Only a file whose first line is known is hashed through, so a never-seen file costs one line to answer.
//! The first run with a ledger seeds it from _state.tsv by hashing each file up to where it was read (read only).

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;

pub type Chain = [u8; 16];

/// The chain after one more line whose hash is `h`.
pub fn step(prev: &Chain, h: i64) -> Chain {
    let mut st = blake2b_simd::Params::new().hash_length(16).to_state();
    st.update(prev).update(&h.to_be_bytes());
    let mut c = [0u8; 16];
    c.copy_from_slice(st.finalize().as_bytes());
    c
}

pub fn hex(c: &Chain) -> String {
    c.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn unhex(s: &str) -> Option<Chain> {
    if s.len() != 32 {
        return None;
    }
    let mut c = [0u8; 16];
    for (i, b) in c.iter_mut().enumerate() {
        *b = u8::from_str_radix(s.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(c)
}

/// One version of one file: where its lines had reached, and the ctx a parse resumes from there with.
#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    pub first: i64,
    pub lines: i64,
    pub bytes: u64,
    pub chain: Chain,
    pub session: String,
    pub last_ms: Option<i64>,
    pub path: String,
}

impl Version {
    pub fn of(path: &str, c: &crate::parse::Ctx) -> Version {
        Version { first: c.first, lines: c.lines, bytes: c.offset, chain: c.chain, session: c.session.clone(), last_ms: c.last_ms,
                  path: path.to_string() }
    }
    /// The ctx a parse of a file holding this version goes on from.
    pub fn ctx(&self) -> crate::parse::Ctx {
        crate::parse::Ctx { offset: self.bytes, lines: self.lines, session: self.session.clone(), last_ms: self.last_ms,
                            first: self.first, chain: self.chain, model: String::new(), meta: Default::default() }
    }
}

/// Every known version, by the hash of its first line; one entry per (first, lines, chain), the first path seen.
#[derive(Default)]
pub struct Ledger {
    pub by_first: HashMap<i64, Vec<Version>>,
    pub entries: usize,
}

impl Ledger {
    fn add(&mut self, v: Version) {
        self.entries += 1;
        let vs = self.by_first.entry(v.first).or_default();
        if !vs.iter().any(|x| x.lines == v.lines && x.chain == v.chain) {
            vs.push(v);
            vs.sort_by_key(|x| x.lines);
        }
    }
}

pub fn path(out: &Path) -> PathBuf {
    out.join("_ledger.tsv")
}

const HEAD: &str = "first\tlines\tbytes\tchain\tsession\tlast_ms\tpath\tat\thow";

/// The ledger, or None when this index has none yet.
pub fn load(out: &Path) -> Option<Ledger> {
    let f = fs::File::open(path(out)).ok()?;
    let mut l = Ledger::default();
    for line in BufReader::new(f).lines().skip(1).map_while(Result::ok) {
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() < 9 {
            continue; // a line torn by a crash: the rest of the ledger still holds
        }
        let (Ok(first), Ok(lines), Ok(bytes), Some(chain)) = (c[0].parse(), c[1].parse(), c[2].parse(), unhex(c[3])) else { continue };
        l.add(Version { first, lines, bytes, chain, session: c[4].to_string(), last_ms: c[5].parse().ok(), path: c[6].to_string() });
    }
    Some(l)
}

/// Append versions, each line written whole; `how`: read (parsed this run), copy (recognised, not read), seed.
pub fn append(out: &Path, vs: &[(Version, &str)]) {
    if vs.is_empty() {
        return;
    }
    let p = path(out);
    let fresh = !p.exists();
    let Ok(f) = fs::OpenOptions::new().create(true).append(true).open(&p) else { return };
    let mut w = std::io::BufWriter::new(f);
    if fresh {
        let _ = writeln!(w, "{HEAD}");
    }
    let at = chrono::Utc::now().timestamp();
    for (v, how) in vs {
        if v.lines == 0 || v.path.contains(['\t', '\n', '\r']) {
            continue;
        }
        let _ = writeln!(w, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{at}\t{how}", v.first, v.lines, v.bytes, hex(&v.chain), v.session,
                         v.last_ms.map(|x| x.to_string()).unwrap_or_default(), v.path);
    }
    let _ = w.flush();
}

#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    Same(Version),
    Update(Version),
    PastChanged,
    Never,
}

impl Answer {
    pub fn name(&self) -> &'static str {
        match self {
            Answer::Same(_) => "same",
            Answer::Update(_) => "update",
            Answer::PastChanged => "past-changed",
            Answer::Never => "never",
        }
    }
}

/// What the file at `path` is, against the ledger (read only; see the module doc).
pub fn answer(l: &Ledger, path: &str) -> Answer {
    let quiet = AtomicU64::new(0);
    let mut first: Option<i64> = None;
    let mut versions: &[Version] = &[];
    let (mut n, mut chain, mut bytes) = (0i64, Chain::default(), 0u64);
    let mut best: Option<&Version> = None;
    let mut next = 0usize;
    let read = crate::parse::each_line(path, 0, 8 << 20, &quiet, &mut |line: &[u8]| {
        let h = crate::engine::key(line);
        if first.is_none() {
            first = Some(h);
            match l.by_first.get(&h) {
                Some(vs) => versions = vs,
                None => return false, // never seen: one line was enough
            }
        }
        n += 1;
        bytes += line.len() as u64;
        chain = step(&chain, h);
        while next < versions.len() && versions[next].lines < n {
            next += 1;
        }
        while next < versions.len() && versions[next].lines == n {
            if versions[next].chain == chain && versions[next].bytes == bytes {
                best = Some(&versions[next]);
            }
            next += 1;
        }
        true
    });
    if read.is_none() || first.is_none() {
        return Answer::Never;
    }
    if versions.is_empty() {
        return Answer::Never;
    }
    match best {
        Some(v) if v.lines == n && v.bytes == bytes => Answer::Same(v.clone()),
        Some(v) => Answer::Update(v.clone()),
        None => Answer::PastChanged,
    }
}

/// The ctx of a file read from 0 to `offset` (the bytes a state entry covers), with its chain: None if the file no
/// longer holds those bytes as lines (it changed since).
pub fn chain_upto(path: &str, offset: u64, lines: i64) -> Option<(i64, Chain)> {
    let quiet = AtomicU64::new(0);
    let (mut n, mut first, mut chain, mut bytes) = (0i64, 0i64, Chain::default(), 0u64);
    crate::parse::each_line(path, 0, 8 << 20, &quiet, &mut |line: &[u8]| {
        if bytes >= offset {
            return false;
        }
        let h = crate::engine::key(line);
        n += 1;
        if n == 1 {
            first = h;
        }
        chain = step(&chain, h);
        bytes += line.len() as u64;
        true
    })?;
    (bytes == offset && n == lines && n > 0).then_some((first, chain))
}

/// The first run with a ledger: hash every file of the state up to where it was read, in parallel, set its chain,
/// and write the ledger. Returns how many files it seeded.
pub fn seed(out: &Path, st: &mut HashMap<String, crate::state::FileState>, threads: usize) -> usize {
    let todo: Vec<(String, u64, i64)> = st.iter().filter(|(_, s)| s.ctx.offset > 0 && s.ctx.chain == Chain::default())
        .map(|(p, s)| (p.clone(), s.ctx.offset, s.ctx.lines)).collect();
    let n = if threads == 0 { std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8) } else { threads };
    let next = std::sync::atomic::AtomicUsize::new(0);
    let done: std::sync::Mutex<Vec<(String, i64, Chain)>> = std::sync::Mutex::default();
    std::thread::scope(|sc| {
        for _ in 0..n {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some((p, off, lines)) = todo.get(i) else { break };
                if let Some((first, chain)) = chain_upto(p, *off, *lines) {
                    done.lock().unwrap().push((p.clone(), first, chain));
                }
            });
        }
    });
    let done = done.into_inner().unwrap();
    let mut vs = Vec::with_capacity(done.len() + 1);
    for (p, first, chain) in done {
        if let Some(s) = st.get_mut(&p) {
            s.ctx.first = first;
            s.ctx.chain = chain;
            vs.push((Version::of(&p, &s.ctx), "seed"));
        }
    }
    vs.sort_by(|a, b| a.0.path.cmp(&b.0.path));
    // the ledger file exists from now on, even when nothing could be seeded
    if vs.is_empty() && !path(out).exists() {
        let _ = fs::write(path(out), format!("{HEAD}\n"));
    }
    append(out, &vs);
    vs.len()
}

/// `answer` for many paths at once, in parallel; the answers in the order of `paths`.
pub fn answer_all(l: &Ledger, paths: &[String], threads: usize) -> Vec<Answer> {
    let n = if threads == 0 { std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8) } else { threads };
    let next = std::sync::atomic::AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<Option<Answer>>> = paths.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|sc| {
        for _ in 0..n.min(paths.len().max(1)) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(p) = paths.get(i) else { break };
                *out[i].lock().unwrap() = Some(answer(l, p));
            });
        }
    });
    out.into_iter().map(|m| m.into_inner().unwrap().unwrap_or(Answer::Never)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("relic-scan-test-{}-ledger-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }
    fn line(i: usize, word: &str) -> String {
        format!(r#"{{"type":"user","sessionId":"s1","timestamp":"2026-07-01T00:{:02}:00.000Z","message":{{"role":"user","content":"{word} {i}"}}}}"#, i % 60) + "\n"
    }
    fn body(n: usize, word: &str) -> String {
        (0..n).map(|i| line(i, word)).collect()
    }

    #[test]
    fn answers_same_update_past_changed_never() {
        let d = tmp("answers");
        let a = d.join("a.jsonl");
        fs::write(&a, body(10, "hello")).unwrap();
        let ap = a.to_str().unwrap();
        // the chain the parser keeps is the chain the ledger computes from the bytes
        let st = crate::parse::Stats::default();
        let ctx = crate::parse::parse_file(ap, &crate::parse::Ctx::default(), false, &st, &mut |_| {}).unwrap();
        assert_eq!(chain_upto(ap, ctx.offset, ctx.lines), Some((ctx.first, ctx.chain)));
        let mut l = Ledger::default();
        l.add(Version::of(ap, &ctx));
        let w = |name: &str, s: String| { let p = d.join(name); fs::write(&p, s).unwrap(); p.to_str().unwrap().to_string() };
        assert_eq!(answer(&l, &w("copy.jsonl", body(10, "hello"))), Answer::Same(Version::of(ap, &ctx)));
        assert_eq!(answer(&l, &w("half-line.jsonl", body(10, "hello") + "{\"type\":")), Answer::Same(Version::of(ap, &ctx)),
                   "a last line still being written is not part of the version");
        assert_eq!(answer(&l, &w("grown.jsonl", body(12, "hello"))), Answer::Update(Version::of(ap, &ctx)));
        let mut edited = body(10, "hello");
        edited = edited.replacen("hello 3", "HELLO 3", 1);
        assert_eq!(answer(&l, &w("edited.jsonl", edited)), Answer::PastChanged);
        assert_eq!(answer(&l, &w("other.jsonl", body(10, "other"))), Answer::Never);
        assert_eq!(answer(&l, &w("empty.jsonl", String::new())), Answer::Never);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn a_run_skips_copies_reads_updates_from_the_known_version_and_keeps_the_rows() {
        let d = tmp("run");
        let (root, out) = (d.join("root"), d.join("out"));
        let home = root.join(".claude/projects/-tmp-x");
        fs::create_dir_all(&home).unwrap();
        let name = "0b867d73-405b-4b03-8c46-aee7648f1bd7.jsonl";
        fs::write(home.join(name), body(20, "hello")).unwrap();
        let args = |dry_run| crate::RunArgs {
            out: out.display().to_string(), roots: vec![root.display().to_string()], depth: 8, threads: 2, exclude: vec![], full: false,
            agent: Some("t@x".into()), jt_compat: false, only: vec![], kind: vec![], month: None, day: None, from: None, to: None, dry_run,
        };
        let r1 = crate::run::run_inner(args(false), None, false);
        assert_eq!((r1.new, r1.copies), (1, 0));
        let rows = crate::parse::tests::month_rows(&out.join("2026-07.parquet"));
        // a mirror of that home, and another machine's copy that has two lines more
        let mirror = root.join(".relic-mirrors/mba/.claude/projects/-tmp-x");
        let other = root.join(".relic-mirrors/white/.claude/projects/-tmp-x");
        fs::create_dir_all(&mirror).unwrap();
        fs::create_dir_all(&other).unwrap();
        fs::write(mirror.join(name), body(20, "hello")).unwrap();
        fs::write(other.join(name), body(22, "hello")).unwrap();
        let r2 = crate::run::run_inner(args(false), None, false);
        assert_eq!((r2.new, r2.copies, r2.resumed, r2.past_changed), (2, 1, 1, 0));
        assert_eq!(r2.bytes, (line(20, "hello").len() + line(21, "hello").len()) as u64, "only the two new lines are read");
        let after = crate::parse::tests::month_rows(&out.join("2026-07.parquet"));
        assert_eq!(after.len(), rows.len() + 2, "the copy adds no row, the update adds its two");
        let st = crate::state::load(&out);
        let m = mirror.join(name).display().to_string();
        assert_eq!(st[&m].copy_of.as_deref(), Some(home.join(name).display().to_string().as_str()));
        assert_eq!(st[&m].ctx.lines, 20);
        let led = load(&out).unwrap();
        assert!(led.entries >= 3, "a read, a copy and an update are in the ledger");
        let _ = fs::remove_dir_all(&d);
    }
}
