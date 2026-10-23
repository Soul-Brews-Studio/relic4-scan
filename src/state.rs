//! out/_state.tsv: where every file was left, so the next run reads only what is new.
//!
//!   new        not in the state                       -> parse from 0
//!   grown      same inode, size > offset, same prefix -> parse from offset with the saved ctx
//!   rewritten  other inode, size < offset, or the
//!              first bytes changed                    -> parse from 0 again (old rows stay: truth is append-only)
//!   unchanged  size == offset                         -> nothing read
//!   gone       in the state, not on disk              -> dropped from the state (its rows stay)
//! The prefix check reads at most 4 KB, and only for files whose size changed.

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

use crate::parse::Ctx;

pub const PREFIX: u64 = 4096;

#[derive(Clone, Debug)]
pub struct FileState {
    pub dev: u64,
    pub ino: u64,
    pub size: u64,
    pub ctx: Ctx,
    pub prefix_len: u64,
    pub prefix_hash: i64,
    /// a copy the ledger recognised (same lines as a version read elsewhere): that file's path; its rows are there
    pub copy_of: Option<String>,
}

pub fn prefix_hash(path: &str, len: u64) -> Option<i64> {
    let mut buf = vec![0u8; len as usize];
    let mut f = fs::File::open(path).ok()?;
    f.read_exact(&mut buf).ok()?;
    Some(crate::engine::key(&buf))
}

pub fn load(out: &Path) -> HashMap<String, FileState> {
    let mut m = HashMap::new();
    let Ok(f) = fs::File::open(out.join("_state.tsv")) else { return m };
    for line in BufReader::new(f).lines().skip(1).map_while(Result::ok) {
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() < 10 {
            continue;
        }
        let num = |i: usize| c[i].parse::<i64>().unwrap_or(0);
        // columns 10-12 (first, chain, copy_of) came with the ledger; an older state has none and is seeded once
        m.insert(c[0].to_string(), FileState {
            dev: num(1) as u64, ino: num(2) as u64, size: num(3) as u64,
            ctx: Ctx { offset: num(4) as u64, lines: num(5), session: c[6].to_string(), last_ms: c[7].parse().ok(),
                       first: c.get(10).and_then(|x| x.parse().ok()).unwrap_or(0),
                       chain: c.get(11).and_then(|x| crate::ledger::unhex(x)).unwrap_or_default() },
            prefix_len: num(8) as u64, prefix_hash: num(9),
            copy_of: c.get(12).filter(|x| !x.is_empty()).map(|x| x.to_string()),
        });
    }
    m
}

pub fn save(out: &Path, m: &HashMap<String, FileState>) {
    let tmp = out.join("_state.tsv.part");
    let mut w = std::io::BufWriter::new(fs::File::create(&tmp).unwrap());
    writeln!(w, "path\tdev\tino\tsize\toffset\tlines\tsession\tlast_ms\tprefix_len\tprefix_hash\tfirst\tchain\tcopy_of").unwrap();
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    for k in keys {
        let s = &m[k];
        writeln!(w, "{k}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", s.dev, s.ino, s.size, s.ctx.offset, s.ctx.lines,
                 s.ctx.session, s.ctx.last_ms.map(|x| x.to_string()).unwrap_or_default(), s.prefix_len, s.prefix_hash, s.ctx.first,
                 crate::ledger::hex(&s.ctx.chain), s.copy_of.as_deref().unwrap_or("")).unwrap();
    }
    // on disk before the rename, so a power cut can never leave a state that points past what was written
    w.into_inner().unwrap().sync_all().unwrap();
    fs::rename(tmp, out.join("_state.tsv")).unwrap(); // the state is written LAST: a crash re-reads, h folds it
}
