//! Drawing: header, footer, the overview and the three lists, and the popup.

use super::*;

impl App {
    pub(super) fn draw(&mut self, f: &mut Frame) {
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

    pub(super) fn header(&self, f: &mut Frame, a: Rect) {
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

    pub(super) fn footer(&self, f: &mut Frame, a: Rect) {
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

    pub(super) fn frame<'a>(&self, title: String) -> Block<'a> {
        Block::bordered().border_type(BorderType::Rounded).border_style(Style::new().fg(DIM))
            .title(Line::from(Span::styled(format!(" {title} "), Style::new().fg(CYAN).bold())))
    }

    pub(super) fn overview(&self, f: &mut Frame, a: Rect) {
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

    pub(super) fn table(&mut self, f: &mut Frame, a: Rect, title: String, head: Vec<&str>, widths: Vec<Constraint>, rows: Vec<Row>) {
        let n = rows.len();
        let t = Table::new(rows, widths)
            .header(Row::new(head.into_iter().map(|h| Cell::from(h).style(Style::new().fg(DIM).add_modifier(Modifier::BOLD)))).bottom_margin(0))
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol("▶ ")
            .block(self.frame(format!("{title} · {}", num(n as i64))));
        f.render_stateful_widget(t, a, &mut self.state);
    }

    pub(super) fn repos_view(&mut self, f: &mut Frame, a: Rect) {
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

    pub(super) fn sessions_view(&mut self, f: &mut Frame, a: Rect) {
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

    pub(super) fn paths_view(&mut self, f: &mut Frame, a: Rect) {
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

    pub(super) fn popup(&self, f: &mut Frame, k: usize) {
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
