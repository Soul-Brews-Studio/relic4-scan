# ภาคผนวก: โค้ดที่เลือก

ไฟล์ด้านล่างพิมพ์ตามจริงทุกบรรทัด จาก commit `7ae3299` ของ Soul-Brews-Studio/relic4-scan ไม่ย่อ ไม่แก้ให้สวย เลือกเฉพาะส่วนที่บทต่าง ๆ ชี้ไปถึง

> relic-scan เป็นโอเพนซอร์ส (AGPL v3) โค้ดทั้งชุดอยู่ที่ https://github.com/Soul-Brews-Studio/relic4-scan

## src/engine.rs

how one transcript line becomes rows: the Claude, Codex and omp readers, redaction, usage · 526 บรรทัด

```rust
//! Port of jsonl-truth's line parser: src/relic4/engine.py (extract, extract_tools, thinking_of, redact,
//! ts_of, stamp, key) as driven by src/relic4/store.py parse_chunk. Every function mirrors its Python twin;
//! docs/OUTPUT-CHAIN.md lists the rules and verify/understand.py checks them on real files.

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
    // developer/system text is not conversation; an omp tool result (role "toolResult") is already a tool row
    // (extract_tools), so it is not a text row as well
    if role == "developer" || role == "system" || role == "toolResult" {
        return Extract { typ, role, sess, ts, text: String::new() };
    }
    let text = trim_go(&content_text(c)).to_string();
    Extract { typ, role, sess, ts, text }
}

/// Which model answered, and the API response a line belongs to. Claude Code and omp/pi write them on the assistant
/// message: Claude `model`, `id`, `usage.output_tokens` (repeated on every content-block line of one response); omp
/// `model`, `responseId`, `usage.output`. Codex writes no model on its messages: see turn_model.
#[derive(Default, Debug, PartialEq)]
pub struct Usage {
    pub model: String,
    pub msg_id: String,
    pub out_tokens: Option<i64>,
    /// input tokens and the prompt cache (Claude `input_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens`;
    /// omp `input`, `cacheRead`, `cacheWrite`)
    pub in_tokens: Option<i64>,
    pub cache_read: Option<i64>,
    pub cache_write: Option<i64>,
}

pub fn usage_of(r: &Map<String, Value>) -> Usage {
    let Some(Value::Object(m)) = r.get("message") else { return Usage::default() };
    if gstr(m.get("role")) != "assistant" {
        return Usage::default();
    }
    let id = gstr(m.get("id"));
    let u = m.get("usage");
    Usage {
        model: gstr(m.get("model")),
        msg_id: if id.is_empty() { gstr(m.get("responseId")) } else { id },
        out_tokens: get(u, "output_tokens").or_else(|| get(u, "output")).and_then(Value::as_i64),
        in_tokens: get(u, "input_tokens").or_else(|| get(u, "input")).and_then(Value::as_i64),
        cache_read: get(u, "cache_read_input_tokens").or_else(|| get(u, "cacheRead")).and_then(Value::as_i64),
        cache_write: get(u, "cache_creation_input_tokens").or_else(|| get(u, "cacheWrite")).and_then(Value::as_i64),
    }
}

/// What a line says about its session as a whole: where it ran, what it is called, the branch, the Claude version.
/// Claude writes `cwd`, `gitBranch` and `version` on its lines and the name as a `custom-title` (or `ai-title`) line;
/// Codex writes `cwd` in the payload of its session_meta and turn_context lines.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct SessMeta {
    pub cwd: String,
    pub title: String,
    pub branch: String,
    pub version: String,
    /// a message that came in through a channel plugin (Discord, Telegram, a fleet bridge): the plugin, the room and
    /// the sender of the first such message of the session
    pub via: String,
    pub chat: String,
    pub from_user: String,
}

pub fn sess_meta_of(r: &Map<String, Value>) -> SessMeta {
    let typ = gstr(r.get("type"));
    let mut cwd = gstr(r.get("cwd"));
    if cwd.is_empty() && matches!(typ.as_str(), "session_meta" | "turn_context") {
        cwd = gstr(get(r.get("payload"), "cwd"));
    }
    let title = match typ.as_str() {
        "custom-title" => gstr(r.get("customTitle")),
        "ai-title" => gstr(r.get("aiTitle")),
        _ => String::new(),
    };
    SessMeta { cwd, title, branch: gstr(r.get("gitBranch")), version: gstr(r.get("version")), ..Default::default() }
}

/// The channel a user message came in by: `<channel source="plugin:discord:discord" chat_id="…" user="…" …>`.
pub fn channel_of(text: &str) -> SessMeta {
    let Some(i) = text.find("<channel source=\"") else { return SessMeta::default() };
    let tag = &text[i..text[i..].find('>').map(|e| i + e).unwrap_or(text.len())];
    let attr = |k: &str| -> String {
        let pat = format!(" {k}=\"");
        tag.find(&pat).map(|p| &tag[p + pat.len()..]).and_then(|s| s.split('"').next()).unwrap_or("").to_string()
    };
    let via = tag["<channel source=\"".len()..].split('"').next().unwrap_or("").to_string();
    SessMeta { via, chat: attr("chat_id"), from_user: attr("user"), ..Default::default() }
}

impl SessMeta {
    /// Fold a later line in: the first cwd stays (where the session began); the others follow the newest line.
    pub fn take(&mut self, x: SessMeta) {
        if self.cwd.is_empty() {
            self.cwd = x.cwd;
        }
        // the first channel message names the session's channel
        if self.via.is_empty() {
            (self.via, self.chat, self.from_user) = (x.via, x.chat, x.from_user);
        }
        for (a, b) in [(&mut self.title, x.title), (&mut self.branch, x.branch), (&mut self.version, x.version)] {
            if !b.is_empty() {
                *a = b;
            }
        }
    }
}

/// Codex names the model once per turn, in a `turn_context` line; it holds for the turn's rows after it.
pub fn turn_model(r: &Map<String, Value>) -> Option<String> {
    (gstr(r.get("type")) == "turn_context").then(|| gstr(get(r.get("payload"), "model"))).filter(|m| !m.is_empty())
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
/// None: the line has no thinking. Some(""): it thought, but the text was not saved (newer Claude Code writes a thinking
/// block as a signature only; a Codex reasoning item may have no summary), so the row is kept with empty text.
pub fn thinking_of(line: &[u8], r: &Map<String, Value>) -> Option<String> {
    if memchr::memmem::find(line, b"\"thinking\"").is_none() && memchr::memmem::find(line, b"\"reasoning\"").is_none() {
        return None;
    }
    let mut parts: Vec<String> = vec![];
    let mut blocks = 0;
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
        return Some(parts.join("\n\n"));
    }
    let c = match r.get("message") {
        Some(Value::Object(mm)) if mm.contains_key("content") => mm.get("content"),
        _ => r.get("content"),
    };
    if let Some(Value::Array(items)) = c {
        for it in items {
            if gstr(get(Some(it), "type")) == "thinking" {
                blocks += 1;
                add(it.get("thinking"));
            }
        }
    }
    (blocks > 0).then(|| parts.join("\n\n"))
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
```

