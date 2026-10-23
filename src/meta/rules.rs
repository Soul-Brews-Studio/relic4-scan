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
            main_rollout: r(&format!(r"rollout-.*{UUID}\.jsonl$")),
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
