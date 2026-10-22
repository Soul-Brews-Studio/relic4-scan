//! Port of jsonl-truth's line parser: src/relic4/engine.py (extract, extract_tools, thinking_of, redact,
//! ts_of, stamp, key) as driven by src/relic4/store.py parse_chunk. Every function mirrors its Python twin;
//! OUTPUT-CHAIN.md lists the rules and verify/understand.py checks them on real files.

use std::sync::OnceLock;

use chrono::{DateTime, Datelike, Timelike};
use regex::bytes::{Regex, RegexBuilder};
use serde_json::{Map, Value};

// ---------- gjson-like accessors (engine.py gstr / graw / gbool / get) ----------

fn dumps(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default() // compact, key order kept: orjson.dumps(...).decode()
}

/// gjson .String(): strings as is, missing/null as "", anything else as its JSON.
pub fn gstr(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => if *b { "true".into() } else { "false".into() },
        Some(x) => dumps(x),
    }
}

fn graw(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(x) => dumps(x),
    }
}

fn gbool(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => s == "true",
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
        _ => false,
    }
}

fn get<'a>(o: Option<&'a Value>, k: &str) -> Option<&'a Value> {
    o.and_then(|x| x.as_object()).and_then(|m| m.get(k))
}

/// Go strings.TrimSpace = Unicode White_Space, the same set as Rust's char::is_whitespace (not 0x1C-0x1F).
fn trim_go(s: &str) -> &str {
    s.trim_matches(char::is_whitespace)
}

/// Go's builder: a separator only once something non-empty was written.
fn join_nl<I: IntoIterator<Item = String>>(parts: I, sep: &str) -> String {
    let mut out = String::new();
    for p in parts {
        if !out.is_empty() {
            out.push_str(sep);
        }
        out.push_str(&p);
    }
    out
}

// ---------- time ----------

const MONTHS: [&str; 13] = ["", "January", "February", "March", "April", "May", "June", "July", "August", "September",
                            "October", "November", "December"];

fn parse_iso(t: &str) -> Option<i64> {
    // Python datetime.fromisoformat, tz-aware only (Go RFC3339 needs a zone)
    let d = DateTime::parse_from_rfc3339(t)
        .or_else(|_| DateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S%.f%z"))
        .or_else(|_| DateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S%z"))
        .or_else(|_| DateTime::parse_from_str(t, "%Y-%m-%dT%H:%M%z"))
        .ok()?;
    Some(d.timestamp_micros().div_euclid(1000)) // (d - EPOCH) // MS: floor to the millisecond
}

/// Epoch milliseconds, or None. Numbers: seconds (thclaws) or milliseconds.
pub fn ts_of(r: &Map<String, Value>) -> Option<i64> {
    match r.get("timestamp") {
        Some(Value::Number(n)) => {
            let f = n.as_f64()?;
            Some(if f > 1e12 { n.as_i64().unwrap_or(f as i64) } else { (f * 1000.0) as i64 })
        }
        Some(Value::String(t)) if t.chars().count() > 10 && t.chars().nth(10) == Some('T') => parse_iso(t),
        _ => None,
    }
}

pub struct Stamp {
    pub month: String, // "2026-10" or "undated"
    pub ts: String,    // UTC, 24 chars, Z
    pub ts_th: String, // +07:00
}

pub fn stamp(ms: Option<i64>) -> Stamp {
    let Some(ms) = ms else { return Stamp { month: "undated".into(), ts: String::new(), ts_th: String::new() } };
    let u = DateTime::from_timestamp_millis(ms).unwrap_or_default();
    let b = DateTime::from_timestamp_millis(ms + 7 * 3600 * 1000).unwrap_or_default();
    let f = format!(".{:03}", ms.rem_euclid(1000));
    let _ = MONTHS;
    Stamp {
        month: format!("{:04}-{:02}", b.year(), b.month()),
        ts: format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{f}Z", u.year(), u.month(), u.day(), u.hour(), u.minute(), u.second()),
        ts_th: format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{f}+07:00", b.year(), b.month(), b.day(), b.hour(), b.minute(), b.second()),
    }
}

// ---------- extract ----------

fn content_text(c: Option<&Value>) -> String {
    match c {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(a)) => join_nl(
            a.iter().filter(|it| it.is_object() && matches!(gstr(it.get("type")).as_str(), "text" | "input_text" | "output_text"))
                .map(|it| gstr(it.get("text"))),
            "\n",
        ),
        _ => String::new(),
    }
}

pub struct Extract {
    pub typ: String,
    pub role: String,
    pub sess: String,
    pub ts: Option<i64>,
    pub text: String,
}

/// Claude Code, Codex, omp/pi and thclaws shapes -> (type, role, session, ts_ms, text).
pub fn extract(r: &Map<String, Value>) -> Extract {
    let typ = gstr(r.get("type"));
    let ts = ts_of(r);
    let mut sess = gstr(r.get("sessionId"));
    let payload = r.get("payload");
    if typ == "session_meta" {
        sess = gstr(get(payload, "id"));
    } else if typ == "session" || typ == "header" {
        sess = gstr(r.get("id"));
    } else if typ == "summary" {
        let text = gstr(r.get("summary"));
        return Extract { typ, role: String::new(), sess, ts, text };
    }
    let (mut role, mut c) = (String::new(), None);
    if let Some(Value::Object(m)) = r.get("message") {
        role = gstr(m.get("role"));
        c = m.get("content");
    } else if typ == "response_item" && gstr(get(payload, "type")) == "message" {
        role = gstr(get(payload, "role"));
        c = get(payload, "content");
    } else if typ == "user" || typ == "assistant" {
        role = typ.clone();
        c = r.get("content");
    }
    if role == "developer" || role == "system" {
        return Extract { typ, role, sess, ts, text: String::new() };
    }
    let text = trim_go(&content_text(c)).to_string();
    Extract { typ, role, sess, ts, text }
}

const TOOL_HINTS: [&[u8]; 5] = [b"tool_use", b"tool_result", b"_call", b"toolCall", b"toolResult"];

fn block_text(c: Option<&Value>) -> String {
    match c {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(a)) => join_nl(
            a.iter().filter_map(|it| it.as_object()).filter(|o| o.contains_key("text")).map(|o| gstr(o.get("text"))),
            "\n",
        ),
        other => graw(other),
    }
}

