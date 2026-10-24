use super::*;

#[test]
fn the_destination_is_checked_when_it_is_saved_and_holds_no_keys() {
    let d = crate::parse::tests::tmp("s3cfg");
    let mut c = crate::config::Config::default();
    c.roots = vec!["~".into()];
    for bad in ["bucket/x", "s3://", "s3:// b/x"] {
        c.s3.to = bad.into();
        assert!(crate::config::save(&d, c.clone()).is_err(), "{bad}");
    }
    c.s3 = crate::config::S3Cfg { to: " s3://relic-scan-sealed-months/m5 ".into(), endpoint: "http://odin.follow-rankine.ts.net:3920/".into(), region: "garage".into() };
    let saved = crate::config::save(&d, c.clone()).unwrap();
    assert_eq!((saved.s3.to.as_str(), saved.s3.endpoint.as_str()), ("s3://relic-scan-sealed-months/m5", "http://odin.follow-rankine.ts.net:3920"));
    let text = std::fs::read_to_string(d.join("_config.json")).unwrap();
    assert!(!text.to_lowercase().contains("secret") && !text.contains("AWS_"), "no keys in the settings file: {text}");
    c.s3.endpoint = "ftp://x".into();
    assert!(crate::config::save(&d, c).is_err());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn keys_in_a_file_are_used_only_when_the_file_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let d = crate::parse::tests::tmp("s3keys");
    let f = d.join("s3.env");
    std::fs::write(&f, "# comment\nAWS_ACCESS_KEY_ID=AK\nexport AWS_SECRET_ACCESS_KEY=\"SK\"\n").unwrap();
    // SAFETY: only this test points RELIC_S3_ENV_FILE at a file, and the other tests do not read it
    unsafe { std::env::set_var("RELIC_S3_ENV_FILE", &f) };
    std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(export_s3::creds_status().1.contains("readable by others") || std::env::var("AWS_ACCESS_KEY_ID").is_ok());
    std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o600)).unwrap();
    if std::env::var("AWS_ACCESS_KEY_ID").is_err() {
        assert_eq!(export_s3::creds_status().0, "file");
    }
    unsafe { std::env::remove_var("RELIC_S3_ENV_FILE") };
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn status_says_when_nothing_is_set_up_and_start_refuses() {
    let d = crate::parse::tests::tmp("s3status");
    let s = status(&d);
    assert_eq!((s["configured"].as_bool(), s["running"].as_bool()), (Some(false), Some(false)));
    assert!(start(&d, true).unwrap_err().contains("Settings"));
    let _ = std::fs::remove_dir_all(&d);
}
