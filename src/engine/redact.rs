//! Redaction (engine.py RULES, same order): secrets in a text become `[kind:first…last]` and the kinds are listed.

use std::sync::OnceLock;

use regex::bytes::{Regex, RegexBuilder};

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
