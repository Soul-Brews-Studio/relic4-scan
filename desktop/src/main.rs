//! ARRA Relic Scan desktop (Tauri v2): the relic-scan page in a native window.
//!
//! The app starts the same server `relic-scan serve` runs, inside this process on a free 127.0.0.1 port, and opens a
//! window on it. Map, tree, paths, the theme button and the index button all work as in the browser; the index
//! button runs an incremental run in this process. Nothing listens beyond 127.0.0.1 and nothing goes to the network.
//!
//!   RELIC_SCAN_KIND   all (default, like `relic-scan run`) · claude · codex · … · several joined by commas
//!   --view V          open on a view: scan (default), scans, banks, health, map, tree, paths, disk, settings; scan#contents opens the contents popup
//!   RELIC_SCAN_OUT    the output directory (default ~/.relic-scan, or ~/.relic-scan/<kind> with a kind)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use tauri::{WebviewUrl, WebviewWindowBuilder};

fn main() {
    let kind: Vec<String> = match std::env::var("RELIC_SCAN_KIND").ok().as_deref() {
        Some(k) if !k.is_empty() && k != "all" => k.split(',').map(|s| s.trim().to_string()).collect(),
        _ => vec![],
    };
    let out = std::env::var("RELIC_SCAN_OUT").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| relic_scan::DEFAULT_OUT.into());
    let out = PathBuf::from(relic_scan::out_dir(&out, &kind));
    let args: Vec<String> = std::env::args().collect();
    let view = args.iter().position(|a| a == "--view").and_then(|i| args.get(i + 1)).cloned().unwrap_or_default();
    // a view of the page, optionally with its #hash (scan#contents opens the contents popup)
    let base = view.split('#').next().unwrap_or("");
    let views = ["scan", "scans", "banks", "health", "map", "tree", "paths", "disk", "settings"];
    let view = if views.contains(&base) && view.bytes().all(|b| b.is_ascii_alphanumeric() || b"#:-_".contains(&b)) { view } else { String::new() };
    let listener = relic_scan::serve::bind(0);
    let port = listener.local_addr().expect("local address").port();
    {
        let (out, kind) = (out.clone(), kind.clone());
        std::thread::spawn(move || relic_scan::serve::serve_on(listener, &out, kind));
    }
    eprintln!("relic-scan desktop: http://127.0.0.1:{port}/{view}  ({})", out.display());
    tauri::Builder::default()
        .setup(move |app| {
            let url = format!("http://127.0.0.1:{port}/{view}").parse().expect("url");
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("ARRA Relic Scan")
                .inner_size(1440.0, 900.0)
                .min_inner_size(900.0, 600.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("relic-scan desktop");
}
