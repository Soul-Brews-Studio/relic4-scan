# REVIEW-auto — mechanical checks (verify.py), no LLM

34 findings in 11 chapters · code 23 · image 11

(book):0 [image] images/04-dry-run.png used 2 times: 01-intro.md:81, 09-privacy.md:19
(book):0 [image] images/11-seals-verify.png used 2 times: 07-seals.md:48, 09-privacy.md:46
(book):0 [image] images/13-duck-what.png used 2 times: 06-duckdb.md:45, 10-formats.md:165
(book):0 [image] images/16-duck-tokens.png used 2 times: 06-duckdb.md:113, 10-formats.md:216
(book):0 [image] images/17-duck-session.png used 2 times: 06-duckdb.md:142, 10-formats.md:232
(book):0 [image] images/18-duck-read.png used 2 times: 06-duckdb.md:178, 10-formats.md:312
(book):0 [image] images/20-ops.png used 2 times: 08-chain.md:19, 08-chain.md:67
(book):0 [image] images/46-describe-month.png used 2 times: 06-duckdb.md:30, 10-formats.md:181
(book):0 [image] images/47-describe-sess.png used 2 times: 06-duckdb.md:157, 10-formats.md:246
(book):0 [image] images/C1-smart-scan-donut.png used 2 times: 00-frontmatter.md:5, 03-web.md:41
(book):0 [image] images/W21-page-settings.png used 2 times: 04-more-pages.md:109, 07-seals.md:86
03-web.md:12 [code] 3 output line(s) not in a capture, first: relic-scan up # index แล้วเสิร์ฟ http://127.0.0.1:8787
06-duckdb.md:42 [code] 130 chars (max 76): "select coalesce(nullif(kind,''), nullif(role,''),
06-duckdb.md:86 [code] 138 chars (max 76): "select cat as repo, count(*) as sessions, sum(n_r
06-duckdb.md:110 [code] 194 chars (max 76): "select left(ts_th,7) as month, model, count(disti
06-duckdb.md:139 [code] 159 chars (max 76): "select sid, models, model_msgs, in_tokens, out_to
06-duckdb.md:175 [code] 160 chars (max 76): "select role, left(text, 70) as text from '*.parqu
06-duckdb.md:196 [code] 77 chars (max 76): "select left(text, 90) as text, red from '*.parque
07-seals.md:101 [code] 3 output line(s) not in a capture, first: export RELIC_S3_ENDPOINT=http://odin.follow-rankine.ts.net:3
07-seals.md:122 [code] 1 output line(s) not in a capture, first: relic-scan export --to s3://bucket/relic --dry-run
07-seals.md:132 [code] 1 output line(s) not in a capture, first: relic-scan export --to s3://bucket/relic
07-seals.md:142 [code] 2 output line(s) not in a capture, first: relic-scan export --to s3://bucket/relic --verify
07-seals.md:153 [code] 1 output line(s) not in a capture, first: relic-scan export --to s3://bucket/relic --overwrite
08-chain.md:88 [code] 3 output line(s) not in a capture, first: relic-scan relic-extract relic4 embed
09-privacy.md:110 [code] 2 output line(s) not in a capture, first: A sealed month that differs in the store (it was sealed agai
09-privacy.md:123 [code] 1 output line(s) not in a capture, first: move ~/.relic-scan aside, then relic-scan run
09-privacy.md:141 [code] 1 output line(s) not in a capture, first: rm -f ~/.local/bin/relic-scan && cp target/release/relic-sca
10-formats.md:133 [code] 96 chars (max 76): pub(super) fn types_of(s: &str, role: Option<&str>
10-formats.md:192 [code] 119 chars (max 76): // one API response is several rows (one per conte
10-formats.md:193 [code] 106 chars (max 76): // the first of them only, the largest any copy re
10-formats.md:194 [code] 78 chars (max 76): let mut tokens: HashMap<&str, (usize, [Option<i64>
10-formats.md:196 [code] 91 chars (max 76): if let (Some(id), true) = (cands[i].msg_id.as_dere
10-formats.md:261 [code] 102 chars (max 76): } else if r.main_uuid.is_match(path) || r.main_rol
10-formats.md:282 [code] 91 chars (max 76): let Some(p) = proj.filter(|p| !p.is_empty()) else 
