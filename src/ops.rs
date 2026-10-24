//! What the relic chain is doing right now, for `relic-scan ops`, the page's Activity view and the terminal browser's tab 6.
//! One snapshot (JSON) from four places: the chain log (the stage lines each pass writes), the running processes (`ps`),
//! the hub's status (one GET, cached a few seconds) and the S3 job. Read only: nothing here changes a file.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::{Local, NaiveDateTime, TimeZone};
use serde_json::{json, Value};

/// A pass takes a fixed pause after its end before the next one starts (launchd and systemd: 10 minutes).
const PAUSE_S: i64 = 600;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Pass {
    pub machine: String,
    pub start: String,
    pub end: Option<String>,
    pub sync: Option<String>,
    /// seconds and one line of what it found
    pub scan: Option<(f64, String)>,
    pub extract: Option<(f64, String)>,
    pub push: Vec<f64>,
    pub embed: Option<(f64, String)>,
}

fn ts(l: &str) -> Option<&str> {
    l.get(..19).filter(|t| NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S").is_ok())
}

/// The number right after `marker` ("in 64s" -> 64.0).
fn after(l: &str, marker: &str) -> Option<f64> {
    let r = &l[l.find(marker)? + marker.len()..];
    let n: String = r.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    n.parse().ok()
}

/// The passes in a chain log, oldest first. Lines without a time (scan, extract, embed) belong to the pass still open.
pub fn parse_log(text: &str) -> Vec<Pass> {
    let mut v: Vec<Pass> = vec![];
    let mut scan_line = String::new();
    for l in text.lines() {
        let l = l.trim_end();
        if let (Some(t), true) = (ts(l), l.contains("chain start")) {
            let m = l.split('(').nth(1).and_then(|x| x.split(')').next()).unwrap_or("").to_string();
            v.push(Pass { machine: m, start: t.to_string(), ..Default::default() });
            scan_line.clear();
            continue;
        }
        let Some(p) = v.last_mut() else { continue };
        if l.starts_with("run: scan ") {
            scan_line = l.trim_start_matches("run: ").to_string();
        } else if l.starts_with("run: done in ") {
            p.scan = Some((after(l, "done in ").unwrap_or(0.0), scan_line.clone()));
        } else if l.contains("part(s) cut in ") {
            p.extract = Some((after(l, "cut in ").unwrap_or(0.0), l.split('·').skip(1).collect::<Vec<_>>().join("·").trim().to_string()));
        } else if l.starts_with("embed: ") {
            p.embed = Some((after(l, " in ").unwrap_or(0.0), l.trim_start_matches("embed: ").to_string()));
        } else if ts(l).is_some() && l.contains("relic5 push ") {
            p.push.push(l.rsplit(' ').next().and_then(|x| x.trim_end_matches('s').parse().ok()).unwrap_or(0.0));
        } else if ts(l).is_some() && l.contains("chain end") {
            p.end = ts(l).map(str::to_string);
        } else if ts(l).is_some() && l.contains("SYNC ") {
            p.sync = Some(l[20..].trim().to_string());
        }
    }
    v
}

fn epoch(t: &str) -> Option<i64> {
    NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S").ok().and_then(|n| Local.from_local_datetime(&n).single()).map(|d| d.timestamp())
}

fn median(mut v: Vec<f64>) -> Option<f64> {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v.get(v.len() / 2).copied()
}

/// Where the chain writes its log on this machine (RELIC_CHAIN_LOG overrides).
pub fn chain_log() -> PathBuf {
    if let Ok(p) = std::env::var("RELIC_CHAIN_LOG") {
        return PathBuf::from(p);
    }
    let h = PathBuf::from(std::env::var("HOME").unwrap_or_default());
    let mac = h.join("Library").join("Logs").join("relic-chain.log");
    if mac.exists() || cfg!(target_os = "macos") { mac } else { h.join(".local").join("state").join("relic-chain.log") }
}

/// The last part of the log (a pass is a few KB; 400 KB holds dozens).
fn tail(p: &Path) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut f) = std::fs::File::open(p) else { return String::new() };
    let n = f.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = f.seek(SeekFrom::Start(n.saturating_sub(400_000)));
    let mut b = vec![];
    let _ = f.read_to_end(&mut b);
    String::from_utf8_lossy(&b).to_string()
}