pub struct Tool {
    pub kind: &'static str,
    pub call_id: String,
    pub name: String,
    pub body: String,
    pub err: bool,
}

/// Calls and results, joinable by call_id.
pub fn extract_tools(line: &[u8], r: &Map<String, Value>) -> Vec<Tool> {
    if !TOOL_HINTS.iter().any(|h| memchr::memmem::find(line, h).is_some()) {
        return vec![];
    }
    let mut out = vec![];
    if gstr(r.get("type")) == "response_item" {
        let p = r.get("payload");
        match gstr(get(p, "type")).as_str() {
            "function_call" => out.push(Tool { kind: "call", call_id: gstr(get(p, "call_id")), name: gstr(get(p, "name")),
                                               body: gstr(get(p, "arguments")), err: false }),
            "custom_tool_call" => out.push(Tool { kind: "call", call_id: gstr(get(p, "call_id")), name: gstr(get(p, "name")),
                                                  body: gstr(get(p, "input")), err: false }),
            "function_call_output" | "custom_tool_call_output" => {
                let o = get(p, "output");
                let body = match o {
                    Some(Value::Object(m)) => gstr(m.get("content")),
                    other => gstr(other),
                };
                out.push(Tool { kind: "result", call_id: gstr(get(p, "call_id")), name: String::new(), body, err: false });
            }
            _ => {}
        }
        return out;
    }
    let m = r.get("message");
    if let Some(Value::Object(mm)) = m {
        if gstr(mm.get("role")) == "toolResult" {
            return vec![Tool { kind: "result", call_id: gstr(mm.get("toolCallId")), name: gstr(mm.get("toolName")),
                               body: block_text(mm.get("content")), err: gbool(mm.get("isError")) }];
        }
    }
    let c = match m {
        Some(Value::Object(mm)) if mm.contains_key("content") => mm.get("content"),
        _ => r.get("content"),
    };
    let Some(Value::Array(items)) = c else { return out };
    for it in items {
        let o = it.as_object();
        match gstr(get(Some(it), "type")).as_str() {
            "tool_use" => out.push(Tool { kind: "call", call_id: gstr(get(Some(it), "id")), name: gstr(get(Some(it), "name")),
                                          body: graw(o.and_then(|o| o.get("input"))), err: false }),
            "toolCall" => out.push(Tool { kind: "call", call_id: gstr(get(Some(it), "id")), name: gstr(get(Some(it), "name")),
                                          body: graw(o.and_then(|o| o.get("arguments"))), err: false }),
            "tool_result" => out.push(Tool { kind: "result", call_id: gstr(get(Some(it), "tool_use_id")), name: String::new(),
                                             body: block_text(o.and_then(|o| o.get("content"))),
                                             err: gbool(o.and_then(|o| o.get("is_error"))) }),
            _ => {}
        }
    }
    out
}

