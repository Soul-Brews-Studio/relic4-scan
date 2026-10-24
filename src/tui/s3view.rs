//! 5 s3: the S3 copy of the sealed months. Same job as the page's Export card (s3job): c checks against the store and
//! sends nothing, s sends what is missing and never replaces a month that differs.

use super::*;

/// (words, colour) for an item's action, as the page's chips say it.
pub(super) fn act(a: &str) -> (&'static str, Color) {
    match a {
        "same" => ("identical in S3", CYAN),
        "upload" => ("sent", CYAN),
        "would send" => ("not in S3 yet", AMBER),
        "differs" => ("differs from S3", RED),
        "missing" => ("not in S3", AMBER),
        "outdated" => ("older in S3: send updates it", AMBER),
        "refused" => ("sealed again: not replaced", RED),
        _ => ("?", DIM),
    }
}

impl App {
    pub(super) fn s3_view(&self, f: &mut Frame, a: Rect) {
        let j = crate::s3job::status(Path::new(&self.out));
        let st = |k: &str| j[k].as_str().unwrap_or("").to_string();
        let kv = |k: &str, v: String, c: Color| Line::from(vec![Span::styled(format!("{k:<12}"), Style::new().fg(DIM)), Span::styled(v, Style::new().fg(c).bold())]);
        let mut top = if j["configured"].as_bool() == Some(true) {
            let mut v = vec![kv("to", st("to"), Color::Reset)];
            if !st("endpoint").is_empty() {
                v.push(kv("endpoint", st("endpoint"), Color::Reset));
            }
            v
        } else {
            vec![kv("to", "not set: put it in Settings (page or app)".into(), AMBER)]
        };
        let ok = j["creds"]["ok"].as_bool() == Some(true);
        top.push(kv("keys", j["creds"]["detail"].as_str().unwrap_or("").to_string(), if ok { Color::Reset } else { RED }));
        let (done, total) = (j["done"].as_u64().unwrap_or(0), j["total"].as_u64().unwrap_or(0));
        if j["running"].as_bool() == Some(true) {
            top.push(kv(&st("job"), format!("{done}/{total}  {}", bar(done as f64 / total.max(1) as f64, 24)), PINK));
        } else if let Some(e) = j["error"].as_str() {
            top.push(kv("error", e.to_string(), RED));
        } else if let Some(m) = j["summary"].as_object() {
            let n = |k: &str| m[k].as_u64().unwrap_or(0);
            let bad = n("differs") + n("refused");
            top.push(kv(&st("job"), format!("{} same · {} sent · {} missing · {} outdated · {} differ", n("same"), n("uploaded"), n("missing"), n("outdated"), bad),
                        if bad > 0 { RED } else { CYAN }));
        } else {
            top.push(kv("", "press c to check against S3, s to send what is missing".into(), DIM));
        }
        let [h, rest] = Layout::vertical([Constraint::Length(top.len() as u16 + 2), Constraint::Min(3)]).areas(a);
        f.render_widget(Paragraph::new(top).block(self.frame("S3 copy of the sealed months".into())).wrap(Wrap { trim: false }), h);
        let items = j["items"].as_array().cloned().unwrap_or_default();
        let rows: Vec<Row> = items.iter().map(|i| {
            let (t, c) = act(i["action"].as_str().unwrap_or(""));
            Row::new(vec![
                Cell::from(i["key"].as_str().unwrap_or("").split('/').skip(1).collect::<Vec<_>>().join("/")),
                Cell::from(size(i["size"].as_i64().unwrap_or(0))),
                Cell::from(Span::styled(t, Style::new().fg(c))),
            ])
        }).collect();
        let t = Table::new(rows, [Constraint::Min(24), Constraint::Length(10), Constraint::Min(24)]).block(self.frame(format!("{} objects", items.len())));
        f.render_widget(t, rest);
    }

    /// c = check, s = send; the reason a job did not start goes to the toast.
    pub(super) fn s3_start(&mut self, verify: bool) {
        match crate::s3job::start(Path::new(&self.out), verify) {
            Ok(()) => self.toast = Some((if verify { "checking against S3…" } else { "sending to S3…" }.into(), Instant::now())),
            Err(e) => self.toast = Some((e, Instant::now())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::act;

    #[test]
    fn every_action_has_words() {
        for a in ["same", "upload", "would send", "differs", "missing", "outdated", "refused"] {
            assert_ne!(act(a).0, "?", "{a}");
        }
        assert_eq!(act("nope").0, "?");
    }
}
