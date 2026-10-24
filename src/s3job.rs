//! The S3 copy as a background job for the Export view: Check (compare with the store, send nothing) or Send (upload what is
//! missing), one at a time, with the objects as they are handled. The page polls GET /api/s3; the work is export_s3.rs.

use std::path::Path;
use std::sync::Mutex;

use crate::export_s3::{self, Item, Opts, Summary};

#[derive(Default)]
struct State {
    running: bool,
    job: &'static str,
    done: usize,
    total: usize,
    items: Vec<Item>,
    summary: Option<Summary>,
    error: Option<String>,
    at: String,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn opts(out: &Path, verify: bool) -> Result<Opts, String> {
    let c = crate::config::load(out).s3;
    if c.to.is_empty() {
        return Err("no S3 destination yet: set it in Settings".into());
    }
    Ok(Opts { out: out.to_path_buf(), to: c.to, endpoint: Some(c.endpoint).filter(|e| !e.is_empty()), region: Some(c.region).filter(|r| !r.is_empty()),
              dry_run: false, overwrite: false, verify, insecure_http: false })
}

pub fn status(out: &Path) -> serde_json::Value {
    let c = crate::config::load(out).s3;
    let (src, detail) = export_s3::creds_status();
    let g = STATE.lock().unwrap();
    let s = g.as_ref();
    serde_json::json!({
        "configured": !c.to.is_empty(), "to": c.to, "endpoint": c.endpoint, "region": c.region,
        "creds": {"source": src, "ok": src != "missing", "detail": detail},
        "running": s.is_some_and(|s| s.running), "job": s.map(|s| s.job), "done": s.map(|s| s.done), "total": s.map(|s| s.total), "at": s.map(|s| s.at.clone()),
        "items": s.map(|s| s.items.iter().map(|i| serde_json::json!({"key": i.key, "size": i.size, "action": i.action})).collect::<Vec<_>>()),
        "summary": s.and_then(|s| s.summary.as_ref()).map(|m| serde_json::json!({"uploaded": m.uploaded, "same": m.same, "refused": m.refused, "differs": m.differs, "outdated": m.outdated, "missing": m.missing, "bytes": m.bytes})),
        "error": s.and_then(|s| s.error.clone()),
    })
}

/// Start Check (verify = true) or Send. Err when one is running, or the destination or keys are not set.
pub fn start(out: &Path, verify: bool) -> Result<(), String> {
    let o = opts(out, verify)?;
    if export_s3::creds_status().0 == "missing" {
        return Err(export_s3::creds_status().1);
    }
    // sealed months plus months.json (and the manifest, when sending): for the progress bar
    let total = std::fs::read(out.join("meta").join("months.json")).ok().and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .map(|j| j["months"].as_array().map_or(0, |m| m.iter().filter(|e| e["sealed_at"].as_str().is_some_and(|s| !s.is_empty())).count()) + if verify { 1 } else { 2 }).unwrap_or(2);
    {
        let mut g = STATE.lock().unwrap();
        if g.as_ref().is_some_and(|s| s.running) {
            return Err("an S3 job is already running".into());
        }
        *g = Some(State { running: true, job: if verify { "check" } else { "send" }, total, at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(), ..Default::default() });
    }
    std::thread::spawn(move || {
        let r = export_s3::run_with(&o, &mut |i| {
            let mut g = STATE.lock().unwrap();
            if let Some(s) = g.as_mut() {
                s.done += 1;
                s.items.push(i.clone());
            }
        });
        let mut g = STATE.lock().unwrap();
        if let Some(s) = g.as_mut() {
            s.running = false;
            match r {
                Ok(m) => s.summary = Some(m),
                Err(e) => s.error = Some(e),
            }
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests;
