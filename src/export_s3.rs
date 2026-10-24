//! `relic-scan export --to s3://bucket/prefix`: copy the sealed months to an S3-compatible store (AWS S3, Garage, MinIO).
//!
//! What goes: every month whose seal is in meta/months.json (`YYYY-MM.parquet`), then `meta/months.json` (the seals
//! themselves), then `export-manifest.json` (what was sent: key, size, sha256, month). Months go first, so the store never
//! holds a seal whose month is missing. The open month, the state and the logs are not sent: they change every run.
//!
//! Safe to re-run: every object carries its sha256 in `x-amz-meta-sha256`. A HEAD says "same" and nothing is sent. A sealed
//! month that differs in the store (it was sealed again after a late row) is refused unless --overwrite is given, so a
//! sealed copy is never replaced by accident. `--dry-run` only reads (HEAD); `--verify` compares the store with the files.
//!
//! The endpoint and keys come from the environment, never the command line or the output:
//!   RELIC_S3_ENDPOINT (or AWS_ENDPOINT_URL_S3, AWS_ENDPOINT_URL)   unset = AWS S3 (https, virtual-hosted)
//!   RELIC_S3_REGION   (or AWS_REGION, AWS_DEFAULT_REGION)           default us-east-1
//!   AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_SESSION_TOKEN     the credentials
//! A custom endpoint is addressed path-style (bucket in the path), which Garage and MinIO need. Plain http is refused
//! unless the host is this machine or on the tailnet (*.ts.net, 100.64/10), or --insecure-http says so.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

pub struct Opts {
    pub out: PathBuf,
    pub to: String,
    pub endpoint: Option<String>,
    pub region: Option<String>,
    pub dry_run: bool,
    pub overwrite: bool,
    pub verify: bool,
    pub insecure_http: bool,
}

#[derive(Debug, Default)]
pub struct Summary {
    pub uploaded: usize,
    pub same: usize,
    pub refused: usize,
    pub missing: usize,
    pub differs: usize,
    pub bytes: u64,
}

fn hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut m = HmacSha256::new_from_slice(key).expect("hmac takes any key length");
    m.update(data);
    m.finalize().into_bytes().to_vec()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn sha256_hex(b: &[u8]) -> String {
    hex(&Sha256::digest(b))
}

fn sha256_file(p: &Path) -> std::io::Result<(String, u64)> {
    let mut f = fs::File::open(p)?;
    let (mut h, mut buf, mut n) = (Sha256::new(), vec![0u8; 1 << 20], 0u64);
    loop {
        let k = f.read(&mut buf)?;
        if k == 0 {
            break;
        }
        h.update(&buf[..k]);
        n += k as u64;
    }
    Ok((hex(&h.finalize()), n))
}

/// S3 path encoding: everything but unreserved characters and '/' becomes %XX.
fn uri_encode(s: &str) -> String {
    s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => (b as char).to_string(),
        _ => format!("%{b:02X}"),
    }).collect()
}

pub struct Creds {
    pub key: String,
    pub secret: String,
    pub token: Option<String>,
}

/// AWS Signature Version 4 for S3: the Authorization header value. `headers` are (lowercase name, value); the signed set is
/// all of them, so the caller passes exactly the headers it sends (host, x-amz-content-sha256, x-amz-date, ...).
pub fn sign(c: &Creds, region: &str, method: &str, uri: &str, headers: &[(String, String)], payload_hash: &str, amz_date: &str) -> String {
    let mut hs: Vec<(String, String)> = headers.iter().map(|(k, v)| (k.to_lowercase(), v.trim().to_string())).collect();
    hs.sort();
    let canon_headers: String = hs.iter().map(|(k, v)| format!("{k}:{v}\n")).collect();
    let signed = hs.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(";");
    let canonical = format!("{method}\n{uri}\n\n{canon_headers}\n{signed}\n{payload_hash}");
    let day = &amz_date[..8];
    let scope = format!("{day}/{region}/s3/aws4_request");
    let to_sign = format!("AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}", sha256_hex(canonical.as_bytes()));
    let k = hmac(format!("AWS4{}", c.secret).as_bytes(), day.as_bytes());
    let k = hmac(&k, region.as_bytes());
    let k = hmac(&k, b"s3");
    let k = hmac(&k, b"aws4_request");
    let sig = hex(&hmac(&k, to_sign.as_bytes()));
    format!("AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed}, Signature={sig}", c.key)
}