## src/meta/rules.rs

which file is which session: tier_of, cat_of, proj_of and the path rules · 136 บรรทัด

```rust
//! The path rules (sql.py MACROS, one for one): tier, engine, repo (cat), project, parent and home of a file.

use super::*;


pub(super) const UUID: &str = "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";

pub(super) struct Rx {
    pub(super) wf: Regex,
    pub(super) sub: Regex,
    pub(super) main_uuid: Regex,
    pub(super) main_rollout: Regex,
    pub(super) main_omp: Regex,
    pub(super) sub_omp: Regex,
    pub(super) parent_omp: Regex,
    pub(super) main_other: Regex,
    pub(super) history: Regex,
    pub(super) proj: Regex,
    pub(super) parent: Regex,
    pub(super) home: Regex,
    pub(super) agent_sid: Regex,
    pub(super) uuid_file: Regex,
    pub(super) gh: Regex,
    pub(super) gh_org: Regex,
    pub(super) gh_wt: Regex,
    pub(super) dashes: Regex,
    pub(super) wts: Regex,
    pub(super) wts_wt: Regex,
}

pub(super) fn rx() -> &'static Rx {
    static R: OnceLock<Rx> = OnceLock::new();
    R.get_or_init(|| {
        let r = |p: &str| Regex::new(p).expect("meta rule");
        Rx {
            wf: r(r"/subagents/workflows/.*agent-[^/]+\.jsonl$"),
            sub: r(r"/subagents/agent-[^/]+\.jsonl$"),
            main_uuid: r(&format!(r"/{UUID}\.jsonl$")),
            main_rollout: r(&format!(r"rollout-.*{UUID}\.jsonl(\.zst)?$")),
            main_omp: r(r"/\.(omp|pi)/.*_[0-9a-f-]+\.jsonl$"),
            // omp keeps a subagent's transcript inside its parent session's folder, named for its task:
            //   sessions/<project>/<time>_<uuid>/<Task>.jsonl  (the parent is sessions/<project>/<time>_<uuid>.jsonl)
            sub_omp: r(&format!(r"/\.(omp|pi)/.*_{UUID}/[^/]+\.jsonl$")),
            parent_omp: r(&format!(r"/\.(?:omp|pi)/.*_({UUID})/[^/]+\.jsonl$")),
            main_other: r(r"/\.(omx|thclaws|gemini)/"),
            history: r(r"history\.jsonl$"),
            proj: r(&format!(r"/([^/]+)/(?:{UUID}\.jsonl|{UUID}/subagents/)")),
            parent: r(&format!(r"/({UUID})/subagents/")),
            home: r(r"^(.*?/\.?(?:claude|codex|omp|omx|pi|thclaws|gemini)[^/]*)/"),
            agent_sid: r(r"^agent-(.*)\.jsonl$"),
            uuid_file: r(&format!(r"^({UUID})\.jsonl$")),
            gh: r(r".*github-com-(.*)$"),
            gh_org: r(r"^(soul-brews-studio|laris-co|dustboy-pm25|floodboy-cm|nazt|nat-weerawan)-"),
            gh_wt: r(r"(-worktree-|-wt-|--claude-worktrees-).*$"),
            dashes: r(r"^-+|-+$"),
            wts: r(r".*-worktrees-(.*)$"),
            wts_wt: r(r"(-worktree-|-wt-).*$"),
        }
    })
}

/// DuckDB regexp_extract: group `g` of the first match, "" when there is none.
pub(super) fn extract(re: &Regex, s: &str, g: usize) -> String {
    re.captures(s).and_then(|c| c.get(g)).map(|m| m.as_str().to_string()).unwrap_or_default()
}

pub fn tier_of(path: &str) -> &'static str {
    let r = rx();
    if r.wf.is_match(path) {
        "workflow-agent"
    } else if r.sub.is_match(path) || r.sub_omp.is_match(path) {
        "subagent"
    } else if r.main_uuid.is_match(path) || r.main_rollout.is_match(path) || r.main_omp.is_match(path)
        || (r.main_other.is_match(path) && !r.history.is_match(path)) {
        "main"
    } else {
        "other"
    }
}

pub fn engine_of(path: &str) -> &'static str {
    if path.contains("rollout-") || path.contains("/.codex") {
        "codex"
    } else if path.contains("/.omp/") {
        "omp"
    } else if path.contains("/.omx/") {
        "omx"
    } else if path.contains("/.thclaws/") {
        "thclaws"
    } else if path.contains("/.gemini/") {
        "gemini"
    } else if path.contains("/.pi/") {
        "pi"
    } else {
        "claude"
    }
}

pub(super) fn base_of(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

pub(super) fn proj_of(path: &str) -> Option<String> {
    Some(extract(&rx().proj, path, 1)).filter(|s| !s.is_empty())
}

pub(super) fn parent_of(path: &str) -> Option<String> {
    Some(extract(&rx().parent, path, 1)).filter(|s| !s.is_empty())
        .or_else(|| Some(extract(&rx().parent_omp, path, 1)).filter(|s| !s.is_empty()))
}

pub(super) fn home_of(path: &str) -> String {
    extract(&rx().home, path, 1)
}

pub fn cat_of(engine: &str, proj: Option<&str>) -> String {
    let r = rx();
    if engine != "claude" {
        return format!("{engine} (no cwd in path)");
    }
    let Some(p) = proj.filter(|p| !p.is_empty()) else { return "(no project dir)".into() };
    let lp = p.to_lowercase();
    if lp.contains("oracle-probe-cwd") {
        "probe (automated)".into()
    } else if lp.contains("github-com-") {
        let x = extract(&r.gh, &lp, 1);
        let x = r.gh_org.replace(&x, "");
        let x = r.gh_wt.replace(&x, "");
        r.dashes.replace_all(&x, "").into_owned()
    } else if lp.contains("-worktrees-") {
        let x = extract(&r.wts, &lp, 1);
        r.wts_wt.replace(&x, "").into_owned()
    } else {
        "home / other".into()
    }
}
```

