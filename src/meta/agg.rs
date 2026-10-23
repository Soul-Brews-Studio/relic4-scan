//! One pass per month over the rows' sources, and its cache in OUT/meta/cache/YYYY-MM.tsv.

use super::*;


/// Rows by type, in this order: user text, assistant text, thinking, tool calls, tool results, tool errors
/// (errors are results with err = true, so they are also counted in results).
pub const TYPES: [&str; 6] = ["user_text", "assistant_text", "thinking", "tool_calls", "tool_results", "tool_errors"];

pub(super) fn types_of(s: &str, role: Option<&str>, kind: Option<&str>, err: bool) -> [i64; 6] {
    let mut t = [0; 6];
    match (s, role, kind) {
        ("text", Some("user"), _) => t[0] = 1,
        ("text", Some("assistant"), _) => t[1] = 1,
        ("thinking", _, _) => t[2] = 1,
        ("tools", _, Some("call")) => t[3] = 1,
        ("tools", _, Some("result")) => {
            t[4] = 1;
            t[5] = err as i64;
        }
        _ => {}
    }
    t
}

#[derive(Default, Clone)]
pub(super) struct Agg {
    pub(super) n_rows: i64,
    pub(super) text_rows: i64,
    pub(super) text_msgs: i64,
    pub(super) by: [i64; 6],
    pub(super) first_ts: Option<String>,
    pub(super) last_ts: Option<String>,
}

impl Agg {
    pub(super) fn ts(&mut self, ts: Option<&str>) {
        let Some(t) = ts else { return };
        if self.first_ts.as_deref().is_none_or(|f| t < f) {
            self.first_ts = Some(t.to_string());
        }
        if self.last_ts.as_deref().is_none_or(|l| t > l) {
            self.last_ts = Some(t.to_string());
        }
    }
    pub(super) fn add(&mut self, o: &Agg) {
        self.n_rows += o.n_rows;
        self.text_rows += o.text_rows;
        self.text_msgs += o.text_msgs;
        for (a, b) in self.by.iter_mut().zip(o.by) {
            *a += b;
        }
        self.ts(o.first_ts.as_deref());
        self.ts(o.last_ts.as_deref());
    }
}

/// One month's aggregates: per file path, and per session (base, tier).
#[derive(Default)]
pub(super) struct MonthAgg {
    pub(super) files: HashMap<String, Agg>,
    pub(super) sessions: HashMap<(String, String), Agg>,
}

/// msgkey for a text row is md5(to_json([s, role, ts, text])); count(DISTINCT msgkey) only needs a stable key of
/// (role, ts, text) with NULL kept apart from any string, so a length-prefixed blake2b-8 does the same job.
pub(super) fn msg_key(role: Option<&str>, ts: Option<&str>, text: Option<&str>) -> u64 {
    let mut st = blake2b_simd::Params::new().hash_length(8).to_state();
    for v in [role, ts, text] {
        match v {
            None => {
                st.update(&[0]);
            }
            Some(s) => {
                st.update(&[1]);
                st.update(&(s.len() as u64).to_le_bytes());
                st.update(s.as_bytes());
            }
        }
    }
    u64::from_le_bytes(st.finalize().as_bytes().try_into().unwrap())
}

pub(super) fn str_col<'a>(b: &'a RecordBatch, n: &str) -> &'a StringArray {
    b.column_by_name(n).unwrap().as_any().downcast_ref::<StringArray>().unwrap()
}

pub(super) fn opt(a: &StringArray, i: usize) -> Option<&str> {
    if a.is_null(i) { None } else { Some(a.value(i)) }
}

