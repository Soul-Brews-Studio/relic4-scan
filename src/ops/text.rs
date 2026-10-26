//! The snapshot as text lines, for `relic-scan ops` and `--watch`.

use serde_json::Value;

pub fn dur(s: i64) -> String {
    if s >= 3600 { format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60) } else { format!("{}:{:02}", s / 60, s % 60) }
}

pub fn render(v: &Value) -> Vec<String> {
    let c = &v["chain"];
    let mut o = vec![];
    match c["state"].as_str() {
        Some("running") => {
            let (el, eta) = (c["elapsed_s"].as_i64().unwrap_or(0), c["eta_s"].as_f64());
            o.push(format!("CHAIN {}  RUNNING  {} elapsed{}", c["machine"].as_str().unwrap_or(""), dur(el),
                match (c["typical_total_s"].as_f64(), eta) {
                    (Some(t), Some(e)) if e > 0.0 => format!(" · typical {} · about {} left", dur(t as i64), dur(e as i64)),
                    (Some(t), _) => format!(" · typical {}, over it now", dur(t as i64)),
                    _ => String::new() }));
        }
        Some("idle") => o.push(format!("CHAIN {}  idle  last pass {} took {} · {}", c["machine"].as_str().unwrap_or(""),
            &c["started"].as_str().unwrap_or("")[11..], dur(c["took_s"].as_i64().unwrap_or(0)),
            c["next_in_s"].as_i64().map_or("manual (no loop)".to_string(), |n| format!("next in about {}", dur(n))))),
        _ => o.push("CHAIN  no pass logged yet (".to_string() + v["log"].as_str().unwrap_or("") + ")"),
    }
    for s in c["stages"].as_array().into_iter().flatten() {
        let mark = match s["state"].as_str() { Some("done") => "✓", Some("running") => "●", _ => "·" };
        let t = s["secs"].as_f64().map(|x| format!("{x:.0} s")).unwrap_or_default();
        let ty = s["typical_s"].as_f64().map(|x| format!("(typical {x:.0} s)")).unwrap_or_default();
        o.push(format!("  {mark} {:<18} {:>7} {:<18} {}", s["name"].as_str().unwrap_or(""), t, ty, s["detail"].as_str().unwrap_or("")));
    }
    let p = v["processes"].as_array().cloned().unwrap_or_default();
    o.push(String::new());
    o.push(if p.is_empty() { "PROCESSES  none running".into() } else { "PROCESSES".into() });
    for x in &p {
        o.push(format!("  {:<22} {:<8} {:>9}  {:>5.0}% cpu  {:>6.0} MB", x["what"].as_str().unwrap_or(""), x["user"].as_str().unwrap_or(""), x["elapsed"].as_str().unwrap_or(""),
            x["cpu"].as_f64().unwrap_or(0.0), x["rss_mb"].as_f64().unwrap_or(0.0)));
    }
    let h = &v["hub"];
    o.push(String::new());
    if h["ok"].as_bool() == Some(true) {
        o.push(format!("HUB  load {:.1} of {} cpus · {} job(s) running", h["load"][0].as_f64().unwrap_or(0.0), h["cpus"], h["running"].as_array().map_or(0, |a| a.len())));
        for m in h["machines"].as_array().into_iter().flatten() {
            o.push(format!("  {:<20} last push {:>6} ago  {}", m["machine"].as_str().unwrap_or(""), dur(m["push_age_s"].as_i64().unwrap_or(0)),
                m["sync"].as_str().unwrap_or("").chars().take(60).collect::<String>()));
        }
        for w in h["warnings"].as_array().into_iter().flatten() {
            o.push(format!("  ! {}", w.as_str().unwrap_or("")));
        }
    } else {
        o.push(format!("HUB  not read ({})", h["error"].as_str().unwrap_or("")));
    }
    let s = &v["s3"];
    if s["running"].as_bool() == Some(true) {
        o.push(String::new());
        o.push(format!("S3  {} {}/{}", s["job"].as_str().unwrap_or(""), s["done"], s["total"]));
    }
    o
}

/// `render` for a terminal: the word HUB is a link to the hub, and the last line points at the page's Activity view.
/// `on` false leaves the escapes out (a pipe, NO_COLOR); the page line stays as plain text.
pub fn render_linked(v: &Value, base: &str, hub: &str, on: bool) -> Vec<String> {
    use crate::links::{link, view_url};
    let mut o = render(v);
    if hub != "off" && v["hub"]["ok"].as_bool() == Some(true)
        && let Some(l) = o.iter_mut().find(|l| l.starts_with("HUB")) {
            *l = format!("{}{}", link(on, hub, "HUB"), &l[3..]);
        }
    let page = view_url(base, "ops");
    o.push(String::new());
    o.push(format!("page  {}", link(on, &page, &page)));
    o
}