## src/meta/agg.rs

the per-month pass that counts rows by type and sums tokens per session, and its cache · 273 บรรทัด

```rust
//! One pass per month over the rows' sources, and its cache in OUT/meta/cache/YYYY-MM.tsv.

use super::*;


/// Rows by type, in this order: user text, assistant text, thinking, tool calls, tool results, tool errors
/// (errors are results with err = true, so they are also counted in results).
pub const TYPES: [&str; 6] = ["user_text", "assistant_text", "thinking", "tool_calls", "tool_results", "tool_errors"];

pub(super) fn types_of(s: &str, role: Option<&str>, kind: Option<&str>, err: bool) -> [i64; 6] {
    let mut t = [0; 6];
    match (s, role, kind) {
        ("text", Some("user"), _) => t[0] = 1,
        ("text", Some("assistant"), _) => t[1] = 1,
        ("thinking", _, _) => t[2] = 1,
        ("tools", _, Some("call")) => t[3] = 1,
        ("tools", _, Some("result")) => {
            t[4] = 1;
            t[5] = err as i64;
        }
        _ => {}
    }
    t
}

#[derive(Default, Clone)]
pub(super) struct Agg {
    pub(super) n_rows: i64,
    pub(super) text_rows: i64,
    pub(super) text_msgs: i64,
    pub(super) by: [i64; 6],
    pub(super) first_ts: Option<String>,
    pub(super) last_ts: Option<String>,
}

impl Agg {
    pub(super) fn ts(&mut self, ts: Option<&str>) {
        let Some(t) = ts else { return };
        if self.first_ts.as_deref().is_none_or(|f| t < f) {
            self.first_ts = Some(t.to_string());
        }
        if self.last_ts.as_deref().is_none_or(|l| t > l) {
            self.last_ts = Some(t.to_string());
        }
    }
    pub(super) fn add(&mut self, o: &Agg) {
        self.n_rows += o.n_rows;
        self.text_rows += o.text_rows;
        self.text_msgs += o.text_msgs;
        for (a, b) in self.by.iter_mut().zip(o.by) {
            *a += b;
        }
        self.ts(o.first_ts.as_deref());
        self.ts(o.last_ts.as_deref());
    }
}

/// What a session's model answers cost: tokens summed over its API responses (each once: the month file keeps a
/// response's tokens on one row) and the answers per model.
#[derive(Default, Clone, Debug, PartialEq)]
pub(super) struct Usage {
    pub(super) out: i64,
    pub(super) inp: i64,
    pub(super) cache_read: i64,
    pub(super) cache_write: i64,
    pub(super) models: BTreeMap<String, i64>,
}

impl Usage {
    pub(super) fn add(&mut self, o: &Usage) {
        self.out += o.out;
        self.inp += o.inp;
        self.cache_read += o.cache_read;
        self.cache_write += o.cache_write;
        for (m, n) in &o.models {
            *self.models.entry(m.clone()).or_default() += n;
        }
    }
    /// The models, the most answers first, and their answer counts.
    pub(super) fn ranked(&self) -> (Vec<String>, Vec<i64>) {
        let mut v: Vec<(&String, &i64)> = self.models.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        (v.iter().map(|x| x.0.clone()).collect(), v.iter().map(|x| *x.1).collect())
    }
}

/// One month's aggregates: per file path, and per session (base, tier).
#[derive(Default)]
pub(super) struct MonthAgg {
    pub(super) files: HashMap<String, Agg>,
    pub(super) sessions: HashMap<(String, String), Agg>,
    pub(super) usage: HashMap<(String, String), Usage>,
}

/// msgkey for a text row is md5(to_json([s, role, ts, text])); count(DISTINCT msgkey) only needs a stable key of
/// (role, ts, text) with NULL kept apart from any string, so a length-prefixed blake2b-8 does the same job.
pub(super) fn msg_key(role: Option<&str>, ts: Option<&str>, text: Option<&str>) -> u64 {
    let mut st = blake2b_simd::Params::new().hash_length(8).to_state();
    for v in [role, ts, text] {
        match v {
            None => {
                st.update(&[0]);
            }
            Some(s) => {
                st.update(&[1]);
                st.update(&(s.len() as u64).to_le_bytes());
                st.update(s.as_bytes());
            }
        }
    }
    u64::from_le_bytes(st.finalize().as_bytes().try_into().unwrap())
}

pub(super) fn str_col<'a>(b: &'a RecordBatch, n: &str) -> &'a StringArray {
    b.column_by_name(n).unwrap().as_any().downcast_ref::<StringArray>().unwrap()
}

pub(super) fn opt(a: &StringArray, i: usize) -> Option<&str> {
    if a.is_null(i) { None } else { Some(a.value(i)) }
}

pub(super) fn month_agg(path: &Path, agent: &str) -> MonthAgg {
    let b = ParquetRecordBatchReaderBuilder::try_new(fs::File::open(path).unwrap()).unwrap();
    // model and the token columns came later: a month written before them has none
    let want = ["s", "role", "kind", "err", "ts", "text", "sources", "model", "msg_id", "out_tokens", "in_tokens", "cache_read", "cache_write"];
    let idx: Vec<usize> = want.iter().filter_map(|n| b.schema().index_of(n).ok()).collect();
    let mask = ProjectionMask::roots(b.parquet_schema(), idx);
    let rd = b.with_projection(mask).with_batch_size(16_384).build().unwrap();
    let prefix = format!("{agent}:");
    let mut out = MonthAgg::default();
    let mut class: HashMap<String, Option<(String, String)>> = HashMap::new(); // path -> session key (None = tier other)
    let mut msgs: HashMap<(String, String), HashSet<u64>> = HashMap::new();
    let mut touched: Vec<(String, String)> = vec![];
    for batch in rd {
        let batch = batch.unwrap();
        let (s, role, ts, text) = (str_col(&batch, "s"), str_col(&batch, "role"), str_col(&batch, "ts"), str_col(&batch, "text"));
        let kind = str_col(&batch, "kind");
        let err = batch.column_by_name("err").unwrap().as_any().downcast_ref::<arrow::array::BooleanArray>().unwrap();
        let model = batch.column_by_name("model").map(|a| a.as_any().downcast_ref::<StringArray>().unwrap());
        let msg_id = batch.column_by_name("msg_id").map(|a| a.as_any().downcast_ref::<StringArray>().unwrap());
        let tok = |n: &str| batch.column_by_name(n).map(|a| a.as_any().downcast_ref::<Int64Array>().unwrap());
        let (t_out, t_in, t_cr, t_cw) = (tok("out_tokens"), tok("in_tokens"), tok("cache_read"), tok("cache_write"));
        let src = batch.column_by_name("sources").unwrap().as_any().downcast_ref::<ListArray>().unwrap();
        let vals = src.values().as_any().downcast_ref::<StringArray>().unwrap();
        let offs = src.value_offsets();
        for i in 0..batch.num_rows() {
            let is_text = s.value(i) == "text";
            let t = opt(ts, i);
            let ty = types_of(s.value(i), opt(role, i), opt(kind, i), !err.is_null(i) && err.value(i));
            touched.clear();
            for j in offs[i] as usize..offs[i + 1] as usize {
                let Some(rest) = vals.value(j).strip_prefix(&prefix) else { continue };
                let Some((p, _line)) = rest.rsplit_once(':') else { continue };
                let fa = out.files.entry(p.to_string()).or_default();
                fa.n_rows += 1;
                fa.text_rows += is_text as i64;
                for (a, b) in fa.by.iter_mut().zip(ty) {
                    *a += b;
                }
                fa.ts(t);
                let k = class.entry(p.to_string()).or_insert_with(|| {
                    let tier = tier_of(p);
                    (tier != "other").then(|| (base_of(p).to_string(), tier.to_string()))
                });
                if let Some(k) = k
                    && !touched.contains(k) {
                    touched.push(k.clone()); // a row counts once per session, however many of its copies it holds
                }
            }
            // an answer: the row that carries its tokens, or an assistant text row with no response id (Codex)
            let n = |c: Option<&Int64Array>| c.and_then(|a| (!a.is_null(i)).then(|| a.value(i))).unwrap_or(0);
            let has_tokens = t_out.is_some_and(|a| !a.is_null(i));
            let answer = has_tokens || (is_text && opt(role, i) == Some("assistant") && msg_id.is_none_or(|a| a.is_null(i)));
            let mdl = model.and_then(|a| opt(a, i)).filter(|m| !m.is_empty());
            for k in &touched {
                if has_tokens || answer {
                    let u = out.usage.entry(k.clone()).or_default();
                    u.out += n(t_out);
                    u.inp += n(t_in);
                    u.cache_read += n(t_cr);
                    u.cache_write += n(t_cw);
                    if let (true, Some(m)) = (answer, mdl) {
                        *u.models.entry(m.to_string()).or_default() += 1;
                    }
                }
                let sa = out.sessions.entry(k.clone()).or_default();
                sa.n_rows += 1;
                sa.text_rows += is_text as i64;
                for (a, b) in sa.by.iter_mut().zip(ty) {
                    *a += b;
                }
                sa.ts(t);
                if is_text {
                    msgs.entry(k.clone()).or_default().insert(msg_key(opt(role, i), t, opt(text, i)));
                }
            }
        }
    }
    for (k, m) in msgs {
        out.sessions.get_mut(&k).unwrap().text_msgs = m.len() as i64;
    }
    out
}

// ---------- the month cache: OUT/meta/cache/YYYY-MM.tsv ----------

pub(super) fn stamp_of(p: &Path) -> String {
    let m = fs::metadata(p).unwrap();
    format!("{}\t{}.{}", m.size(), m.mtime(), m.mtime_nsec())
}

// NULL is written as \N: an empty ts is a real value (the undated month holds rows with ts = "")
pub(super) fn agg_tsv(a: &Agg) -> String {
    let by: Vec<String> = a.by.iter().map(|x| x.to_string()).collect();
    format!("{}\t{}\t{}\t{}\t{}\t{}", a.n_rows, a.text_rows, a.text_msgs, by.join("\t"), a.first_ts.as_deref().unwrap_or("\\N"),
            a.last_ts.as_deref().unwrap_or("\\N"))
}

pub(super) fn agg_from(c: &[&str]) -> Agg {
    let n = |i: usize| c[i].parse::<i64>().unwrap_or(0);
    let t = |i: usize| Some(c[i].to_string()).filter(|s| s != "\\N");
    Agg { n_rows: n(0), text_rows: n(1), text_msgs: n(2), by: std::array::from_fn(|j| n(3 + j)), first_ts: t(9), last_ts: t(10) }
}

/// The cache format and the rules it was counted with: bump it when a path rule changes what a session is (#4: omp
/// subagents are sessions), so every month is counted again once.
const CACHE_V: &str = "#5";

pub(super) fn save_cache(p: &Path, stamp: &str, agent: &str, m: &MonthAgg) {
    let tmp = p.with_extension("tsv.part");
    let mut w = std::io::BufWriter::new(fs::File::create(&tmp).unwrap());
    writeln!(w, "{CACHE_V}\t{stamp}\t{agent}").unwrap();
    for (f, a) in &m.files {
        writeln!(w, "F\t{f}\t{}", agg_tsv(a)).unwrap();
    }
    for ((base, tier), a) in &m.sessions {
        writeln!(w, "S\t{base}\t{tier}\t{}", agg_tsv(a)).unwrap();
    }
    for ((base, tier), u) in &m.usage {
        let models = serde_json::to_string(&u.models).unwrap();
        writeln!(w, "U\t{base}\t{tier}\t{}\t{}\t{}\t{}\t{models}", u.out, u.inp, u.cache_read, u.cache_write).unwrap();
    }
    w.flush().unwrap();
    drop(w);
    fs::rename(tmp, p).unwrap();
}

/// The cached aggregates, or None when the cache was made from another version of the month file.
pub(super) fn load_cache(p: &Path, stamp: &str, agent: &str) -> Option<MonthAgg> {
    let mut lines = BufReader::new(fs::File::open(p).ok()?).lines().map_while(Result::ok);
    if lines.next()? != format!("{CACHE_V}\t{stamp}\t{agent}") {
        return None;
    }
    let mut m = MonthAgg::default();
    for l in lines {
        let c: Vec<&str> = l.split('\t').collect();
        match c[0] {
            "F" if c.len() == 13 => {
                m.files.insert(c[1].to_string(), agg_from(&c[2..]));
            }
            "S" if c.len() == 14 => {
                m.sessions.insert((c[1].to_string(), c[2].to_string()), agg_from(&c[3..]));
            }
            "U" if c.len() == 8 => {
                let n = |i: usize| c[i].parse::<i64>().unwrap_or(0);
                let models = serde_json::from_str(c[7]).ok()?;
                m.usage.insert((c[1].to_string(), c[2].to_string()), Usage { out: n(3), inp: n(4), cache_read: n(5), cache_write: n(6), models });
            }
            _ => return None,
        }
    }
    Some(m)
}
```

## src/ops/text.rs

the text view of `relic-scan ops` · 74 บรรทัด

```rust
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
        Some("idle") => o.push(format!("CHAIN {}  idle  last pass {} took {} · next in about {}", c["machine"].as_str().unwrap_or(""),
            c["started"].as_str().unwrap_or("")[11..].to_string(), dur(c["took_s"].as_i64().unwrap_or(0)), dur(c["next_in_s"].as_i64().unwrap_or(0)))),
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
    if hub != "off" && v["hub"]["ok"].as_bool() == Some(true) {
        if let Some(l) = o.iter_mut().find(|l| l.starts_with("HUB")) {
            *l = format!("{}{}", link(on, hub, "HUB"), &l[3..]);
        }
    }
    let page = view_url(base, "ops");
    o.push(String::new());
    o.push(format!("page  {}", link(on, &page, &page)));
    o
}
```
