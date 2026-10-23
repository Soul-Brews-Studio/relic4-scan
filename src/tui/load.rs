//! Loading OUT/meta into the app, and the list helpers (filter, switch, copy).

use super::*;

impl App {
    pub(super) fn load(out: &Path) -> App {
        let dir = out.join("meta");
        let mut sess = vec![];
        for b in read(&dir.join("sessions.parquet")) {
            let (base, tier, sid, cat, proj, f, l) = (scol(&b, "base"), scol(&b, "tier"), scol(&b, "sid"), scol(&b, "cat"),
                                                      scol(&b, "proj"), scol(&b, "first_ts"), scol(&b, "last_ts"));
            let (rows, copies, sz) = (icol(&b, "n_rows"), icol(&b, "copies"), icol(&b, "size"));
            let by: Vec<_> = crate::meta::TYPES.iter().map(|t| icol(&b, t)).collect();
            for k in 0..b.num_rows() {
                let t = match tier.value(k) { "main" => 0, "subagent" => 1, _ => 2 };
                let sid = get(sid, k).unwrap_or(base.value(k)).to_string();
                let repo = get(cat, k).unwrap_or("(none)").to_string();
                let proj = get(proj, k).unwrap_or("").to_string();
                let find = format!("{sid} {repo} {proj} {}", ["main", "subagent", "workflow"][t]).to_lowercase();
                sess.push(Sess {
                    sid, tier: t, repo, t0: epoch(get(f, k)), t1: epoch(get(l, k)), rows: rows.value(k),
                    by: std::array::from_fn(|j| by[j].value(k)), copies: copies.value(k), bytes: sz.value(k), proj, find,
                });
            }
        }
        sess.sort_by(|a, b| b.t0.cmp(&a.t0));
        let mut rm: BTreeMap<String, Repo> = BTreeMap::new();
        for s in &sess {
            let r = rm.entry(s.repo.clone()).or_insert_with(|| Repo {
                name: s.repo.clone(), n: [0; 3], rows: 0, by: [0; 6], last: None, find: s.repo.to_lowercase(),
            });
            r.n[s.tier] += 1;
            r.rows += s.rows;
            for j in 0..6 {
                r.by[j] += s.by[j];
            }
            r.last = r.last.max(s.t1.or(s.t0));
        }
        let mut repos: Vec<Repo> = rm.into_values().collect();
        repos.sort_by(|a, b| b.rows.cmp(&a.rows));
        // paths: rows per file from files.parquet, every .jsonl from jsonl.parquet
        let mut rows_of: HashMap<String, i64> = HashMap::new();
        for b in read(&dir.join("files.parquet")) {
            let (p, r) = (scol(&b, "path"), icol(&b, "n_rows"));
            for k in 0..b.num_rows() {
                rows_of.insert(p.value(k).to_string(), r.value(k));
            }
        }
        let (mut dirs, mut files, mut dir_at) = (vec![], vec![], HashMap::new());
        if dir.join("jsonl.parquet").exists() {
            for b in read(&dir.join("jsonl.parquet")) {
                let (path, place, verdict, sz, mt, parsed) = (scol(&b, "path"), scol(&b, "place"), scol(&b, "verdict"), icol(&b, "size"),
                                                              icol(&b, "mtime_ms"), crate::map::b(&b, "parsed"));
                for k in 0..b.num_rows() {
                    let p = path.value(k);
                    let (d, name) = p.rsplit_once('/').unwrap_or(("", p));
                    let di = *dir_at.entry(d.to_string()).or_insert_with(|| {
                        dirs.push(Dir { path: d.to_string(), files: vec![], by: [0; 4], bytes: 0, rows: 0, find: d.to_lowercase() });
                        dirs.len() - 1
                    });
                    let status = match place.value(k) {
                        "home" => if parsed.value(k) { 0 } else { 1 },
                        "vault" => 2,
                        _ => 3,
                    };
                    let rows = rows_of.get(p).copied().unwrap_or(-1);
                    let dd = &mut dirs[di];
                    dd.files.push(files.len());
                    dd.by[status] += 1;
                    dd.bytes += sz.value(k);
                    dd.rows += rows.max(0);
                    files.push(PFile {
                        name: name.to_string(), dir: di, status, verdict: get(verdict, k).unwrap_or("").to_string(), size: sz.value(k), rows,
                        mtime: mt.value(k) / 1000,
                    });
                }
            }
        }
        dirs.sort_by(|a, b| b.files.len().cmp(&a.files.len()));
        for (di, d) in dirs.iter().enumerate() {
            for &f in &d.files {
                files[f].dir = di;
            }
        }
        let mut state = TableState::default();
        state.select(Some(0));
        App {
            out: out.display().to_string(), sess, repos, dirs, files, tab: Tab::Overview, filter: String::new(), typing: false,
            repo_pick: None, dir_pick: None, open: None, state, toast: None, home: std::env::var("HOME").unwrap_or_default(),
        }
    }

    pub(super) fn tilde(&self, p: &str) -> String {
        match p.strip_prefix(&self.home) {
            Some(r) if !self.home.is_empty() => format!("~{r}"),
            _ => p.to_string(),
        }
    }

    pub(super) fn words(&self) -> Vec<String> {
        self.filter.to_lowercase().split_whitespace().map(str::to_string).collect()
    }

    /// The indices the current list shows, in order.
    pub(super) fn visible(&self) -> Vec<usize> {
        let w = self.words();
        let hit = |s: &str| w.iter().all(|x| s.contains(x.as_str()));
        match self.tab {
            Tab::Overview => vec![],
            Tab::Repos => (0..self.repos.len()).filter(|&k| hit(&self.repos[k].find)).collect(),
            Tab::Sessions => (0..self.sess.len())
                .filter(|&k| self.repo_pick.as_ref().is_none_or(|r| *r == self.sess[k].repo) && hit(&self.sess[k].find)).collect(),
            Tab::Paths => match self.dir_pick {
                Some(d) => {
                    let mut v: Vec<usize> = self.dirs[d].files.iter().copied()
                        .filter(|&f| hit(&format!("{} {} {}", self.files[f].name, STATUS[self.files[f].status].0, self.files[f].verdict).to_lowercase()))
                        .collect();
                    v.sort_by(|&a, &b| self.files[b].size.cmp(&self.files[a].size));
                    v
                }
                None => (0..self.dirs.len()).filter(|&k| hit(&self.dirs[k].find) || hit(&self.tilde(&self.dirs[k].path).to_lowercase())).collect(),
            },
        }
    }

    pub(super) fn switch(&mut self, t: Tab) {
        self.tab = t;
        self.filter.clear();
        self.open = None;
        self.state.select(Some(0));
    }

    pub(super) fn copy(&mut self, text: &str) {
        use std::io::Write;
        let ok = ["pbcopy", "wl-copy", "xclip"].iter().any(|c| {
            let mut cmd = std::process::Command::new(c);
            if *c == "xclip" {
                cmd.args(["-selection", "clipboard"]);
            }
            cmd.stdin(std::process::Stdio::piped()).spawn().ok()
                .and_then(|mut ch| { ch.stdin.take()?.write_all(text.as_bytes()).ok()?; ch.wait().ok() })
                .is_some_and(|s| s.success())
        });
        self.toast = Some((if ok { format!("copied: {text}") } else { format!("no clipboard tool; {text}") }, Instant::now()));
    }

}
