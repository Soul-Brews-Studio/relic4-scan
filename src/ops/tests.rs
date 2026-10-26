use super::*;

const LOG: &str = "2026-10-24 14:48:26 chain start (m5)
run: scan 97.2s · new 1 · grown 5 · rewritten 38 · unchanged 178,612 · gone 0 · 0.38 GB to read
run: done in 177.8s · state 178,656 files -> /x/_state.tsv
3 part(s) cut in 12.0s · 320,706 chunks · 15,468 messages cleaned out
2026-10-24 14:51:00 relic5 push m5 -> nat@white:/home/nat/relic5/parts/m5 rc=0 8s
2026-10-24 14:52:00 relic5 push m5 -> nat@white:/home/nat/relic5/parts/m5 rc=0 7s
embed: 93 chunks on ane in 64s -> relic5/vectors/x.parquet
2026-10-24 14:53:21 chain end
2026-10-24 14:53:23 SYNC OK  text|1075229 thinking|104869 tools|2412500
2026-10-24 15:03:23 chain start (m5)
run: scan 101.0s · new 4 · grown 5
run: done in 147.1s · state 235,158 files -> /x/_state.tsv
";

fn at(t: &str) -> i64 {
    epoch(t).unwrap()
}

#[test]
fn passes_are_read_from_the_log() {
    let p = parse_log(LOG);
    assert_eq!(p.len(), 2);
    assert_eq!((p[0].machine.as_str(), p[0].end.as_deref()), ("m5", Some("2026-10-24 14:53:21")));
    assert_eq!(p[0].scan.as_ref().map(|s| s.0), Some(177.8));
    assert!(p[0].scan.as_ref().unwrap().1.contains("new 1"), "{:?}", p[0].scan);
    assert_eq!(p[0].push, vec![8.0, 7.0]);
    assert_eq!(p[0].embed.as_ref().map(|e| e.0), Some(64.0));
    assert!(p[0].sync.as_deref().unwrap().starts_with("SYNC OK"));
    assert!(p[1].end.is_none() && p[1].extract.is_none());
}

#[test]
fn a_running_pass_names_its_stage_and_an_eta() {
    let v = chain_view(&parse_log(LOG), at("2026-10-24 15:06:00"));
    assert_eq!(v["state"], "running");
    let st: Vec<&str> = v["stages"].as_array().unwrap().iter().map(|s| s["state"].as_str().unwrap()).collect();
    assert_eq!(st, ["done", "running", "waiting", "waiting", "waiting"], "scan is done, extract is next");
    assert_eq!(v["elapsed_s"], 157);
    // the one finished pass took 4:55, so 2:38 are left
    assert_eq!(v["eta_s"].as_f64().map(|x| x.round()), Some(138.0), "{v}");
}

#[test]
fn an_idle_chain_says_when_the_next_pass_starts() {
    let v = chain_view(&parse_log(&LOG[..LOG.find("2026-10-24 15:03:23").unwrap()]), at("2026-10-24 14:58:21"));
    assert_eq!((v["state"].as_str(), v["took_s"].as_i64(), v["next_in_s"].as_i64()), (Some("idle"), Some(295), Some(300)), "{v}");
    assert!(v["stages"].as_array().unwrap().iter().all(|s| s["state"] == "done"));
}

#[test]
fn an_empty_log_is_never_not_an_error() {
    assert_eq!(chain_view(&parse_log(""), 0)["state"], "never");
}

#[test]
fn processes_are_named_and_the_hub_view_keeps_the_useful_parts() {
    assert_eq!(what("relic-scan run"), Some("scan"));
    assert_eq!(what("/bin/sh /Users/x/.local/bin/relic-chain"), Some("chain"));
    assert_eq!(what("ssh -o BatchMode=yes nat@white '~/relic5/app/.venv/bin/relic5 index --wait'"), Some("hub index (on white)"));
    assert_eq!(what("vim notes.txt"), None);
    let h = hub_view(&serde_json::json!({"freshness": {"machines": {"m5": {"push_age_s": 90, "sync": "SYNC OK"}}}, "jobs": {"running": []}, "warnings": [], "resources": {"load": [1.5], "cpus": 32}}));
    assert_eq!((h["ok"].as_bool(), h["machines"][0]["machine"].as_str(), h["cpus"].as_i64()), (Some(true), Some("m5"), Some(32)));
}

#[test]
fn the_text_view_shows_every_stage() {
    let v = serde_json::json!({"at": 0, "log": "x", "chain": chain_view(&parse_log(LOG), at("2026-10-24 15:06:00")), "processes": [],
        "hub": {"ok": false, "error": "off"}, "s3": {"running": false}});
    let t = render(&v).join("\n");
    assert!(t.contains("RUNNING") && t.contains("● extract") && t.contains("✓ scan") && t.contains("about 2:18 left"), "{t}");
}

#[test]
fn the_ops_text_links_the_hub_and_the_page_only_for_a_terminal() {
    let v = serde_json::json!({"chain": {"state": "never"}, "log": "/x", "processes": [], "hub": {"ok": false, "error": "off"}, "s3": {}});
    let plain = render_linked(&v, "http://127.0.0.1:8787", "http://hub:1", false);
    assert!(plain.iter().all(|l| !l.contains('\x1b')), "{plain:?}");
    assert_eq!(plain.last().unwrap(), "page  http://127.0.0.1:8787/ops");
    assert!(plain.iter().any(|l| l.starts_with("HUB  not read")));
    let v2 = serde_json::json!({"chain": {"state": "never"}, "log": "/x", "processes": [], "hub": {"ok": true, "load": [1.0], "cpus": 4, "running": [], "machines": [], "warnings": []}, "s3": {}});
    let on = render_linked(&v2, "http://127.0.0.1:8787", "http://hub:1", true);
    assert!(on.iter().any(|l| l.starts_with("\x1b]8;;http://hub:1\x1b\\HUB\x1b]8;;\x1b\\  load")), "{on:?}");
    let unread = render_linked(&v, "http://127.0.0.1:8787", "http://hub:1", true);
    assert!(unread.iter().any(|l| l.starts_with("HUB  not read")), "a hub that was not read gets no link: {unread:?}");
    assert!(on.last().unwrap().contains("\x1b]8;;http://127.0.0.1:8787/ops"));
    let off = render_linked(&v2, "http://127.0.0.1:8787", "off", true);
    assert!(off.iter().any(|l| l.starts_with("HUB  load")), "no link to a hub that is switched off");
}

#[test]
fn an_idle_chain_without_a_loop_says_manual() {
    let mut c = chain_view(&parse_log(&LOG[..LOG.find("2026-10-24 15:03:23").unwrap()]), at("2026-10-24 14:58:21"));
    let with_loop = render(&serde_json::json!({"at": 0, "log": "x", "chain": c, "processes": [], "hub": {"ok": false, "error": "off"}, "s3": {"running": false}})).join("\n");
    assert!(with_loop.contains("next in about"), "{with_loop}");
    c["next_in_s"] = serde_json::Value::Null;
    let manual = render(&serde_json::json!({"at": 0, "log": "x", "chain": c, "processes": [], "hub": {"ok": false, "error": "off"}, "s3": {"running": false}})).join("\n");
    assert!(manual.contains("manual (no loop)") && !manual.contains("next in"), "{manual}");
}
