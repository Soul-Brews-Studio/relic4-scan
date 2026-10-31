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
        // a leading \b also matches after a JSON escape such as \n (see the match loop)
        let b = |p: &str| {
            let p = if let Some(r) = p.strip_prefix(r"(?i)\b") { r#"(?i)(?:\b|\\[nrt"])"#.to_string() + r }
                    else if let Some(r) = p.strip_prefix(r"\b") { r#"(?:\b|\\[nrt"])"#.to_string() + r } else { p.to_string() };
            RegexBuilder::new(&p).unicode(false).build().expect("rule")
        };
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
            // shapes the rules above miss (found with decoy transcripts, 2026-10-31)
            Rule { kind: "named-secret", hints: &["token", "secret", "key", "passw", "hmac", "credential"], group: 1,
                   rx: b(r#"(?i)\b[A-Z0-9_]*(?:TOKEN|SECRET|KEY|PASSWORD|PASSWD|HMAC|CREDENTIALS?)[A-Z0-9_]*\\?["']?\s*[:=]\s*\\?["']?([A-Za-z0-9!#%&*+,./:;=?@^_|~-]{16,})"#) },
            Rule { kind: "discord-token", hints: &["."], group: 0, rx: b(r"\b[MNO][A-Za-z0-9_-]{23,27}\.[A-Za-z0-9_-]{6}\.[A-Za-z0-9_-]{27,40}\b") },
            Rule { kind: "telegram-token", hints: &[":AA"], group: 0, rx: b(r"\b[0-9]{8,10}:AA[A-Za-z0-9_-]{33}\b") },
            Rule { kind: "linear-key", hints: &["lin_"], group: 0, rx: b(r"\blin_(?:api|oauth)_[A-Za-z0-9]{32,}") },
            Rule { kind: "stripe-key", hints: &["k_live_", "k_test_"], group: 0, rx: b(r"\b[sr]k_(?:live|test)_[A-Za-z0-9]{20,}") },
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
        // hints compared case-insensitively, so PASSWORD= and BEARER reach their (?i) rules
        let low = s.to_ascii_lowercase();
        if !rule.hints.iter().any(|h| low.contains(&h.to_ascii_lowercase())) {
            continue;
        }
        let bytes = s.as_bytes();
        let mut out = String::new();
        let mut last = 0usize;
        let mut hit = false;
        for caps in rule.rx.captures_iter(bytes) {
            let Some(m) = caps.get(rule.group) else { continue };
            let (mut a, z) = (m.start(), m.end());
            if bytes.get(a) == Some(&b'\\') && a + 2 < z {
                a += 2; // keep the JSON escape the leading boundary consumed
            }
            if a < last || bytes.get(a) == Some(&b'[') {
                continue; // already redacted by an earlier rule
            }
            let v = &s[a..z];
            if (rule.kind == "password" || rule.kind == "named-secret") && !secret_like(v) {
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

#[cfg(test)]
mod tests {
    // What these guard: secret shapes that reached the index raw before this change (found with decoys and on a
    // real home, 2026-10-31), plus look-alikes that must stay readable. All values are fake.
    use super::redact;

    // fake keys are assembled at runtime so secret scanners do not read the test file as a leak
    fn discord() -> String {
        format!("MTAx{}.{}.{}", "A".repeat(22), "B".repeat(6), "C".repeat(38))
    }

    fn kinds(s: &str) -> String {
        redact(s).1
    }

    #[test]
    fn uppercase_names_reach_case_insensitive_rules() {
        assert_eq!(kinds("PASSWORD=Pw7abc123XYZdef9"), "password");
        assert!(redact("Authorization: BEARER a1b2c3d4e5f6g7h8i9j0k1l2m3n4").0.contains("[bearer:"));
    }

    #[test]
    fn named_env_and_json_keys() {
        assert_eq!(kinds("CLOUDFLARE_API_TOKEN=a1B2c3D4e5F6g7H8i9J0k1L2m3N4o5P6q7R8s9T0"), "named-secret");
        assert_eq!(kinds("FLEET_HMAC_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"), "named-secret");
        assert_eq!(kinds(r#"{"secret_access_key": "0123456789abcdef0123456789abcdef"}"#), "named-secret");
        // a JSON key inside an escaped string, as in tool input or Codex function_call arguments
        assert_eq!(kinds(r#"{\"api_key\": \"a1b2c3d4e5f6g7h8i9j0\"}"#), "named-secret");
    }

    #[test]
    fn provider_tokens() {
        assert_eq!(kinds(&discord()), "discord-token");
        assert_eq!(kinds(&format!("7123456789:AA{}", "a".repeat(33))), "telegram-token");
        assert_eq!(kinds(&format!("lin_{}_{}", "api", "a".repeat(40))), "linear-key");
        assert_eq!(kinds(&format!("sk_{}_{}", "test", "a".repeat(24))), "stripe-key");
    }

    #[test]
    fn token_after_an_escaped_newline_keeps_the_escape() {
        let (clean, k) = redact(&format!(r"cat .env\n{}", discord()));
        assert_eq!(k, "discord-token");
        assert!(clean.starts_with(r"cat .env\n[discord-token:"), "{clean}");
        assert_eq!(kinds(&format!(r"x\ngh{}_{}", "p", "a".repeat(36))), "github-token");
    }

    #[test]
    fn look_alikes_stay() {
        for s in ["commit 0123456789abcdef0123456789abcdef01234567 fixed it",
                  "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  dist/app.tar.gz",
                  "session 3f2b8c1e-9d4a-4b7e-8f1a-2c3d4e5f6a7b resumed",
                  "the key: see docs"] {
            assert_eq!(redact(s), (s.to_string(), String::new()), "{s}");
        }
    }
}