/// Plain http is only for this machine and the tailnet.
pub fn private_host(host: &str) -> bool {
    let h = host.split(':').next().unwrap_or(host);
    if h == "localhost" || h.ends_with(".ts.net") || h.starts_with("127.") {
        return true;
    }
    let p: Vec<u32> = h.split('.').filter_map(|x| x.parse().ok()).collect();
    p.len() == 4 && p[0] == 100 && (64..=127).contains(&p[1])
}

struct Target {
    key: String,
    path: Option<PathBuf>,
    body: Option<Vec<u8>>,
    month: bool,
}

struct Client {
    agent: ureq::Agent,
    creds: Creds,
    region: String,
    base: String,   // scheme://authority, no trailing slash
    host: String,   // the Host header value
    bucket: String,
    virtual_host: bool,
}

impl Client {
    fn new(o: &Opts, bucket: &str) -> Result<Client, String> {
        let env = |ks: &[&str]| ks.iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()));
        let endpoint = o.endpoint.clone().or_else(|| env(&["RELIC_S3_ENDPOINT", "AWS_ENDPOINT_URL_S3", "AWS_ENDPOINT_URL"]));
        let region = o.region.clone().or_else(|| env(&["RELIC_S3_REGION", "AWS_REGION", "AWS_DEFAULT_REGION"])).unwrap_or_else(|| "us-east-1".into());
        let key = env(&["AWS_ACCESS_KEY_ID", "RELIC_S3_KEY"]).ok_or("set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY")?;
        let secret = env(&["AWS_SECRET_ACCESS_KEY", "RELIC_S3_SECRET"]).ok_or("set AWS_SECRET_ACCESS_KEY")?;
        let creds = Creds { key, secret, token: env(&["AWS_SESSION_TOKEN"]) };
        let (base, virtual_host) = match endpoint {
            Some(e) => (e.trim_end_matches('/').to_string(), false),
            None => (format!("https://{bucket}.s3.{region}.amazonaws.com"), true),
        };
        let (scheme, authority) = base.split_once("://").ok_or("the endpoint must start with http:// or https://")?;
        if scheme == "http" && !private_host(authority) && !o.insecure_http {
            return Err(format!("{authority} over plain http is not on this machine or the tailnet: use https, or pass --insecure-http"));
        }
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(3600)))
            .build()
            .into();
        Ok(Client { agent, creds, region, host: authority.to_string(), base, bucket: bucket.to_string(), virtual_host })
    }

    fn uri(&self, key: &str) -> String {
        if self.virtual_host { format!("/{}", uri_encode(key)) } else { format!("/{}/{}", self.bucket, uri_encode(key)) }
    }

    fn headers(&self, method: &str, uri: &str, payload_hash: &str, meta: Option<&str>) -> Vec<(String, String)> {
        let date = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let mut hs = vec![("host".to_string(), self.host.clone()), ("x-amz-content-sha256".into(), payload_hash.into()), ("x-amz-date".into(), date.clone())];
        if let Some(m) = meta {
            hs.push(("x-amz-meta-sha256".into(), m.into()));
        }
        if let Some(t) = &self.creds.token {
            hs.push(("x-amz-security-token".into(), t.clone()));
        }
        let auth = sign(&self.creds, &self.region, method, uri, &hs, payload_hash, &date);
        hs.push(("authorization".into(), auth));
        hs
    }

    /// (status, the sha256 the object was stored with, its size)
    fn head(&self, key: &str) -> Result<(u16, Option<String>, Option<u64>), String> {
        let uri = self.uri(key);
        let hs = self.headers("HEAD", &uri, &sha256_hex(b""), None);
        let mut req = self.agent.head(format!("{}{}", self.base, uri));
        for (k, v) in hs.iter().filter(|(k, _)| k != "host") {
            req = req.header(k, v);
        }
        let resp = req.call().map_err(|e| format!("HEAD {key}: {e}"))?;
        let get = |n: &str| resp.headers().get(n).and_then(|v| v.to_str().ok()).map(str::to_string);
        Ok((resp.status().as_u16(), get("x-amz-meta-sha256"), get("content-length").and_then(|v| v.parse().ok())))
    }

    fn put(&self, key: &str, t: &Target, sha: &str, size: u64) -> Result<(), String> {
        let uri = self.uri(key);
        let hs = self.headers("PUT", &uri, sha, Some(sha));
        let mut req = self.agent.put(format!("{}{}", self.base, uri)).header("content-length", size.to_string());
        for (k, v) in hs.iter().filter(|(k, _)| k != "host") {
            req = req.header(k, v);
        }
        let resp = if let Some(b) = &t.body {
            req.send(&b[..])
        } else {
            let mut f = fs::File::open(t.path.as_ref().unwrap()).map_err(|e| format!("{key}: {e}"))?;
            req.send(ureq::SendBody::from_reader(&mut f))
        }
        .map_err(|e| format!("PUT {key}: {e}"))?;
        let status = resp.status().as_u16();
        if (200..300).contains(&status) {
            return Ok(());
        }
        let mut body = resp.into_body();
        let msg = body.read_to_string().unwrap_or_default();
        Err(format!("PUT {key}: HTTP {status} {}", msg.chars().take(200).collect::<String>()))
    }
}

