//! OUT/_config.json: what the page's Settings view saves, and what `run` (and the page's Index now) uses when no
//! flag says otherwise. A flag on the command line always wins: `--root`, `--exclude` and `--depth` replace these.
//!
//!   roots           folders to walk                          default ~ and /opt/Code
//!   exclude         a path holding any of these is skipped    default none
//!   depth           how deep the walk looks for agent homes   default 8
//!   auto_check_min  the server runs Check changes this often  0 (off), 15, 30 or 60 minutes; it never indexes

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Where the S3 copy goes (Export view, `relic-scan export`). No keys: they come from the environment or
/// ~/.config/relic-scan/s3.env (mode 600), never from this file.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct S3Cfg {
    /// s3://bucket/prefix; empty = no S3 copy set up
    pub to: String,
    /// an S3-compatible endpoint (Garage, MinIO); empty = AWS S3
    pub endpoint: String,
    pub region: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub roots: Vec<String>,
    pub exclude: Vec<String>,
    pub depth: usize,
    pub auto_check_min: u32,
    pub s3: S3Cfg,
}

impl Default for Config {
    fn default() -> Self {
        Config { roots: vec!["~".into(), "/opt/Code".into()], exclude: vec![], depth: 8, auto_check_min: 0, s3: S3Cfg::default() }
    }
}

pub fn load(out: &Path) -> Config {
    std::fs::read(out.join("_config.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

/// Check what the page sent, then write it (temp file, then rename). Err says what is wrong, for the page to show.
pub fn save(out: &Path, mut c: Config) -> Result<Config, String> {
    let clean = |v: Vec<String>| -> Vec<String> {
        let mut v: Vec<String> = v.into_iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        v.dedup();
        v
    };
    c.roots = clean(c.roots);
    c.exclude = clean(c.exclude);
    if c.roots.is_empty() {
        return Err("at least one folder to scan".into());
    }
    if let Some(r) = c.roots.iter().find(|r| !Path::new(&crate::expand(r)).is_dir()) {
        return Err(format!("{r} is not a folder on this machine"));
    }
    c.s3.to = c.s3.to.trim().to_string();
    c.s3.endpoint = c.s3.endpoint.trim().trim_end_matches('/').to_string();
    c.s3.region = c.s3.region.trim().to_string();
    if !c.s3.to.is_empty() {
        let rest = c.s3.to.strip_prefix("s3://").ok_or("the S3 destination looks like s3://bucket/prefix")?;
        if rest.split('/').next().is_none_or(str::is_empty) || c.s3.to.contains(char::is_whitespace) {
            return Err("the S3 destination needs a bucket: s3://bucket/prefix".into());
        }
    }
    if !c.s3.endpoint.is_empty() && !(c.s3.endpoint.starts_with("http://") || c.s3.endpoint.starts_with("https://")) {
        return Err("the S3 endpoint starts with http:// or https://".into());
    }
    if !(1..=32).contains(&c.depth) {
        return Err("depth is 1 to 32".into());
    }
    if ![0, 15, 30, 60].contains(&c.auto_check_min) {
        return Err("auto check is off, 15, 30 or 60 minutes".into());
    }
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let tmp = out.join("_config.json.part");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&c).unwrap()).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, out.join("_config.json")).map_err(|e| e.to_string())?;
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_what_is_valid_and_refuses_the_rest() {
        let d = std::env::temp_dir().join(format!("relic-scan-test-{}-config", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        assert_eq!(load(&d), Config::default(), "no file: the defaults");
        let ok = Config { roots: vec![" /tmp ".into(), "".into()], exclude: vec!["node_modules".into()], depth: 6, auto_check_min: 30, s3: S3Cfg::default() };
        let saved = save(&d, ok).unwrap();
        assert_eq!(saved.roots, ["/tmp"]);
        assert_eq!(load(&d), saved);
        for bad in [Config { roots: vec![], ..Config::default() }, Config { roots: vec!["/no/such/dir/x".into()], ..Config::default() },
                    Config { depth: 0, ..Config::default() }, Config { auto_check_min: 7, ..Config::default() }] {
            assert!(save(&d, bad).is_err());
        }
        assert_eq!(load(&d), saved, "a refused save leaves the file as it was");
        let _ = std::fs::remove_dir_all(&d);
    }
}