pub(super) fn month_agg(path: &Path, agent: &str) -> MonthAgg {
    let b = ParquetRecordBatchReaderBuilder::try_new(fs::File::open(path).unwrap()).unwrap();
    let want = ["s", "role", "kind", "err", "ts", "text", "sources"];
    let idx: Vec<usize> = want.iter().map(|n| b.schema().index_of(n).unwrap()).collect();
    let mask = ProjectionMask::roots(b.parquet_schema(), idx);
    let rd = b.with_projection(mask).with_batch_size(16_384).build().unwrap();
    let prefix = format!("{agent}:");
    let mut out = MonthAgg::default();
    let mut class: HashMap<String, Option<(String, String)>> = HashMap::new(); // path -> session key (None = tier other)
    let mut msgs: HashMap<(String, String), HashSet<u64>> = HashMap::new();
    let mut touched: Vec<(String, String)> = vec![];
    for batch in rd {
        let batch = batch.unwrap();
        let (s, role, ts, text) = (str_col(&batch, "s"), str_col(&batch, "role"), str_col(&batch, "ts"), str_col(&batch, "text"));
        let kind = str_col(&batch, "kind");
        let err = batch.column_by_name("err").unwrap().as_any().downcast_ref::<arrow::array::BooleanArray>().unwrap();
        let src = batch.column_by_name("sources").unwrap().as_any().downcast_ref::<ListArray>().unwrap();
        let vals = src.values().as_any().downcast_ref::<StringArray>().unwrap();
        let offs = src.value_offsets();
        for i in 0..batch.num_rows() {
            let is_text = s.value(i) == "text";
            let t = opt(ts, i);
            let ty = types_of(s.value(i), opt(role, i), opt(kind, i), !err.is_null(i) && err.value(i));
            touched.clear();
            for j in offs[i] as usize..offs[i + 1] as usize {
                let Some(rest) = vals.value(j).strip_prefix(&prefix) else { continue };
                let Some((p, _line)) = rest.rsplit_once(':') else { continue };
                let fa = out.files.entry(p.to_string()).or_default();
                fa.n_rows += 1;
                fa.text_rows += is_text as i64;
                for (a, b) in fa.by.iter_mut().zip(ty) {
                    *a += b;
                }
                fa.ts(t);
                let k = class.entry(p.to_string()).or_insert_with(|| {
                    let tier = tier_of(p);
                    (tier != "other").then(|| (base_of(p).to_string(), tier.to_string()))
                });
                if let Some(k) = k
                    && !touched.contains(k) {
                    touched.push(k.clone()); // a row counts once per session, however many of its copies it holds
                }
            }
            for k in &touched {
                let sa = out.sessions.entry(k.clone()).or_default();
                sa.n_rows += 1;
                sa.text_rows += is_text as i64;
                for (a, b) in sa.by.iter_mut().zip(ty) {
                    *a += b;
                }
                sa.ts(t);
                if is_text {
                    msgs.entry(k.clone()).or_default().insert(msg_key(opt(role, i), t, opt(text, i)));
                }
            }
        }
    }
    for (k, m) in msgs {
        out.sessions.get_mut(&k).unwrap().text_msgs = m.len() as i64;
    }
    out
}

// ---------- the month cache: OUT/meta/cache/YYYY-MM.tsv ----------

pub(super) fn stamp_of(p: &Path) -> String {
    let m = fs::metadata(p).unwrap();
    format!("{}\t{}.{}", m.size(), m.mtime(), m.mtime_nsec())
}

// NULL is written as \N: an empty ts is a real value (the undated month holds rows with ts = "")
pub(super) fn agg_tsv(a: &Agg) -> String {
    let by: Vec<String> = a.by.iter().map(|x| x.to_string()).collect();
    format!("{}\t{}\t{}\t{}\t{}\t{}", a.n_rows, a.text_rows, a.text_msgs, by.join("\t"), a.first_ts.as_deref().unwrap_or("\\N"),
            a.last_ts.as_deref().unwrap_or("\\N"))
}

pub(super) fn agg_from(c: &[&str]) -> Agg {
    let n = |i: usize| c[i].parse::<i64>().unwrap_or(0);
    let t = |i: usize| Some(c[i].to_string()).filter(|s| s != "\\N");
    Agg { n_rows: n(0), text_rows: n(1), text_msgs: n(2), by: std::array::from_fn(|j| n(3 + j)), first_ts: t(9), last_ts: t(10) }
}

/// The cache format and the rules it was counted with: bump it when a path rule changes what a session is (#4: omp
/// subagents are sessions), so every month is counted again once.
const CACHE_V: &str = "#4";

pub(super) fn save_cache(p: &Path, stamp: &str, agent: &str, m: &MonthAgg) {
    let tmp = p.with_extension("tsv.part");
    let mut w = std::io::BufWriter::new(fs::File::create(&tmp).unwrap());
    writeln!(w, "{CACHE_V}\t{stamp}\t{agent}").unwrap();
    for (f, a) in &m.files {
        writeln!(w, "F\t{f}\t{}", agg_tsv(a)).unwrap();
    }
    for ((base, tier), a) in &m.sessions {
        writeln!(w, "S\t{base}\t{tier}\t{}", agg_tsv(a)).unwrap();
    }
    w.flush().unwrap();
    drop(w);
    fs::rename(tmp, p).unwrap();
}

/// The cached aggregates, or None when the cache was made from another version of the month file.
pub(super) fn load_cache(p: &Path, stamp: &str, agent: &str) -> Option<MonthAgg> {
    let mut lines = BufReader::new(fs::File::open(p).ok()?).lines().map_while(Result::ok);
    if lines.next()? != format!("{CACHE_V}\t{stamp}\t{agent}") {
        return None;
    }
    let mut m = MonthAgg::default();
    for l in lines {
        let c: Vec<&str> = l.split('\t').collect();
        match c[0] {
            "F" if c.len() == 13 => {
                m.files.insert(c[1].to_string(), agg_from(&c[2..]));
            }
            "S" if c.len() == 14 => {
                m.sessions.insert((c[1].to_string(), c[2].to_string()), agg_from(&c[3..]));
            }
            _ => return None,
        }
    }
    Some(m)
}
