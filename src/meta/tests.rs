use super::*;

const U: &str = "0b867d73-405b-4b03-8c46-aee7648f1bd7";

#[test]
fn tiers() {
    let h = "/Users/x/.claude/projects/-opt-Code-github-com-laris-co-neo";
    assert_eq!(tier_of(&format!("{h}/{U}.jsonl")), "main");
    assert_eq!(tier_of(&format!("{h}/{U}/subagents/agent-a1.jsonl")), "subagent");
    assert_eq!(tier_of(&format!("{h}/{U}/subagents/workflows/wf_1/agent-a2.jsonl")), "workflow-agent");
    assert_eq!(tier_of(&format!("{h}/{U}/subagents/workflows/wf_1/journal.jsonl")), "other");
    assert_eq!(tier_of(&format!("/Users/x/.codex/sessions/2026/07/01/rollout-2026-07-01T00-00-00-{U}.jsonl")), "main");
    assert_eq!(tier_of("/Users/x/.codex/sessions/rollout-x.jsonl"), "other");
    assert_eq!(tier_of("/Users/x/.omp/agent/sessions/p/2026_ab12-cd.jsonl"), "main");
    // an omp subagent: a task's transcript inside its parent session's folder
    assert_eq!(tier_of(&format!("/Users/x/.omp/agent/sessions/p/2026-10-13T02-07-20-692Z_{U}/GqlShapes.jsonl")), "subagent");
    assert_eq!(tier_of(&format!("/Users/x/.omp/agent/sessions/p/2026-10-13T02-07-20-692Z_{U}.jsonl")), "main");
    assert_eq!(tier_of("/Users/x/.omx/logs/run.jsonl"), "main");
    assert_eq!(tier_of("/Users/x/.omx/history.jsonl"), "other");
    assert_eq!(tier_of("/Users/x/.claude/history.jsonl"), "other");
    // the scan uses the very same rules
    assert_eq!(crate::scan::tier("/Users/x/.omp/agent/sessions/p/2026_ab12-cd.jsonl"), "main");
}

#[test]
fn repos_and_projects() {
    let p = format!("/h/.claude/projects/-opt-Code-github-com-Soul-Brews-Studio-relic4-scan/{U}.jsonl");
    assert_eq!(proj_of(&p).as_deref(), Some("-opt-Code-github-com-Soul-Brews-Studio-relic4-scan"));
    assert_eq!(cat_of("claude", proj_of(&p).as_deref()), "relic4-scan");
    assert_eq!(cat_of("claude", Some("-opt-Code-github-com-laris-co-neo-oracle-wt-fix-x")), "neo-oracle");
    assert_eq!(cat_of("claude", Some("-Users-x-proj-worktrees-thing-wt-2")), "thing");
    assert_eq!(cat_of("claude", Some("-Users-x-oracle-probe-cwd-1")), "probe (automated)");
    assert_eq!(cat_of("claude", Some("-Users-x")), "home / other");
    assert_eq!(cat_of("claude", None), "(no project dir)");
    assert_eq!(cat_of("codex", None), "codex (no cwd in path)");
    assert_eq!(parent_of(&format!("/h/p/{U}/subagents/agent-1.jsonl")).as_deref(), Some(U));
    assert_eq!(parent_of(&format!("/Users/x/.omp/agent/sessions/p/2026-10-13T02-07-20-692Z_{U}/GqlShapes.jsonl")).as_deref(), Some(U));
    // a Claude session's id is its file name; a codex rollout and a task-named omp subagent take the id inside the file
    assert_eq!(extract(&rx().uuid_file, &format!("{U}.jsonl"), 1), U);
    assert_eq!(extract(&rx().uuid_file, &format!("rollout-2026-07-01T00-00-00-{U}.jsonl"), 1), "");
    assert_eq!(extract(&rx().uuid_file, "GqlShapes.jsonl", 1), "");
    assert_eq!(home_of(&p), "/h/.claude");
}

#[test]
fn msg_key_keeps_null_apart_from_empty() {
    assert_ne!(msg_key(Some("user"), None, Some("x")), msg_key(Some("user"), Some(""), Some("x")));
    assert_ne!(msg_key(Some("ab"), Some("c"), None), msg_key(Some("a"), Some("bc"), None));
    assert_eq!(msg_key(Some("user"), Some("t"), Some("x")), msg_key(Some("user"), Some("t"), Some("x")));
}

#[test]
fn months_and_cache_round_trip() {
    assert_eq!(month_of("2026-07.parquet"), Some("2026-07"));
    assert_eq!(month_of("undated.parquet"), Some("undated"));
    assert_eq!(month_of(".parquet"), None);
    assert_eq!(month_of("_state.tsv"), None);
    let a = Agg { n_rows: 5, text_rows: 2, text_msgs: 1, by: [1, 1, 0, 2, 1, 1], first_ts: Some(String::new()), last_ts: None };
    let line = format!("F\tp\t{}", agg_tsv(&a));
    let c: Vec<&str> = line.split('\t').collect();
    assert_eq!(c.len(), 13);
    let b = agg_from(&c[2..]);
    assert_eq!((b.n_rows, b.by, b.first_ts, b.last_ts), (5, [1, 1, 0, 2, 1, 1], Some(String::new()), None));
}

#[test]
fn usage_adds_ranks_and_survives_the_cache() {
    let mut a = Usage { out: 10, inp: 2, cache_read: 5, cache_write: 1, models: BTreeMap::from([("opus".to_string(), 2), ("haiku".to_string(), 1)]) };
    a.add(&Usage { out: 5, inp: 1, cache_read: 0, cache_write: 0, models: BTreeMap::from([("haiku".to_string(), 3)]) });
    assert_eq!((a.out, a.inp, a.cache_read, a.cache_write), (15, 3, 5, 1));
    assert_eq!(a.ranked(), (vec!["haiku".to_string(), "opus".to_string()], vec![4, 2]));
    let d = std::env::temp_dir().join(format!("relic-scan-test-{}-usage", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    let mut m = MonthAgg::default();
    m.usage.insert(("b".into(), "main".into()), a.clone());
    let p = d.join("2026-07.tsv");
    save_cache(&p, "1\t2.3", "t@x", &m);
    let back = load_cache(&p, "1\t2.3", "t@x").expect("cache reads back");
    assert_eq!(back.usage[&("b".to_string(), "main".to_string())], a);
    let _ = fs::remove_dir_all(&d);
}
