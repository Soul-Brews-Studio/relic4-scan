//! `relic-scan tui`: the same data as the page, in the terminal. Reads OUT/meta/{sessions,files,jsonl}.parquet.
//!
//!   1 overview   totals, rows by type, sessions per month, homes
//!   2 repos      one row per repo; Enter lists its sessions
//!   3 sessions   newest first; Enter opens one (c copies `claude --resume <sid>`)
//!   4 paths      every .jsonl folder the scan saw (indexed / skipped / vault / loose); Enter lists its files
//!
//! / filters the current list (words, AND), Esc clears, Backspace goes back, q quits.
//! No background colour is set anywhere, so it reads the same on a dark or a light terminal.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Cell, Clear, Paragraph, Row, Table, TableState, Wrap};
use ratatui::{DefaultTerminal, Frame};

use crate::map::{get, i as icol, read, s as scol};

const CYAN: Color = Color::Rgb(0, 214, 255);
const PINK: Color = Color::Rgb(255, 60, 172);
const AMBER: Color = Color::Rgb(255, 184, 0);
const VIOLET: Color = Color::Rgb(160, 120, 255);
const RED: Color = Color::Rgb(255, 99, 72);
const DIM: Color = Color::Rgb(128, 136, 156);
const TYPE_LABEL: [&str; 6] = ["user text", "assistant text", "thinking", "tool calls", "tool results", "tool errors"];
const TYPE_COLOR: [Color; 6] = [AMBER, Color::Rgb(230, 200, 120), PINK, CYAN, Color::Rgb(70, 150, 230), RED];
const TIER_GLYPH: [&str; 3] = ["●", "◦", "·"];
const TIER_COLOR: [Color; 3] = [AMBER, CYAN, VIOLET];
const STATUS: [(&str, Color); 4] = [("indexed", CYAN), ("skipped", RED), ("vault", VIOLET), ("loose", DIM)];

struct Sess {
    sid: String,
    tier: usize,
    repo: String,
    t0: Option<i64>,
    t1: Option<i64>,
    rows: i64,
    by: [i64; 6],
    copies: i64,
    bytes: i64,
    proj: String,
    find: String,
}

struct Repo {
    name: String,
    n: [i64; 3],
    rows: i64,
    by: [i64; 6],
    last: Option<i64>,
    find: String,
}

struct PFile {
    name: String,
    dir: usize,
    status: usize,
    verdict: String,
    size: i64,
    rows: i64,
    mtime: i64,
}

struct Dir {
    path: String,
    files: Vec<usize>,
    by: [i64; 4],
    bytes: i64,
    rows: i64,
    find: String,
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Overview,
    Repos,
    Sessions,
    Paths,
}

struct App {
    out: String,
    sess: Vec<Sess>,
    repos: Vec<Repo>,
    dirs: Vec<Dir>,
    files: Vec<PFile>,
    tab: Tab,
    filter: String,
    typing: bool,
    repo_pick: Option<String>, // sessions of one repo (Enter on a repo)
    dir_pick: Option<usize>,   // files of one folder (Enter on a folder)
    open: Option<usize>,       // the session shown in the popup
    state: TableState,
    toast: Option<(String, Instant)>,
    home: String,
}

fn epoch(ts: Option<&str>) -> Option<i64> {
    crate::map::epoch(ts).as_i64()
}

/// epoch seconds -> Bangkok "YYYY-MM-DD HH:MM"
fn bkk(t: Option<i64>) -> String {
    t.and_then(|t| chrono::DateTime::from_timestamp(t + 7 * 3600, 0)).map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or("—".into())
}