/// Readable reasoning: Claude/omp thinking blocks, Codex reasoning summary (content is encrypted).
pub fn thinking_of(line: &[u8], r: &Map<String, Value>) -> String {
    if memchr::memmem::find(line, b"\"thinking\"").is_none() && memchr::memmem::find(line, b"\"reasoning\"").is_none() {
        return String::new();
    }
    let mut parts: Vec<String> = vec![];
    let mut add = |t: Option<&Value>| {
        let s = gstr(t);
        let s = trim_go(&s);
        if !s.is_empty() {
            parts.push(s.to_string());
        }
    };
    let payload = r.get("payload");
    if gstr(r.get("type")) == "response_item" && gstr(get(payload, "type")) == "reasoning" {
        for k in ["summary", "content"] {
            if let Some(Value::Array(v)) = get(payload, k) {
                for it in v {
                    add(get(Some(it), "text"));
                }
            }
        }
        return parts.join("\n\n");
    }
    let c = match r.get("message") {
        Some(Value::Object(mm)) if mm.contains_key("content") => mm.get("content"),
        _ => r.get("content"),
    };
    if let Some(Value::Array(items)) = c {
        for it in items {
            if gstr(get(Some(it), "type")) == "thinking" {
                add(it.get("thinking"));
            }
        }
    }
    parts.join("\n\n")
}

// ---------- line key + JSON parse ----------

/// blake2b(line, digest_size=8) read as a big-endian signed i64 (engine.py:364). The line keeps its "\n".
pub fn key(line: &[u8]) -> i64 {
    let h = blake2b_simd::Params::new().hash_length(8).hash(line);
    i64::from_be_bytes(h.as_bytes().try_into().unwrap())
}

/// engine.py LONE: a lone UTF-16 surrogate escape becomes � (what Go's gjson keeps), then parse again.
fn fix_lone_surrogates(line: &[u8]) -> Vec<u8> {
    let hex = |c: u8| c.is_ascii_hexdigit();
    let is_hi = |b: &[u8], i: usize| b.len() >= i + 6 && b[i] == b'\\' && b[i + 1] == b'u' && matches!(b[i + 2], b'd' | b'D')
        && matches!(b[i + 3], b'8' | b'9' | b'a' | b'b' | b'A' | b'B') && hex(b[i + 4]) && hex(b[i + 5]);
    let is_lo = |b: &[u8], i: usize| b.len() >= i + 6 && b[i] == b'\\' && b[i + 1] == b'u' && matches!(b[i + 2], b'd' | b'D')
        && (b[i + 3].is_ascii_hexdigit() && matches!(b[i + 3].to_ascii_lowercase(), b'c'..=b'f')) && hex(b[i + 4]) && hex(b[i + 5]);
    let mut out = Vec::with_capacity(line.len());
    let mut i = 0;
    while i < line.len() {
        let not_escaped = i == 0 || line[i - 1] != b'\\';
        if not_escaped && is_hi(line, i) && !is_lo(line, i + 6) {
            out.extend_from_slice(b"\\ufffd");
            i += 6;
            continue;
        }
        if not_escaped && is_lo(line, i) {
            let prev_hi_tail = i >= 3 && matches!(line[i - 3], b'8' | b'9' | b'a' | b'b' | b'A' | b'B') && hex(line[i - 2]) && hex(line[i - 1]);
            if !prev_hi_tail {
                out.extend_from_slice(b"\\ufffd");
                i += 6;
                continue;
            }
        }
        out.push(line[i]);
        i += 1;
    }
    out
}

/// The line as a JSON object, {} when it is JSON but not an object, None when it is not JSON at all.
pub fn parse(line: &[u8]) -> Option<Value> {
    serde_json::from_slice::<Value>(line).ok().or_else(|| serde_json::from_slice::<Value>(&fix_lone_surrogates(line)).ok())
}

// ---------- redaction (engine.py RULES, same order) ----------

struct Rule {
    kind: &'static str,
    hints: &'static [&'static str],
    rx: Regex,
    group: usize,
}

