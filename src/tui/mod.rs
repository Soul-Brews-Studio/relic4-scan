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

mod draw;
mod keys;
mod load;

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
