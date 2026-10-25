//! Links from the terminal to the page (`relic-scan up` serves it): the URL of a view or of a query in a view, the OSC 8
//! escape that makes it clickable, and the call that opens it. The page reads `/<view>#<encodeURIComponent(query)>`.

use std::io::IsTerminal;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Where the page is: `RELIC_PAGE_URL`, else the local server on its default port.
pub fn base() -> String {
    std::env::var("RELIC_PAGE_URL").map(|u| u.trim_end_matches('/').to_string()).unwrap_or_else(|_| "http://127.0.0.1:8787".into())
}

/// `encodeURIComponent`: everything but letters, digits and `-_.!~*'()` is %-encoded, byte by byte.
pub fn enc(s: &str) -> String {
    s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

pub fn view_url(base: &str, view: &str) -> String {
    format!("{base}/{view}")
}

/// A view with its query box filled in (`sid:…`, `repo:…`, a folder word).
pub fn query_url(base: &str, view: &str, query: &str) -> String {
    format!("{base}/{view}#{}", enc(query))
}

pub fn session_url(base: &str, sid: &str) -> String {
    query_url(base, "tree", &format!("sid:{sid}"))
}

/// `text` as a clickable link (OSC 8; iTerm2, Ghostty, kitty, WezTerm, Terminal.app). Unknown terminals show `text` only.
pub fn osc8(url: &str, text: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
}

/// Links go to a terminal only: not into a pipe or a file, not with NO_COLOR or `RELIC_LINKS=off`.
pub fn enabled() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none() && std::env::var("RELIC_LINKS").map_or(true, |v| v != "off")
}

/// `text` as a link when `on`, else as it is.
pub fn link(on: bool, url: &str, text: &str) -> String {
    if on { osc8(url, text) } else { text.to_string() }
}

/// Is something listening where `base` points? (150 ms; a link to a stopped server goes nowhere.)
pub fn up(base: &str) -> bool {
    let hp = base.split("//").nth(1).unwrap_or(base).split('/').next().unwrap_or("");
    let hp = if hp.contains(':') { hp.to_string() } else { format!("{hp}:80") };
    hp.to_socket_addrs().ok().and_then(|mut a| a.next()).is_some_and(|a: SocketAddr| TcpStream::connect_timeout(&a, Duration::from_millis(150)).is_ok())
}

/// Open `url` in the default browser.
pub fn open(url: &str) -> std::io::Result<()> {
    let mut c = std::process::Command::new(if cfg!(target_os = "macos") { "open" } else { "xdg-open" });
    c.arg(url).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_query_is_encoded_like_the_page_does_it() {
        assert_eq!(enc("sid:ab c#1"), "sid%3Aab%20c%231");
        assert_eq!(enc("a-b_c.d~(e)"), "a-b_c.d~(e)");
        assert_eq!(enc("ψ"), "%CF%88");
    }

    #[test]
    fn urls() {
        assert_eq!(view_url("http://127.0.0.1:8787", "ops"), "http://127.0.0.1:8787/ops");
        assert_eq!(session_url("http://x:1", "0a1b"), "http://x:1/tree#sid%3A0a1b");
    }

    #[test]
    fn a_link_is_an_osc8_pair_and_off_is_plain_text() {
        assert_eq!(osc8("http://a/b", "go"), "\x1b]8;;http://a/b\x1b\\go\x1b]8;;\x1b\\");
        assert_eq!(link(false, "http://a/b", "go"), "go");
        assert!(link(true, "http://a/b", "go").contains("\x1b]8;;http://a/b"));
    }

    #[test]
    fn a_closed_port_is_not_up() {
        assert!(!up("http://127.0.0.1:1"));
    }
}