fn plan(o: &Opts, prefix: &str) -> Result<Vec<Target>, String> {
    let file = o.out.join("meta").join("months.json");
    let months: serde_json::Value = serde_json::from_slice(&fs::read(&file).map_err(|e| format!("{}: {e} (run relic-scan run first)", file.display()))?)
        .map_err(|e| format!("{}: {e}", file.display()))?;
    let mut t = vec![];
    for m in months["months"].as_array().into_iter().flatten() {
        let (name, sealed) = (m["month"].as_str().unwrap_or(""), m["sealed_at"].as_str().unwrap_or(""));
        let p = o.out.join(format!("{name}.parquet"));
        if !sealed.is_empty() && p.is_file() {
            t.push(Target { key: format!("{prefix}{name}.parquet"), path: Some(p), body: None, month: true });
        }
    }
    t.push(Target { key: format!("{prefix}meta/months.json"), path: Some(file), body: None, month: false });
    Ok(t)
}

pub fn run(o: &Opts) -> Result<Summary, String> {
    let rest = o.to.strip_prefix("s3://").ok_or("--to must look like s3://bucket/prefix")?;
    let (bucket, prefix) = rest.split_once('/').unwrap_or((rest, ""));
    if bucket.is_empty() {
        return Err("--to must name a bucket: s3://bucket/prefix".into());
    }
    let prefix = if prefix.is_empty() || prefix.ends_with('/') { prefix.to_string() } else { format!("{prefix}/") };
    let client = Client::new(o, bucket)?;
    let mut targets = plan(o, &prefix)?;
    let mut s = Summary::default();
    let mut sent: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut i = 0;
    while i < targets.len() {
        let t = &targets[i];
        let (sha, size) = match (&t.path, &t.body) {
            (Some(p), _) => sha256_file(p).map_err(|e| format!("{}: {e}", p.display()))?,
            (None, Some(b)) => (sha256_hex(b), b.len() as u64),
            _ => unreachable!(),
        };
        let (status, remote, _) = client.head(&t.key)?;
        let action = match (status, remote.as_deref()) {
            (200, Some(r)) if r == sha => "same",
            (200, _) if t.month && !o.overwrite && !o.verify => "refused",
            (200, _) if o.verify => "differs",
            (404, _) if o.verify => "missing",
            (200, _) | (404, _) => "upload",
            (c, _) => return Err(format!("HEAD {}: HTTP {c} (wrong keys, bucket or endpoint?)", t.key)),
        };
        match action {
            "same" => s.same += 1,
            "refused" => s.refused += 1,
            "differs" => s.differs += 1,
            "missing" => s.missing += 1,
            _ if o.dry_run || o.verify => {}
            _ => {
                client.put(&t.key, t, &sha, size)?;
                s.uploaded += 1;
                s.bytes += size;
            }
        }
        println!("{:<9} {:>12}  {}", if o.dry_run && action == "upload" { "would send" } else { action }, size, t.key);
        sent.insert(t.key.clone(), serde_json::json!({"size": size, "sha256": sha}));
        i += 1;
        // the manifest of what the store now holds goes last, built from the objects just handled
        if i == targets.len() && !o.verify && !sent.contains_key(&format!("{prefix}export-manifest.json")) {
            let body = serde_json::to_vec_pretty(&serde_json::json!({
                "version": 1, "objects": sent,
            })).unwrap();
            targets.push(Target { key: format!("{prefix}export-manifest.json"), path: None, body: Some(body), month: false });
        }
    }
    Ok(s)
}

#[cfg(test)]
mod tests;