fn num(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

fn size(b: i64) -> String {
    let b = b as f64;
    if b >= 1e9 { format!("{:.2} GB", b / 1e9) } else if b >= 1e6 { format!("{:.1} MB", b / 1e6) } else { format!("{:.0} KB", b / 1e3) }
}

/// A bar of `w` cells for `frac` of the whole, in eighths.
fn bar(frac: f64, w: usize) -> String {
    let cells = (frac.clamp(0.0, 1.0) * w as f64 * 8.0).round() as usize;
    let mut s = "█".repeat(cells / 8);
    if cells % 8 > 0 {
        s.push(['▏', '▎', '▍', '▌', '▋', '▊', '▉'][cells % 8 - 1]);
    }
    format!("{s:<w$}")
}

impl App {
    fn load(out: &Path) -> App {
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

    fn tilde(&self, p: &str) -> String {
        match p.strip_prefix(&self.home) {
            Some(r) if !self.home.is_empty() => format!("~{r}"),
            _ => p.to_string(),
        }
    }

    fn words(&self) -> Vec<String> {
        self.filter.to_lowercase().split_whitespace().map(str::to_string).collect()
    }

    /// The indices the current list shows, in order.
    fn visible(&self) -> Vec<usize> {
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

    fn switch(&mut self, t: Tab) {
        self.tab = t;
        self.filter.clear();
        self.open = None;
        self.state.select(Some(0));
    }

    fn copy(&mut self, text: &str) {
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

    /// One key. Returns false to quit.
    fn key(&mut self, code: KeyCode, mods: KeyModifiers) -> bool {
        if self.typing {
            match code {
                KeyCode::Esc => {
                    self.typing = false;
                    self.filter.clear();
                }
                KeyCode::Enter => self.typing = false,
                KeyCode::Backspace => {
                    self.filter.pop();
                }
                KeyCode::Char(c) => self.filter.push(c),
                _ => {}
            }
            self.state.select(Some(0));
            return true;
        }
        if self.open.is_some() {
            match code {
                KeyCode::Char('c') => {
                    let s = &self.sess[self.open.unwrap()];
                    let t = if s.tier == 0 { format!("claude --resume {}", s.sid) } else { s.sid.clone() };
                    self.copy(&t);
                }
                KeyCode::Char('q') if mods.is_empty() => return false,
                _ => self.open = None,
            }
            return true;
        }
        let n = self.visible().len();
        let at = self.state.selected().unwrap_or(0);
        let go = |s: &mut TableState, x: usize| s.select(Some(x.min(n.saturating_sub(1))));
        match code {
            KeyCode::Char('q') => return false,
            KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => return false,
            KeyCode::Char('1') => self.switch(Tab::Overview),
            KeyCode::Char('2') => self.switch(Tab::Repos),
            KeyCode::Char('3') => {
                self.repo_pick = None;
                self.switch(Tab::Sessions)
            }
            KeyCode::Char('4') => {
                self.dir_pick = None;
                self.switch(Tab::Paths)
            }
            KeyCode::Tab | KeyCode::Right => self.switch(match self.tab {
                Tab::Overview => Tab::Repos,
                Tab::Repos => Tab::Sessions,
                Tab::Sessions => Tab::Paths,
                Tab::Paths => Tab::Overview,
            }),
            KeyCode::BackTab | KeyCode::Left => self.switch(match self.tab {
                Tab::Overview => Tab::Paths,
                Tab::Repos => Tab::Overview,
                Tab::Sessions => Tab::Repos,
                Tab::Paths => Tab::Sessions,
            }),
            KeyCode::Char('/') => self.typing = true,
            KeyCode::Esc => {
                self.filter.clear();
                self.state.select(Some(0));
            }
            KeyCode::Down | KeyCode::Char('j') => go(&mut self.state, at + 1),
            KeyCode::Up | KeyCode::Char('k') => go(&mut self.state, at.saturating_sub(1)),
            KeyCode::PageDown | KeyCode::Char(' ') => go(&mut self.state, at + 20),
            KeyCode::PageUp => go(&mut self.state, at.saturating_sub(20)),
            KeyCode::Home | KeyCode::Char('g') => go(&mut self.state, 0),
            KeyCode::End | KeyCode::Char('G') => go(&mut self.state, usize::MAX),
            KeyCode::Enter => {
                let v = self.visible();
                let Some(&k) = v.get(at) else { return true };
                match self.tab {
                    Tab::Repos => {
                        self.repo_pick = Some(self.repos[k].name.clone());
                        self.switch(Tab::Sessions);
                    }
                    Tab::Sessions => self.open = Some(k),
                    Tab::Paths if self.dir_pick.is_none() => {
                        self.dir_pick = Some(k);
                        self.filter.clear();
                        self.state.select(Some(0));
                    }
                    Tab::Paths => {
                        let f = &self.files[k];
                        let p = format!("{}/{}", self.dirs[f.dir].path, f.name);
                        self.copy(&p);
                    }
                    Tab::Overview => {}
                }
            }
            KeyCode::Backspace => {
                if self.tab == Tab::Paths && self.dir_pick.is_some() {
                    self.dir_pick = None;
                    self.state.select(Some(0));
                } else if self.tab == Tab::Sessions && self.repo_pick.is_some() {
                    self.repo_pick = None;
                    self.switch(Tab::Repos);
                }
            }
            _ => {}
        }
        true
    }

    fn draw(&mut self, f: &mut Frame) {
        let [head, body, foot] = Layout::vertical([Constraint::Length(3), Constraint::Min(5), Constraint::Length(1)]).areas(f.area());
        self.header(f, head);
        match self.tab {
            Tab::Overview => self.overview(f, body),
            Tab::Repos => self.repos_view(f, body),
            Tab::Sessions => self.sessions_view(f, body),
            Tab::Paths => self.paths_view(f, body),
        }
        self.footer(f, foot);
        if let Some(k) = self.open {
            self.popup(f, k);
        }
    }

    fn header(&self, f: &mut Frame, a: Rect) {
        let title = Line::from(vec![
            Span::styled("▓▒░ ", Style::new().fg(PINK)),
            Span::styled("RELIC", Style::new().fg(PINK).bold()),
            Span::styled("·", Style::new().fg(DIM)),
            Span::styled("SCAN", Style::new().fg(CYAN).bold()),
            Span::styled(" ░▒▓", Style::new().fg(CYAN)),
            Span::styled(format!("   {} sessions · {} .jsonl · {}", num(self.sess.len() as i64), num(self.files.len() as i64), self.tilde(&self.out)),
                         Style::new().fg(DIM)),
        ]);
        let tabs: Vec<Span> = [(Tab::Overview, "1 overview"), (Tab::Repos, "2 repos"), (Tab::Sessions, "3 sessions"), (Tab::Paths, "4 paths")]
            .iter()
            .flat_map(|(t, l)| {
                let st = if *t == self.tab { Style::new().fg(Color::Black).bg(CYAN).bold() } else { Style::new().fg(DIM) };
                [Span::styled(format!(" {l} "), st), Span::raw(" ")]
            })
            .collect();
        f.render_widget(Paragraph::new(vec![title, Line::from(tabs)]), a.inner(ratatui::layout::Margin { horizontal: 1, vertical: 0 }));
    }

    fn footer(&self, f: &mut Frame, a: Rect) {
        let line = if self.typing {
            Line::from(vec![Span::styled(" / ", Style::new().fg(Color::Black).bg(PINK)), Span::raw(format!(" {}▏", self.filter))])
        } else if let Some((t, at)) = self.toast.as_ref().filter(|(_, at)| at.elapsed() < Duration::from_secs(3)) {
            let _ = at;
            Line::from(Span::styled(format!(" {t}"), Style::new().fg(CYAN)))
        } else {
            let mut s = vec![Span::styled(" q", Style::new().fg(PINK)), Span::styled(" quit  ", Style::new().fg(DIM)),
                             Span::styled("1-4/tab", Style::new().fg(PINK)), Span::styled(" view  ", Style::new().fg(DIM)),
                             Span::styled("/", Style::new().fg(PINK)), Span::styled(" filter  ", Style::new().fg(DIM)),
                             Span::styled("↑↓ jk g G", Style::new().fg(PINK)), Span::styled(" move  ", Style::new().fg(DIM)),
                             Span::styled("enter", Style::new().fg(PINK)), Span::styled(" open  ", Style::new().fg(DIM)),
                             Span::styled("⌫", Style::new().fg(PINK)), Span::styled(" back", Style::new().fg(DIM))];
            if !self.filter.is_empty() {
                s.push(Span::styled(format!("   filter: {}", self.filter), Style::new().fg(AMBER)));
            }
            Line::from(s)
        };
        f.render_widget(Paragraph::new(line), a);
    }

    fn frame<'a>(&self, title: String) -> Block<'a> {
        Block::bordered().border_type(BorderType::Rounded).border_style(Style::new().fg(DIM))
            .title(Line::from(Span::styled(format!(" {title} "), Style::new().fg(CYAN).bold())))
    }

    fn overview(&self, f: &mut Frame, a: Rect) {
        let [top, bottom] = Layout::vertical([Constraint::Length(10), Constraint::Min(6)]).areas(a);
        let [stats, types] = Layout::horizontal([Constraint::Length(38), Constraint::Min(40)]).areas(top);
        let mut n = [0i64; 3];
        let (mut rows, mut by) = (0i64, [0i64; 6]);
        let (mut lo, mut hi) = (i64::MAX, i64::MIN);
        for s in &self.sess {
            n[s.tier] += 1;
            rows += s.rows;
            for j in 0..6 {
                by[j] += s.by[j];
            }
            if let Some(t) = s.t0 {
                lo = lo.min(t);
                hi = hi.max(s.t1.unwrap_or(t));
            }
        }
        let bytes: i64 = self.files.iter().filter(|f| f.status == 0).map(|f| f.size).sum();
        let kv = |k: &str, v: String, c: Color| Line::from(vec![Span::styled(format!("{k:<12}"), Style::new().fg(DIM)), Span::styled(v, Style::new().fg(c).bold())]);
        let lines = vec![
            kv("sessions", num(self.sess.len() as i64), Color::Reset),
            kv("  main", num(n[0]), AMBER),
            kv("  subagent", num(n[1]), CYAN),
            kv("  workflow", num(n[2]), VIOLET),
            kv("rows", num(rows), Color::Reset),
            kv("indexed", format!("{} files · {}", num(self.files.iter().filter(|f| f.status == 0).count() as i64), size(bytes)), Color::Reset),
            kv("span (BKK)", if lo <= hi { format!("{} → {}", &bkk(Some(lo))[..10], &bkk(Some(hi))[..10]) } else { "—".into() }, Color::Reset),
            kv("repos", num(self.repos.len() as i64), Color::Reset),
        ];
        f.render_widget(Paragraph::new(lines).block(self.frame("totals".into())), stats);
        let w = types.width.saturating_sub(48) as usize;
        let tl: Vec<Line> = (0..6).map(|j| {
            let base = if j == 5 { by[4] } else { rows };
            let frac = if base > 0 { by[j] as f64 / base as f64 } else { 0.0 };
            Line::from(vec![
                Span::styled(format!("{:<15}", TYPE_LABEL[j]), Style::new().fg(TYPE_COLOR[j])),
                Span::styled(bar(frac, w.max(8)), Style::new().fg(TYPE_COLOR[j])),
                Span::raw(format!(" {:>10} ", num(by[j]))),
                Span::styled(format!("{:>5.1}%{}", frac * 100.0, if j == 5 { " of results" } else { "" }), Style::new().fg(DIM)),
            ])
        }).collect();
        f.render_widget(Paragraph::new(tl).block(self.frame("rows by type".into())), types);
        // sessions started per Bangkok month, as vertical bars
        let mut per: BTreeMap<String, [i64; 3]> = BTreeMap::new();
        for s in &self.sess {
            if let Some(t) = s.t0 {
                per.entry(bkk(Some(t))[..7].to_string()).or_default()[s.tier] += 1;
            }
        }
        let inner = self.frame("sessions per month (● main ◦ subagent · workflow)".into());
        let area = inner.inner(bottom);
        f.render_widget(inner, bottom);
        let months: Vec<(&String, &[i64; 3])> = per.iter().collect();
        if months.is_empty() || area.height < 3 {
            return;
        }
        let colw = ((area.width as usize) / months.len()).clamp(3, 9);
        let max = months.iter().map(|(_, v)| v.iter().sum::<i64>()).max().unwrap_or(1).max(1);
        let h = area.height.saturating_sub(2) as usize;
        let mut grid: Vec<Vec<Span>> = vec![vec![]; h];
        for (_, v) in &months {
            let tot: i64 = v.iter().sum();
            let cells = (tot as f64 / max as f64 * h as f64 * 8.0).round() as usize;
            // colour by the tier mix: the bar's lower part main, then subagent, then workflow
            let parts: Vec<usize> = v.iter().map(|x| (*x as f64 / tot.max(1) as f64 * cells as f64).round() as usize).collect();
            for (r, row) in grid.iter_mut().enumerate() {
                let level = (h - 1 - r) * 8;
                let fill = cells.saturating_sub(level).min(8);
                let ch = if fill == 0 { " " } else { ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"][fill - 1] };
                let col = if level < parts[0] { AMBER } else if level < parts[0] + parts[1] { CYAN } else { VIOLET };
                row.push(Span::styled(format!("{:^colw$}", ch.repeat(colw.saturating_sub(1))), Style::new().fg(col)));
            }
        }
        let mut lines: Vec<Line> = grid.into_iter().map(Line::from).collect();
        lines.push(Line::from(months.iter().map(|(m, _)| Span::styled(format!("{:^colw$}", if colw >= 8 { m.to_string() } else { m[5..].to_string() }), Style::new().fg(DIM))).collect::<Vec<_>>()));
        lines.push(Line::from(months.iter().map(|(_, v)| Span::raw(format!("{:^colw$}", num(v.iter().sum())))).collect::<Vec<_>>()));
        f.render_widget(Paragraph::new(lines), area);
    }

    fn table(&mut self, f: &mut Frame, a: Rect, title: String, head: Vec<&str>, widths: Vec<Constraint>, rows: Vec<Row>) {
        let n = rows.len();
        let t = Table::new(rows, widths)
            .header(Row::new(head.into_iter().map(|h| Cell::from(h).style(Style::new().fg(DIM).add_modifier(Modifier::BOLD)))).bottom_margin(0))
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol("▶ ")
            .block(self.frame(format!("{title} · {}", num(n as i64))));
        f.render_stateful_widget(t, a, &mut self.state);
    }

    fn repos_view(&mut self, f: &mut Frame, a: Rect) {
        let v = self.visible();
        let max = v.iter().map(|&k| self.repos[k].rows).max().unwrap_or(1).max(1);
        let rows: Vec<Row> = v.iter().map(|&k| {
            let r = &self.repos[k];
            Row::new(vec![
                Cell::from(r.name.clone()),
                Cell::from(format!("{:>7}", num(r.n.iter().sum()))),
                Cell::from(Line::from(vec![Span::styled(format!("{:>6}", num(r.n[0])), Style::new().fg(AMBER)),
                                           Span::styled(format!("{:>7}", num(r.n[1])), Style::new().fg(CYAN)),
                                           Span::styled(format!("{:>7}", num(r.n[2])), Style::new().fg(VIOLET))])),
                Cell::from(format!("{:>11}", num(r.rows))),
                Cell::from(Span::styled(bar(r.rows as f64 / max as f64, 16), Style::new().fg(CYAN))),
                Cell::from(Span::styled(format!("{:>8}", num(r.by[2])), Style::new().fg(PINK))),
                Cell::from(Span::styled(format!("{:>7}", num(r.by[5])), Style::new().fg(RED))),
                Cell::from(Span::styled(bkk(r.last).get(..10).unwrap_or("—").to_string(), Style::new().fg(DIM))),
            ])
        }).collect();
        self.table(f, a, "repos (enter: its sessions)".into(),
                   vec!["repo", "  sessions", "  main    sub     wf", "       rows", "", "thinking", " errors", "last (BKK)"],
                   vec![Constraint::Min(24), Constraint::Length(10), Constraint::Length(21), Constraint::Length(12), Constraint::Length(16),
                        Constraint::Length(9), Constraint::Length(8), Constraint::Length(11)], rows);
    }

    fn sessions_view(&mut self, f: &mut Frame, a: Rect) {
        let v = self.visible();
        let rows: Vec<Row> = v.iter().map(|&k| {
            let s = &self.sess[k];
            Row::new(vec![
                Cell::from(Span::styled(bkk(s.t0), Style::new().fg(DIM))),
                Cell::from(Span::styled(TIER_GLYPH[s.tier], Style::new().fg(TIER_COLOR[s.tier]))),
                Cell::from(Span::styled(s.sid.chars().take(36).collect::<String>(), Style::new().fg(TIER_COLOR[s.tier]))),
                Cell::from(s.repo.clone()),
                Cell::from(format!("{:>9}", num(s.rows))),
                Cell::from(Span::styled(format!("{:>7}", num(s.by[0] + s.by[1])), Style::new().fg(AMBER))),
                Cell::from(Span::styled(format!("{:>6}", num(s.by[2])), Style::new().fg(PINK))),
                Cell::from(Span::styled(format!("{:>7}", num(s.by[3])), Style::new().fg(CYAN))),
                Cell::from(Span::styled(format!("{:>6}", num(s.by[5])), Style::new().fg(RED))),
            ])
        }).collect();
        let title = match &self.repo_pick {
            Some(r) => format!("sessions of {r} (⌫ back)"),
            None => "sessions, newest first (enter: open)".into(),
        };
        self.table(f, a, title, vec!["started (BKK)", "", "sid", "repo", "     rows", "   text", " think", "  calls", "errors"],
                   vec![Constraint::Length(16), Constraint::Length(1), Constraint::Length(36), Constraint::Min(16), Constraint::Length(9),
                        Constraint::Length(7), Constraint::Length(6), Constraint::Length(7), Constraint::Length(6)], rows);
    }

    fn paths_view(&mut self, f: &mut Frame, a: Rect) {
        let v = self.visible();
        if let Some(d) = self.dir_pick {
            let rows: Vec<Row> = v.iter().map(|&k| {
                let x = &self.files[k];
                let (st, c) = STATUS[x.status];
                Row::new(vec![
                    Cell::from(Span::styled("●", Style::new().fg(c))),
                    Cell::from(x.name.clone()),
                    Cell::from(Span::styled(st, Style::new().fg(c))),
                    Cell::from(Span::styled(x.verdict.clone(), Style::new().fg(DIM))),
                    Cell::from(format!("{:>9}", if x.rows >= 0 { num(x.rows) } else { "—".into() })),
                    Cell::from(format!("{:>9}", size(x.size))),
                    Cell::from(Span::styled(bkk(Some(x.mtime)), Style::new().fg(DIM))),
                ])
            }).collect();
            let title = format!("{} (enter: copy path · ⌫ back)", self.tilde(&self.dirs[d].path));
            self.table(f, a, title, vec!["", "file (biggest first)", "status", "verdict", "     rows", "     size", "modified (BKK)"],
                       vec![Constraint::Length(1), Constraint::Min(30), Constraint::Length(8), Constraint::Length(18), Constraint::Length(9),
                            Constraint::Length(9), Constraint::Length(16)], rows);
            return;
        }
        // the folder column gets what the fixed columns leave; a long path keeps its end (the part that differs)
        let room = a.width.saturating_sub(6 + 12 + 6 + 6 + 6 + 10 + 9 + 7 + 4) as usize;
        let rows: Vec<Row> = v.iter().map(|&k| {
            let d = &self.dirs[k];
            let p = self.tilde(&d.path);
            let n_ch = p.chars().count();
            let p = if n_ch > room && room > 1 { format!("…{}", p.chars().skip(n_ch - room + 1).collect::<String>()) } else { p };
            let n = d.files.len() as f64;
            // a 12-cell bar split by status
            let mut spans = vec![];
            let mut used = 0;
            for (j, (_, c)) in STATUS.iter().enumerate() {
                let w = if j == 3 { 12 - used } else { ((d.by[j] as f64 / n) * 12.0).round() as usize };
                let w = w.min(12 - used);
                used += w;
                spans.push(Span::styled("█".repeat(w), Style::new().fg(*c)));
            }
            Row::new(vec![
                Cell::from(p),
                Cell::from(format!("{:>6}", num(d.files.len() as i64))),
                Cell::from(Line::from(spans)),
                Cell::from(Span::styled(format!("{:>6}", num(d.by[0])), Style::new().fg(CYAN))),
                Cell::from(Span::styled(format!("{:>6}", num(d.by[1])), Style::new().fg(RED))),
                Cell::from(Span::styled(format!("{:>6}", num(d.by[2] + d.by[3])), Style::new().fg(DIM))),
                Cell::from(format!("{:>10}", num(d.rows))),
                Cell::from(format!("{:>9}", size(d.bytes))),
            ])
        }).collect();
        self.table(f, a, "folders holding .jsonl (enter: its files)".into(),
                   vec!["folder", " files", "status", "  idx", " skip", "other", "      rows", "     size"],
                   vec![Constraint::Min(30), Constraint::Length(6), Constraint::Length(12), Constraint::Length(6), Constraint::Length(6),
                        Constraint::Length(6), Constraint::Length(10), Constraint::Length(9)], rows);
    }

    fn popup(&self, f: &mut Frame, k: usize) {
        let s = &self.sess[k];
        let a = f.area();
        let w = a.width.min(84);
        let h = 19u16.min(a.height);
        let r = Rect { x: a.x + (a.width - w) / 2, y: a.y + (a.height - h) / 2, width: w, height: h };
        let kv = |k: &str, v: String| Line::from(vec![Span::styled(format!("{k:<15}"), Style::new().fg(DIM)), Span::raw(v)]);
        let mut lines = vec![
            Line::from(Span::styled(format!("{} {}", TIER_GLYPH[s.tier], ["main session", "subagent", "workflow agent"][s.tier]),
                                    Style::new().fg(TIER_COLOR[s.tier]).bold())),
            kv("sid", s.sid.clone()),
            kv("repo", s.repo.clone()),
            kv("project", s.proj.clone()),
            kv("started", bkk(s.t0)),
            kv("last", bkk(s.t1)),
            kv("rows", num(s.rows)),
        ];
        for j in 0..6 {
            lines.push(Line::from(vec![Span::styled(format!("{:<15}", TYPE_LABEL[j]), Style::new().fg(TYPE_COLOR[j])), Span::raw(num(s.by[j]))]));
        }
        lines.push(kv("copies", format!("{} · {} largest", s.copies, size(s.bytes))));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![Span::styled(" c ", Style::new().fg(Color::Black).bg(CYAN)),
                                   Span::styled(if s.tier == 0 { " copy `claude --resume <sid>`" } else { " copy sid" }, Style::new().fg(DIM)),
                                   Span::styled("   any key closes", Style::new().fg(DIM))]));
        f.render_widget(Clear, r);
        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::bordered().border_type(BorderType::Double).border_style(Style::new().fg(PINK))
                .title(Span::styled(" session ", Style::new().fg(PINK).bold()))), r);
    }
}

fn run_app(term: &mut DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    loop {
        term.draw(|f| app.draw(f))?;
        // wake every 500 ms so a toast fades out on its own
        if !event::poll(Duration::from_millis(500))? {
            continue;
        }
        if let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
            && !app.key(k.code, k.modifiers) {
            return Ok(());
        }
    }
}

pub fn tui(out: &Path) {
    if !out.join("meta").join("sessions.parquet").exists() {
        eprintln!("tui: no {}/meta/sessions.parquet (run `relic-scan run` first)", out.display());
        std::process::exit(1);
    }
    let t0 = Instant::now();
    let mut app = App::load(out);
    let loaded = t0.elapsed();
    let r = ratatui::run(|term| run_app(term, &mut app));
    if let Err(e) = r {
        eprintln!("tui: {e}");
        std::process::exit(1);
    }
    eprintln!("tui: {} sessions, {} .jsonl loaded in {:.2}s", num(app.sess.len() as i64), num(app.files.len() as i64), loaded.as_secs_f64());
}
