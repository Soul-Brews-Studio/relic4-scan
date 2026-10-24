use super::*;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

#[test]
fn sigv4_matches_the_aws_documentation_example() {
    // "Example: GET Object" in the S3 Signature Version 4 docs (examplebucket, Range header)
    let c = Creds { key: "AKIAIOSFODNN7EXAMPLE".into(), secret: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".into(), token: None };
    let hs = vec![
        ("host".to_string(), "examplebucket.s3.amazonaws.com".to_string()),
        ("range".to_string(), "bytes=0-9".to_string()),
        ("x-amz-content-sha256".to_string(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()),
        ("x-amz-date".to_string(), "20130524T000000Z".to_string()),
    ];
    let a = sign(&c, "us-east-1", "GET", "/test.txt", &hs, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "20130524T000000Z");
    assert!(a.contains("SignedHeaders=host;range;x-amz-content-sha256;x-amz-date"), "{a}");
    assert!(a.ends_with("Signature=f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41"), "{a}");
}

#[test]
fn plain_http_is_only_for_this_machine_and_the_tailnet() {
    for h in ["localhost:3900", "127.0.0.1:9000", "odin.follow-rankine.ts.net:3900", "100.111.101.21:3900"] {
        assert!(private_host(h), "{h}");
    }
    for h in ["s3.example.com", "203.0.113.5:3900", "100.200.1.1", "192.168.1.5:9000"] {
        assert!(!private_host(h), "{h}");
    }
    assert_eq!(uri_encode("a b/2026-09.parquet"), "a%20b/2026-09.parquet");
}

type Store = Arc<Mutex<BTreeMap<String, (Vec<u8>, String)>>>;

/// A tiny S3: HEAD and PUT in memory, path-style; it checks that the body matches x-amz-content-sha256 and that every
/// request is signed (Authorization with the five signed headers a PUT carries).
fn fake_s3() -> (String, Store, std::thread::JoinHandle<()>, Arc<Mutex<bool>>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = format!("http://{}", l.local_addr().unwrap());
    let store: Store = Arc::default();
    let stop = Arc::new(Mutex::new(false));
    let (s2, stop2) = (store.clone(), stop.clone());
    l.set_nonblocking(true).unwrap();
    let h = std::thread::spawn(move || loop {
        if *stop2.lock().unwrap() {
            break;
        }
        let Ok((mut c, _)) = l.accept() else {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        };
        c.set_nonblocking(false).unwrap();
        let mut r = BufReader::new(c.try_clone().unwrap());
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        let (method, path) = { let mut p = line.split_whitespace(); (p.next().unwrap_or("").to_string(), p.next().unwrap_or("").to_string()) };
        let mut hdr: BTreeMap<String, String> = BTreeMap::new();
        loop {
            let mut h = String::new();
            r.read_line(&mut h).unwrap();
            if h.trim().is_empty() {
                break;
            }
            if let Some((k, v)) = h.split_once(':') {
                hdr.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
        }
        let auth = hdr.get("authorization").cloned().unwrap_or_default();
        let signed_ok = auth.starts_with("AWS4-HMAC-SHA256 Credential=AKIATEST/") && auth.contains("/us-east-1/s3/aws4_request");
        let resp = if !signed_ok {
            "HTTP/1.1 403 Forbidden\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_string()
        } else if method == "PUT" {
            let n: usize = hdr.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
            let mut body = vec![0u8; n];
            r.read_exact(&mut body).unwrap();
            let ok = sha256_hex(&body) == hdr["x-amz-content-sha256"] && auth.contains("SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-meta-sha256");
            if ok {
                s2.lock().unwrap().insert(path.clone(), (body, hdr["x-amz-meta-sha256"].clone()));
                "HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_string()
            } else {
                "HTTP/1.1 400 Bad Request\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_string()
            }
        } else {
            match s2.lock().unwrap().get(&path) {
                Some((b, m)) => format!("HTTP/1.1 200 OK\r\nx-amz-meta-sha256: {m}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n", b.len()),
                None => "HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_string(),
            }
        };
        c.write_all(resp.as_bytes()).unwrap();
    });
    (addr, store, h, stop)
}

fn index_with_two_closed_months(d: &Path) {
    use crate::parse::tests::lines;
    let src = d.join("s.jsonl");
    std::fs::write(&src, lines(2, 10)).unwrap();
    let job = crate::parse::Job { path: src.to_str().unwrap().to_string(), size: std::fs::metadata(&src).unwrap().len(), verdict: String::new(), ctx: Default::default() };
    crate::parse::run(vec![job], &d.join("out"), "t@x", 2, false, false, None);
    crate::manifest::build_at(&d.join("out"), "2026-08"); // 2026-07 is closed, so it is sealed
}

fn opts(d: &Path, addr: &str) -> Opts {
    Opts { out: d.join("out"), to: "s3://relic-test/p".into(), endpoint: Some(addr.into()), region: None, dry_run: false, overwrite: false, verify: false, insecure_http: false }
}

#[test]
fn export_sends_sealed_months_once_and_refuses_to_replace_a_changed_one() {
    let d = crate::parse::tests::tmp("export-s3");
    index_with_two_closed_months(&d);
    // SAFETY: only these tests set these variables, and with the same values
    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "AKIATEST");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "secret");
    }
    let (addr, store, h, stop) = fake_s3();
    let o = opts(&d, &addr);
    // a dry run reads only
    let dry = run(&Opts { dry_run: true, ..opts(&d, &addr) }).unwrap();
    assert_eq!((dry.uploaded, store.lock().unwrap().len()), (0, 0));
    // first export: the sealed month, months.json, then the manifest
    let a = run(&o).unwrap();
    assert!(a.uploaded >= 3 && a.refused == 0, "{a:?}");
    let keys: Vec<String> = store.lock().unwrap().keys().cloned().collect();
    assert!(keys.contains(&"/relic-test/p/2026-07.parquet".to_string()), "{keys:?}");
    assert!(keys.contains(&"/relic-test/p/meta/months.json".to_string()) && keys.contains(&"/relic-test/p/export-manifest.json".to_string()), "{keys:?}");
    assert!(!keys.iter().any(|k| k.contains("undated") || k.contains("_state")), "only sealed months and the seals: {keys:?}");
    // the stored month is byte for byte the local file
    let local = std::fs::read(d.join("out").join("2026-07.parquet")).unwrap();
    assert_eq!(store.lock().unwrap()["/relic-test/p/2026-07.parquet"].0, local);
    // second export: nothing to send but the manifest's own timestamp
    let b = run(&o).unwrap();
    assert_eq!((b.uploaded <= 1, b.same >= 2, b.refused), (true, true, 0), "{b:?}");
    // verify agrees
    let v = run(&Opts { verify: true, ..opts(&d, &addr) }).unwrap();
    assert_eq!((v.differs, v.missing), (0, 0), "{v:?}");
    // a sealed month changes (it is sealed again): refused by default, replaced with --overwrite
    store.lock().unwrap().get_mut("/relic-test/p/2026-07.parquet").unwrap().1 = "0".repeat(64);
    let c = run(&o).unwrap();
    assert_eq!(c.refused, 1, "{c:?}");
    assert_eq!(run(&Opts { verify: true, ..opts(&d, &addr) }).unwrap().differs, 1);
    let e = run(&Opts { overwrite: true, ..opts(&d, &addr) }).unwrap();
    assert_eq!((e.refused, e.uploaded >= 1), (0, true), "{e:?}");
    // months.json changes with every scan: verify calls it outdated, not a difference, and says the months are fine
    let mj = d.join("out").join("meta").join("months.json");
    let mut t = std::fs::read_to_string(&mj).unwrap();
    t.push('\n');
    std::fs::write(&mj, t).unwrap();
    let w = run(&Opts { verify: true, ..opts(&d, &addr) }).unwrap();
    assert_eq!((w.outdated, w.differs, w.missing), (1, 0, 0), "{w:?}");
    *stop.lock().unwrap() = true;
    h.join().unwrap();
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn plain_http_to_a_public_host_is_refused_before_anything_is_sent() {
    let d = crate::parse::tests::tmp("export-s3-guard");
    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "AKIATEST");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "secret");
    }
    let e = run(&Opts { endpoint: Some("http://s3.example.com".into()), ..opts(&d, "x") }).unwrap_err();
    assert!(e.contains("plain http"), "{e}");
    let e = run(&Opts { to: "bucket/no-scheme".into(), ..opts(&d, "http://127.0.0.1:1") }).unwrap_err();
    assert!(e.contains("s3://"), "{e}");
    let _ = std::fs::remove_dir_all(&d);
}
