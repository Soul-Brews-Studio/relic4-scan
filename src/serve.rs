//! `relic-scan serve` / `up`: OUT/meta/map.html on 127.0.0.1 only. One page, two views:
//!   /  /map   the session star map
//!   /tree     homes -> repos -> sessions -> satellites
//! The page is read from disk on every request, so a `relic-scan run` in another shell shows up on reload.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn reply(mut s: TcpStream, status: &str, ctype: &str, body: &[u8]) {
    let head = format!("HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
                       body.len());
    let _ = s.write_all(head.as_bytes()).and_then(|_| s.write_all(body));
}

fn handle(s: TcpStream, page: &Path) {
    let mut rd = BufReader::new(match s.try_clone() {
        Ok(c) => c,
        Err(_) => return,
    });
    let mut line = String::new();
    if rd.read_line(&mut line).is_err() {
        return;
    }
    // drain the headers; nothing in them matters here
    let mut h = String::new();
    while rd.read_line(&mut h).is_ok_and(|n| n > 2) {
        h.clear();
    }
    let path = line.split_whitespace().nth(1).unwrap_or("/");
    let path = path.split(['?', '#']).next().unwrap_or("/");
    match path {
        "/" | "/map" | "/tree" | "/map.html" => match std::fs::read(page) {
            Ok(b) => reply(s, "200 OK", "text/html; charset=utf-8", &b),
            Err(e) => reply(s, "500 Internal Server Error", "text/plain", format!("{}: {e}\n", page.display()).as_bytes()),
        },
        "/favicon.ico" => reply(s, "204 No Content", "text/plain", b""),
        _ => reply(s, "404 Not Found", "text/plain", b"not found: try /map or /tree\n"),
    }
}

pub fn serve(out: &Path, port: u16) {
    let page = out.join("meta").join("map.html");
    if !page.exists() {
        eprintln!("serve: no {} (run `relic-scan run` first)", page.display());
        std::process::exit(1);
    }
    let l = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| {
        eprintln!("serve: 127.0.0.1:{port}: {e} (another server? try --port)");
        std::process::exit(1);
    });
    eprintln!("serve: http://127.0.0.1:{port}/map · http://127.0.0.1:{port}/tree  ({})  ctrl-c to stop", page.display());
    for s in l.incoming().flatten() {
        let page = page.clone();
        std::thread::spawn(move || handle(s, &page));
    }
}