fn rules() -> &'static Vec<Rule> {
    static R: OnceLock<Vec<Rule>> = OnceLock::new();
    R.get_or_init(|| {
        // Python re.ASCII semantics: \b \s \w and (?i) are ASCII-only, so unicode(false) on bytes
        let b = |p: &str| RegexBuilder::new(p).unicode(false).build().expect("rule");
        vec![
            Rule { kind: "private-key", hints: &["PRIVATE KEY"], group: 0,
                   rx: b(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA |PGP )?PRIVATE KEY-----[\s\S]*?(?:-----END (?:RSA |EC |OPENSSH |DSA |PGP )?PRIVATE KEY-----|\z)") },
            Rule { kind: "anthropic-key", hints: &["sk-ant-"], group: 0, rx: b(r"sk-ant-[A-Za-z0-9_-]{30,}") },
            Rule { kind: "openai-key", hints: &["sk-"], group: 0, rx: b(r"\bsk-(?:proj-)?[A-Za-z0-9_-]{32,}") },
            Rule { kind: "aws-key", hints: &["AKIA", "ASIA"], group: 0, rx: b(r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b") },
            Rule { kind: "github-token", hints: &["gh", "github_pat_"], group: 0,
                   rx: b(r"\b(?:gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{60,})") },
            Rule { kind: "slack-token", hints: &["xox"], group: 0, rx: b(r"\bxox[abprs]-[A-Za-z0-9-]{10,}") },
            Rule { kind: "google-key", hints: &["AIza"], group: 0, rx: b(r"\bAIza[0-9A-Za-z_-]{35}") },
            Rule { kind: "jwt", hints: &["eyJ"], group: 0, rx: b(r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}") },
            Rule { kind: "bearer", hints: &["earer"], group: 1, rx: b(r"(?i)\bbearer\s+([A-Za-z0-9._~+/-]{24,})") },
            Rule { kind: "password", hints: &["ass", "ecret", "oken", "pi_key", "pikey", "API_KEY", "APIKEY"], group: 1,
                   rx: b(r#"(?i)\b(?:password|passwd|secret|api_?key|token)\s*[=:]\s*['"]?([A-Za-z0-9!#%&*+,./:;=?@^_|~-]{8,})"#) },
        ]
    })
}

/// A password/token value has letters AND digits and entropy >= 2.5 bits/char.
fn secret_like(v: &str) -> bool {
    if v.starts_with(['$', '<', '{', '(']) || v.contains('…') {
        return false;
    }
    if !v.chars().any(char::is_alphabetic) || !v.chars().any(|c| c.is_numeric()) {
        return false;
    }
    let chars: Vec<char> = v.chars().collect();
    let n = chars.len() as f64;
    let mut counts = std::collections::HashMap::new();
    for c in &chars {
        *counts.entry(*c).or_insert(0usize) += 1;
    }
    let e: f64 = -counts.values().map(|&k| { let p = k as f64 / n; p * p.log2() }).sum::<f64>();
    e >= 2.5
}

/// (clean text, sorted kinds joined by ',' or "").
pub fn redact(input: &str) -> (String, String) {
    let mut s = input.to_string();
    let mut kinds: Vec<&str> = vec![];
    for rule in rules() {
        if !rule.hints.iter().any(|h| s.contains(h)) {
            continue;
        }
        let bytes = s.as_bytes();
        let mut out = String::new();
        let mut last = 0usize;
        let mut hit = false;
        for caps in rule.rx.captures_iter(bytes) {
            let Some(m) = caps.get(rule.group) else { continue };
            let (a, z) = (m.start(), m.end());
            if a < last || bytes.get(a) == Some(&b'[') {
                continue; // already redacted by an earlier rule
            }
            let v = &s[a..z];
            if rule.kind == "password" && !secret_like(v) {
                continue;
            }
            out.push_str(&s[last..a]);
            let head: String = v.chars().take(4).collect();
            out.push_str(&format!("[{}:{}…{}]", rule.kind, head, v.chars().count()));
            last = z;
            hit = true;
        }
        if hit && last > 0 {
            out.push_str(&s[last..]);
            s = out;
            kinds.push(rule.kind);
        }
    }
    kinds.sort();
    (s, kinds.join(","))
}

/// store.engine_of: the engine from the file path.
pub fn engine_of(path: &str) -> &'static str {
    let p = path.to_lowercase();
    if p.contains(".thclaws") {
        "thclaws"
    } else if p.contains(".omx") {
        "omx"
    } else if p.contains("/.pi/") {
        "pi"
    } else if p.contains(".omp") {
        "omp"
    } else if p.contains("codex") || p.contains("rollout-") {
        "codex"
    } else {
        "claude"
    }
}
