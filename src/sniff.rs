//! Content sniffing: is this .jsonl an agent transcript, and whose? Reads the first 64 KB only.
//!
//! Two independent tests, learned from ground truth (50,188 beta@m5 files, 47,102 of which produced truth
//! rows; see LEARNED.md "fingerprints"):
//!   1. SIGNATURE of the first JSON objects: which agent writes lines shaped like this
//!        claude   sessionId + uuid/parentUuid, or a claude metadata type (permission-mode, queue-operation,
//!                 file-history-snapshot, summary+leafUuid, ai-title, last-prompt, agent-setting …)
//!        codex    session_meta / response_item / event_msg / turn_context with a payload
//!        omp/pi   {"type":"title","v":…} then {"type":"session","version":…,"id":…}
//!        thclaws  {"type":"header","cwd":…,"id":…}
//!      and the look-alikes that are NOT transcripts:
//!        claude history.jsonl {"display","project",…}, workflow journal.jsonl {"type":"launched|started"},
//!        omx logs (watcher_start, agent-turn-complete, tmux_hook …), channel access logs {"op","ts"}
//!   2. PROOF: the real parser (engine.rs) run on those lines yields at least one text/thinking/tool row
//! verdict: transcript (proof) · transcript-empty (signature, no message yet) · not-transcript

use std::fs::File;
use std::io::Read;

use serde_json::{Map, Value};

use crate::engine;

pub struct Sniff {
    pub agent: &'static str,
    pub verdict: &'static str,
    #[allow(dead_code)]
    pub why: String,
}

const CLAUDE_META: &[&str] = &["user", "assistant", "system", "summary", "permission-mode", "queue-operation", "mode",
    "agent-setting", "last-prompt", "ai-title", "custom-title", "cost-state", "progress", "attachment",
    "file-history-snapshot", "fork-context-ref"];
const CODEX_TYPES: &[&str] = &["session_meta", "response_item", "event_msg", "turn_context", "compacted"];
const RUNNER_TYPES: &[&str] = &["launched", "started", "finished", "failed", "phase", "agent-turn-complete", "watcher_start",
    "leader_nudge_tick", "tmux_hook", "auto_nudge_skipped", "visual_verdict_parse_no_match", "visual_verdict_persisted"];

fn signature(o: &Map<String, Value>, path: &str) -> Option<(&'static str, bool)> {
    // (agent, is_transcript_shape)
    let t = o.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if o.contains_key("display") && o.contains_key("project") {
        return Some(("claude", false)); // ~/.claude/history.jsonl: the prompt log, not a conversation
    }
    if RUNNER_TYPES.contains(&t) {
        return Some((if path.contains(".omx") { "omx" } else { "runner" }, false));
    }
    if CODEX_TYPES.contains(&t) && o.contains_key("payload") {
        return Some(("codex", true));
    }
    if (t == "title" && o.contains_key("v")) || (t == "session" && o.contains_key("version") && o.contains_key("id")) {
        return Some((if path.contains("/.pi/") { "pi" } else { "omp" }, true));
    }
    if t == "header" && o.contains_key("cwd") && o.contains_key("id") {
        return Some(("thclaws", true));
    }
    // workflow agents' first lines carry parentUuid/agentId but no sessionId (relic v2.1 lesson: subagent shapes vary)
    let claude_id = o.contains_key("sessionId") || o.contains_key("parentUuid") || o.contains_key("agentId");
    if (claude_id && (o.contains_key("uuid") || o.contains_key("parentUuid") || o.contains_key("message") || CLAUDE_META.contains(&t)))
        || t == "file-history-snapshot" || (t == "summary" && o.contains_key("leafUuid")) || t == "fork-context-ref" {
        return Some(("claude", true));
    }
    None
}

/// Light verdict from the path alone (no read): transcript-shaped names, and known non-transcript names.
pub fn by_path(path: &str) -> Sniff {
    let name = path.rsplit('/').next().unwrap_or(path);
    let not = name == "journal.jsonl" || name == "history.jsonl" || name.ends_with("access.jsonl") || name == "session_index.jsonl"
        || path.contains("/logs/") || name.starts_with('.') || path.contains("/state/usage");
    if not {
        return Sniff { agent: "-", verdict: "not-transcript", why: "name".into() };
    }
    if crate::scan::tier(path) != "other" || path.contains("/.omp/") || path.contains("/.pi/") || path.contains("/sessions/") {
        return Sniff { agent: crate::engine::engine_of(path), verdict: "transcript-shaped", why: "path".into() };
    }
    Sniff { agent: "-", verdict: "unknown", why: "path".into() }
}

/// Read until `lines` complete lines (however long a line is) or `cap` bytes.
fn head_lines(path: &str, lines: usize, cap: usize) -> std::io::Result<Vec<u8>> {
    let mut f = File::open(path)?;
    let mut buf = Vec::with_capacity(64 * 1024);
    let mut chunk = vec![0u8; 64 * 1024];
    loop {
        let n = f.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if memchr::memchr_iter(b'\n', &buf).count() >= lines || buf.len() >= cap {
            break;
        }
    }
    Ok(buf)
}

/// Full verdict: content signature + parser proof on the first complete lines.
pub fn sniff(path: &str) -> Sniff {
    let buf = match head_lines(path, 50, 8 << 20) {
        Ok(b) => b,
        Err(e) => return Sniff { agent: "-", verdict: "unreadable", why: e.to_string() },
    };
    let n = buf.len();
    if n == 0 {
        return Sniff { agent: "-", verdict: "empty", why: "0 bytes".into() };
    }
    let (mut agent, mut shape, mut why) = (None::<&'static str>, None::<bool>, String::new());
    let mut objects = 0;
    for line in buf.split_inclusive(|&b| b == b'\n').take(50) {
        let Some(Value::Object(o)) = engine::parse(line) else { continue };
        objects += 1;
        if agent.is_none() {
            if let Some((a, t)) = signature(&o, path) {
                agent = Some(a);
                shape = Some(t);
                why = format!("type={}", o.get("type").and_then(|v| v.as_str()).unwrap_or("-"));
            }
        }
        let x = engine::extract(&o);
        let rows = !x.text.is_empty() || !engine::thinking_of(line, &o).is_empty() || !engine::extract_tools(line, &o).is_empty();
        if rows && shape != Some(false) {
            let a = agent.unwrap_or_else(|| engine::engine_of(path));
            return Sniff { agent: a, verdict: "transcript", why: if why.is_empty() { "message rows".into() } else { why } };
        }
    }
    match (agent, shape) {
        (Some(a), Some(true)) => Sniff { agent: a, verdict: "transcript-empty", why },
        (Some(a), Some(false)) => Sniff { agent: a, verdict: "not-transcript", why },
        // a transcript-shaped path is never thrown away on a failed sniff: keep it, unverified
        _ if crate::scan::tier(path) != "other" => Sniff { agent: crate::engine::engine_of(path), verdict: "transcript-unverified",
                                                           why: "path shape, content unproven".into() },
        _ => Sniff { agent: "-", verdict: "not-transcript", why: if objects == 0 { "no JSON object".into() } else { "no agent signature".into() } },
    }
}
