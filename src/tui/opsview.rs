//! 6 ops: what the relic chain is doing right now (the same snapshot as `relic-scan ops`), redrawn from a copy that is at
//! most 2 seconds old, so a 500 ms redraw does not run `ps` each time.

use super::*;

impl App {
    pub(super) fn ops_view(&mut self, f: &mut Frame, a: Rect) {
        if self.ops.as_ref().is_none_or(|(at, _)| at.elapsed() > Duration::from_secs(2)) {
            self.ops = Some((Instant::now(), crate::ops::snapshot(Path::new(&self.out), true)));
        }
        let lines: Vec<Line> = crate::ops::render(&self.ops.as_ref().unwrap().1).into_iter().map(|l| {
            let c = if l.starts_with("CHAIN") || l.starts_with("PROCESSES") || l.starts_with("HUB") || l.starts_with("S3") { Style::new().fg(CYAN).bold() }
                else if l.starts_with("  ✓") { Style::new().fg(CYAN) } else if l.starts_with("  ●") { Style::new().fg(PINK).bold() }
                else if l.starts_with("  !") { Style::new().fg(AMBER) } else if l.starts_with("  ·") { Style::new().fg(DIM) } else { Style::new() };
            Line::from(Span::styled(l, c))
        }).collect();
        f.render_widget(Paragraph::new(lines).block(self.frame("operations · refreshes every 2 s".into())), a);
    }
}
