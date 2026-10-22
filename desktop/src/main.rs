//! Relic Scan desktop (Tauri v2): the relic-scan page in a native window.
//!
//! The app starts the same server `relic-scan serve` runs, inside this process on a free 127.0.0.1 port, and opens a
//! window on it. Map, tree, paths, the theme button and the index button all work as in the browser; the index
//! button runs an incremental run in this process. Nothing listens beyond 127.0.0.1 and nothing goes to the network.
//!
//!   RELIC_SCAN_KIND   claude (default) · codex · … · several joined by commas · all (every agent)
//!   RELIC_SCAN_OUT    the output directory (default ~/.relic-scan/<kind>, or ~/.relic-scan for all)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use tauri::{WebviewUrl, WebviewWindowBuilder};

fn main() {
    let kind: Vec<String> = match std::env::var("RELIC_SCAN_KIND").ok().as_deref() {
        Some("all") => vec![],
        Some(k) if !k.is_empty() => k.split(',').map(|s| s.trim().to_string()).collect(),
        _ => vec!["claude".into()],
    };
    let out = std::env::var("RELIC_SCAN_OUT").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| relic_scan::DEFAULT_OUT.into());
    let out = PathBuf::from(relic_scan::out_dir(&out, &kind));
    let listener = relic_scan::serve::bind(0);
    let port = listener.local_addr().expect("local address").port();
    {
        let (out, kind) = (out.clone(), kind.clone());
        std::thread::spawn(move || relic_scan::serve::serve_on(listener, &out, kind));
    }
    eprintln!("relic-scan desktop: http://127.0.0.1:{port}/map  ({})", out.display());
    tauri::Builder::default()
        .setup(move |app| {
            let url = format!("http://127.0.0.1:{port}/map").parse().expect("url");
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("Relic Scan")
                .inner_size(1440.0, 900.0)
                .min_inner_size(900.0, 600.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("relic-scan desktop");
}
