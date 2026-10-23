//! Terminal styling for the CLI: colour only when stderr is a terminal (and NO_COLOR is unset), plain text otherwise,
//! so logs and `herdr pane read` stay clean.

use std::io::IsTerminal;
use std::sync::OnceLock;

pub fn color() -> bool {
    static C: OnceLock<bool> = OnceLock::new();
    *C.get_or_init(|| std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none())
}

fn paint(code: &str, s: &str) -> String {
    if color() { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_string() }
}

pub fn cyan(s: &str) -> String {
    paint("38;2;0;214;255", s)
}
pub fn pink(s: &str) -> String {
    paint("38;2;255;60;172", s)
}
pub fn dim(s: &str) -> String {
    paint("38;2;128;136;156", s)
}
pub fn bold(s: &str) -> String {
    paint("1", s)
}

/// The terminal's width in columns (100 when stderr is not a terminal).
pub fn width() -> usize {
    if !std::io::stderr().is_terminal() {
        return 100;
    }
    ratatui::crossterm::terminal::size().map(|(w, _)| w as usize).unwrap_or(100).max(20)
}

/// One live status line, redrawn in place with `\r`: `head` (may be coloured, `head_cols` wide) then `rest`, cut to the
/// terminal's width. A line wider than the terminal wraps, and `\r` then returns only to the start of its last row, so
/// every redraw would leave a copy behind; on a terminal the row is also cleared first.
pub fn live(head: &str, head_cols: usize, rest: &str) {
    let room = width().saturating_sub(head_cols + 1);
    let cut: String = rest.chars().take(room).collect();
    let clear = if std::io::stderr().is_terminal() { "\x1b[2K" } else { "" };
    eprint!("\r{clear}{head}{cut}");
}

/// ━━━━━━━━╸──────── for `frac` of `w` cells
pub fn bar(frac: f64, w: usize) -> String {
    let done = (frac.clamp(0.0, 1.0) * w as f64) as usize;
    let head = if done < w { "╸" } else { "" };
    let rest = w.saturating_sub(done + head.chars().count());
    format!("{}{}", cyan(&format!("{}{head}", "━".repeat(done))), dim(&"─".repeat(rest)))
}

pub fn logo() -> String {
    format!("{} {}{}{} {}", pink("▓▒░"), pink(&bold("RELIC")), dim("·"), cyan(&bold("SCAN")), cyan("░▒▓"))
}

/// A rounded box around lines (widths counted in chars, colour codes excluded).
pub fn card(title: &str, lines: &[String]) -> String {
    let vis = |s: &str| {
        let mut n = 0;
        let mut esc = false;
        for c in s.chars() {
            if c == '\x1b' {
                esc = true;
            } else if esc {
                esc = c != 'm';
            } else {
                n += 1;
            }
        }
        n
    };
    let w = lines.iter().map(|l| vis(l)).max().unwrap_or(0).max(vis(title) + 4);
    let mut out = format!("{}{}{}\n", dim("╭─ "), title, dim(&format!(" {}╮", "─".repeat(w - vis(title) - 1))));
    for l in lines {
        out += &format!("{} {l}{} {}\n", dim("│"), " ".repeat(w - vis(l)), dim("│"));
    }
    out + &dim(&format!("╰{}╯", "─".repeat(w + 2)))
}
