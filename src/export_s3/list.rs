//! `relic-scan export --list`: what the store holds, one line per month. No bucket listing: it reads the two small files the
//! export writes (`meta/months.json` for the seals, `export-manifest.json` for sizes and hashes) with GET, so a key that
//! may only read and write objects is enough.

use super::*;

/// One month as the store knows it.
#[derive(Debug, PartialEq)]
pub struct Row {
    pub month: String,
    pub rows: u64,
    pub first_ts: String,
    pub max_ts: String,
    pub sealed_at: String,
    pub resealed: u64,
    /// size and sha256 of the object in the store; None when this month was never sent (open, or sealed after the export)
    pub stored: Option<(u64, String)>,
}

impl Client {
    /// (status, body) of GET key.
    fn get(&self, key: &str) -> Result<(u16, String), String> {
        let uri = self.uri(key);
        let hs = self.headers("GET", &uri, &sha256_hex(b""), None);
        let mut req = self.agent.get(format!("{}{}", self.base, uri));
        for (k, v) in hs.iter().filter(|(k, _)| k != "host") {
            req = req.header(k, v);
        }
        let resp = req.call().map_err(|e| format!("GET {key}: {e}"))?;
        let status = resp.status().as_u16();
        let body = resp.into_body().read_to_string().map_err(|e| format!("GET {key}: {e}"))?;
        Ok((status, body))
    }
}

pub fn list(o: &Opts) -> Result<Vec<Row>, String> {
    let rest = o.to.strip_prefix("s3://").ok_or("--to must look like s3://bucket/prefix")?;
    let (bucket, prefix) = rest.split_once('/').unwrap_or((rest, ""));
    if bucket.is_empty() {
        return Err("--to must name a bucket: s3://bucket/prefix".into());
    }
    let prefix = if prefix.is_empty() || prefix.ends_with('/') { prefix.to_string() } else { format!("{prefix}/") };
    let client = Client::new(o, bucket)?;
    let fetch = |key: String| -> Result<serde_json::Value, String> {
        match client.get(&key)? {
            (200, b) => serde_json::from_str(&b).map_err(|e| format!("{key}: {e}")),
            (404, _) => Err(format!("{key} is not in the store: nothing was exported to s3://{bucket}/{prefix} yet")),
            (c, _) => Err(format!("GET {key}: HTTP {c} (wrong keys, bucket or endpoint?)")),
        }
    };
    let months = fetch(format!("{prefix}meta/months.json"))?;
    let manifest = fetch(format!("{prefix}export-manifest.json"))?;
    Ok(rows(&months, &manifest, &prefix))
}

/// The seals joined with the manifest. Months that are open or have no rows are left out unless they were sent.
pub fn rows(months: &serde_json::Value, manifest: &serde_json::Value, prefix: &str) -> Vec<Row> {
    let s = |v: &serde_json::Value, k: &str| v[k].as_str().unwrap_or("").to_string();
    months["months"].as_array().into_iter().flatten().filter_map(|m| {
        let month = s(m, "month");
        let o = &manifest["objects"][format!("{prefix}{month}.parquet")];
        let stored = o["size"].as_u64().map(|n| (n, s(o, "sha256")));
        (stored.is_some() || !s(m, "sealed_at").is_empty()).then(|| Row {
            rows: m["sealed_rows"].as_u64().filter(|&n| n > 0).or(m["rows"].as_u64()).unwrap_or(0),
            first_ts: s(m, "first_ts"), max_ts: s(m, "max_ts"), sealed_at: s(m, "sealed_at"), resealed: m["resealed"].as_u64().unwrap_or(0), month, stored,
        })
    }).collect()
}

/// The table `export --list` prints.
pub fn table(rows: &[Row]) -> Vec<String> {
    let day = |t: &str| t.get(..10).unwrap_or("—").to_string();
    let mut out = vec![format!("{:<9} {:>11}  {:<23} {:>10}  {:<8} {}", "month", "rows", "first → last (UTC)", "size", "sha256", "sealed")];
    for r in rows {
        let (size, sha) = r.stored.as_ref().map_or(("not sent".to_string(), "—".to_string()), |(n, h)| (format!("{:.1} MB", *n as f64 / 1e6), h.chars().take(8).collect()));
        let sealed = if r.sealed_at.is_empty() { "open".to_string() } else { format!("{}{}", day(&r.sealed_at), if r.resealed > 0 { format!(" (sealed again {}×)", r.resealed) } else { String::new() }) };
        out.push(format!("{:<9} {:>11}  {:<23} {:>10}  {:<8} {}", r.month, r.rows, format!("{} → {}", day(&r.first_ts), day(&r.max_ts)), size, sha, sealed));
    }
    let (n, bytes) = rows.iter().filter_map(|r| r.stored.as_ref()).fold((0, 0u64), |(n, b), (s, _)| (n + 1, b + s));
    out.push(format!("{n} months in the store · {:.1} MB · {} rows", bytes as f64 / 1e6, rows.iter().filter(|r| r.stored.is_some()).map(|r| r.rows).sum::<u64>()));
    out
}