/// The chain as a JSON object: the pass in flight (or the last one), its stages with typical times, an ETA.
pub fn chain_view(passes: &[Pass], now: i64) -> Value {
    let Some(p) = passes.last() else { return json!({"state": "never"}) };
    let done: Vec<&Pass> = passes.iter().filter(|q| q.end.is_some() && q.scan.is_some()).rev().take(8).collect();
    let typ = |f: &dyn Fn(&Pass) -> Option<f64>| median(done.iter().filter_map(|q| f(q)).collect());
    let running = p.end.is_none();
    let checked = p.sync.is_some();
    let flags = [p.scan.is_some(), p.extract.is_some(), !p.push.is_empty(), p.end.is_some(), checked];
    let cur = flags.iter().position(|d| !d).unwrap_or(5);
    let names = ["scan", "extract", "push to the hub", "hub index + embed", "sync check"];
    let secs = [p.scan.as_ref().map(|s| s.0), p.extract.as_ref().map(|s| s.0), (!p.push.is_empty()).then(|| p.push.iter().sum()), None, None];
    let detail = [p.scan.as_ref().map(|s| s.1.clone()), p.extract.as_ref().map(|s| s.1.clone()), None, p.embed.as_ref().map(|e| e.1.clone()), p.sync.clone()];
    let typical = [typ(&|q| q.scan.as_ref().map(|s| s.0)), typ(&|q| q.extract.as_ref().map(|s| s.0)), typ(&|q| (!q.push.is_empty()).then(|| q.push.iter().sum())), None, None];
    let stages: Vec<Value> = (0..5).map(|i| json!({"name": names[i], "state": if flags[i] { "done" } else if i == cur { "running" } else { "waiting" },
        "secs": secs[i], "typical_s": typical[i], "detail": detail[i]})).collect();
    let start = epoch(&p.start).unwrap_or(now);
    let total = median(done.iter().filter_map(|q| Some((epoch(q.end.as_ref()?)? - epoch(&q.start)?) as f64)).collect());
    let elapsed = (now - start).max(0);
    let mut v = json!({"state": if running { "running" } else { "idle" }, "machine": p.machine, "started": p.start, "ended": p.end, "stages": stages, "sync": p.sync,
        "elapsed_s": if running { Some(elapsed) } else { None }, "typical_total_s": total,
        "eta_s": if running { total.map(|t| (t - elapsed as f64).max(0.0)) } else { None }});
    if !running {
        let end = p.end.as_deref().and_then(epoch).unwrap_or(now);
        v["took_s"] = json!(end - start);
        v["next_in_s"] = json!((end + PAUSE_S - now).max(0));
    }
    v
}

/// What a `ps` line is, when it is part of the relic chain.
pub fn what(cmd: &str) -> Option<&'static str> {
    [("relic-scan run", "scan"), ("relic-extract run", "extract"), ("relic5-push", "push"), ("relic5 index", "hub index (on white)"),
     ("relic-scan export", "S3 export"), ("relic5 embed", "embed"), ("local/bin/relic-chain", "chain")]
        .iter().find(|(k, _)| cmd.contains(k)).map(|(_, w)| *w)
}

pub fn processes() -> Vec<Value> {
    let Ok(o) = std::process::Command::new("ps").args(["-axo", "user=,etime=,pcpu=,rss=,command="]).output() else { return vec![] };
    String::from_utf8_lossy(&o.stdout).lines().filter_map(|l| {
        let mut f = l.split_whitespace();
        let (user, et, cpu, rss) = (f.next()?, f.next()?, f.next()?, f.next()?);
        let cmd = f.collect::<Vec<_>>().join(" ");
        let w = what(&cmd).filter(|_| !cmd.contains("ps -axo"))?;
        Some(json!({"what": w, "user": user, "elapsed": et, "cpu": cpu.parse::<f64>().unwrap_or(0.0), "rss_mb": rss.parse::<f64>().map(|k| (k / 1024.0).round()).unwrap_or(0.0)}))
    }).collect()
}

/// The parts of the hub's /api/status worth showing.
pub fn hub_view(j: &Value) -> Value {
    let m: Vec<Value> = j["freshness"]["machines"].as_object().into_iter().flatten().map(|(k, v)| json!({"machine": k, "push_age_s": v["push_age_s"], "sync": v["sync"]})).collect();
    json!({"ok": true, "machines": m, "running": j["jobs"]["running"], "warnings": j["warnings"], "load": j["resources"]["load"], "cpus": j["resources"]["cpus"]})
}

static HUB: Mutex<Option<(Instant, Value)>> = Mutex::new(None);

/// The hub's status, at most one GET every 4 seconds (a second reader gets the last answer). RELIC_OPS_HUB=off skips it.
pub fn hub() -> Value {
    let url = std::env::var("RELIC_OPS_HUB").or_else(|_| std::env::var("RELIC5_HUB_URL")).unwrap_or_else(|_| "http://white.follow-rankine.ts.net:8795".into());
    if url == "off" {
        return json!({"ok": false, "error": "off"});
    }
    let mut g = HUB.lock().unwrap();
    if let Some((at, v)) = g.as_ref().filter(|(at, _)| at.elapsed() < Duration::from_secs(4)) {
        let _ = at;
        return v.clone();
    }
    let a: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(3))).build().into();
    let v = match a.get(format!("{}/api/status", url.trim_end_matches('/'))).call().ok().and_then(|mut r| serde_json::from_str::<Value>(&r.body_mut().read_to_string().ok()?).ok()) {
        Some(j) => hub_view(&j),
        None => json!({"ok": false, "error": "the hub did not answer"}),
    };
    *g = Some((Instant::now(), v.clone()));
    v
}

pub fn snapshot(out: &Path, with_hub: bool) -> Value {
    let now = Local::now().timestamp();
    json!({"at": now, "log": chain_log().display().to_string(), "chain": chain_view(&parse_log(&tail(&chain_log())), now), "processes": processes(),
           "hub": if with_hub { hub() } else { json!({"ok": false, "error": "off"}) }, "s3": crate::s3job::status(out)})
}

mod text;
pub use text::render;
#[cfg(test)]
mod tests;
