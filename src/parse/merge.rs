//! The merge: one month's existing file + this run's raw parts -> one truth file, copies folded.

use super::*;


/// One candidate copy of a truth row, from the existing month file or from a new raw part.
pub(super) struct Cand {
    pub(super) s: String, h: i64, idx: i64, engine: Option<String>, role: Option<String>, typ: Option<String>, sid: Option<String>,
    pub(super) ts: Option<String>, ts_th: Option<String>, text: Option<String>, kind: Option<String>, call_id: Option<String>,
    pub(super) name: Option<String>, body: Option<String>, err: Option<bool>, red: Option<String>,
    pub(super) first: (String, i64), // the copy's (file, line): the lowest wins, so incremental == cold
}

pub(super) fn col_str<'a>(b: &'a RecordBatch, name: &str) -> &'a StringArray {
    b.column_by_name(name).unwrap().as_any().downcast_ref::<StringArray>().unwrap()
}

pub(super) fn s_opt(a: &StringArray, i: usize) -> Option<String> {
    if a.is_null(i) { None } else { Some(a.value(i).to_string()) }
}

pub(super) fn cands_from(b: &RecordBatch, srcs: &mut HashMap<i64, BTreeMap<String, i64>>, agent: &str, out: &mut Vec<Cand>) {
    let (h, idx) = (b.column_by_name("h").unwrap().as_any().downcast_ref::<Int64Array>().unwrap(),
                    b.column_by_name("idx").unwrap().as_any().downcast_ref::<Int64Array>().unwrap());
    let err = b.column_by_name("err").unwrap().as_any().downcast_ref::<BooleanArray>().unwrap();
    let c = |n: &str| col_str(b, n);
    let (s, eng, role, typ, sid, ts, tsth, text, kind, cid, name, body, red) =
        (c("s"), c("engine"), c("role"), c("type"), c("sid"), c("ts"), c("ts_th"), c("text"), c("kind"), c("call_id"), c("name"),
         c("body"), c("red"));
    let raw = b.column_by_name("file").is_some();
    let prefix = format!("{agent}:");
    for i in 0..b.num_rows() {
        let hv = h.value(i);
        let first = if raw {
            let (f, l) = (c("file").value(i).to_string(), b.column_by_name("line").unwrap().as_any().downcast_ref::<Int64Array>().unwrap().value(i));
            let e = srcs.entry(hv).or_default().entry(f.clone()).or_insert(l);
            if l < *e {
                *e = l;
            }
            (f, l)
        } else {
            // an existing month row: its sources list holds every copy; its own copy is the lowest (file, line)
            let list = b.column_by_name("sources").unwrap().as_any().downcast_ref::<arrow::array::ListArray>().unwrap().value(i);
            let list = list.as_any().downcast_ref::<StringArray>().unwrap();
            let mut best: Option<(String, i64)> = None;
            for j in 0..list.len() {
                let v = list.value(j);
                let Some(rest) = v.strip_prefix(&prefix) else { continue };
                let Some((f, l)) = rest.rsplit_once(':') else { continue };
                let l: i64 = l.parse().unwrap_or(i64::MAX);
                let e = srcs.entry(hv).or_default().entry(f.to_string()).or_insert(l);
                if l < *e {
                    *e = l;
                }
                if best.as_ref().is_none_or(|b| (f, l) < (b.0.as_str(), b.1)) {
                    best = Some((f.to_string(), l));
                }
            }
            best.unwrap_or((String::new(), i64::MAX))
        };
        out.push(Cand {
            s: s.value(i).to_string(), h: hv, idx: idx.value(i), engine: s_opt(eng, i), role: s_opt(role, i), typ: s_opt(typ, i),
            sid: s_opt(sid, i), ts: s_opt(ts, i), ts_th: s_opt(tsth, i), text: s_opt(text, i), kind: s_opt(kind, i),
            call_id: s_opt(cid, i), name: s_opt(name, i), body: s_opt(body, i), err: if err.is_null(i) { None } else { Some(err.value(i)) },
            red: s_opt(red, i), first,
        });
    }
}

/// Fold one month: the existing out/<month>.parquet (if any) + this run's raw parts -> out/<month>.parquet.
/// Returns (rows, new copies read).
pub fn merge_month(parts_dir: &Path, month: &str, out: &Path, agent: &str) -> (usize, usize) {
    let mut cands: Vec<Cand> = vec![];
    let mut srcs: HashMap<i64, BTreeMap<String, i64>> = HashMap::new();
    let existing = out.join(format!("{month}.parquet"));
    if existing.exists() {
        let r = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(fs::File::open(&existing).unwrap())
            .unwrap().build().unwrap();
        for b in r {
            cands_from(&b.unwrap(), &mut srcs, agent, &mut cands);
        }
    }
    let had = cands.len();
    if let Ok(rd) = fs::read_dir(parts_dir.join(month)) {
        for e in rd.flatten() {
            for b in FileReader::try_new(fs::File::open(e.path()).unwrap(), None).unwrap() {
                cands_from(&b.unwrap(), &mut srcs, agent, &mut cands);
            }
        }
    }
    let copies = cands.len() - had;
    // first copy wins: lowest (file, line) per (s, h, idx)
    let mut keep: HashMap<(String, i64, i64), usize> = HashMap::with_capacity(cands.len());
    for (i, c) in cands.iter().enumerate() {
        let k = (c.s.clone(), c.h, c.idx);
        match keep.get(&k) {
            Some(&o) if cands[o].first <= c.first => {}
            _ => {
                keep.insert(k, i);
            }
        }
    }
    let mut order: Vec<usize> = keep.into_values().collect();
    order.sort_by(|&a, &b| {
        let (x, y) = (&cands[a], &cands[b]);
        (x.ts.as_deref(), x.s.as_str(), x.h, x.idx).cmp(&(y.ts.as_deref(), y.s.as_str(), y.h, y.idx))
    });
    let strs = |f: &dyn Fn(&Cand) -> Option<&str>| -> ArrayRef { Arc::new(order.iter().map(|&i| f(&cands[i])).collect::<StringArray>()) };
    let mut hb = Int64Builder::new();
    let mut ib = Int64Builder::new();
    let mut eb = BooleanBuilder::new();
    let mut machines = ListBuilder::new(StringBuilder::new());
    let mut sources = ListBuilder::new(StringBuilder::new());
    for &i in &order {
        let c = &cands[i];
        hb.append_value(c.h);
        ib.append_value(c.idx);
        eb.append_option(c.err);
        machines.values().append_value(agent);
        machines.append(true);
        for (f, l) in &srcs[&c.h] {
            sources.values().append_value(format!("{agent}:{f}:{l}"));
        }
        sources.append(true);
    }
    let u = |n: &str| Field::new(n, DataType::Utf8, true);
    let list = |n: &str| Field::new(n, DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))), true);
    let schema = Arc::new(Schema::new(vec![
        u("s"), Field::new("h", DataType::Int64, false), Field::new("idx", DataType::Int64, false), u("engine"), u("role"),
        u("type"), u("sid"), u("ts"), u("ts_th"), u("text"), u("kind"), u("call_id"), u("name"), u("body"),
        Field::new("err", DataType::Boolean, true), u("red"), list("machines"), list("sources"),
    ]));
    let cols: Vec<ArrayRef> = vec![
        strs(&|c| Some(c.s.as_str())), Arc::new(hb.finish()), Arc::new(ib.finish()), strs(&|c| c.engine.as_deref()),
        strs(&|c| c.role.as_deref()), strs(&|c| c.typ.as_deref()), strs(&|c| c.sid.as_deref()), strs(&|c| c.ts.as_deref()),
        strs(&|c| c.ts_th.as_deref()), strs(&|c| c.text.as_deref()), strs(&|c| c.kind.as_deref()), strs(&|c| c.call_id.as_deref()),
        strs(&|c| c.name.as_deref()), strs(&|c| c.body.as_deref()), Arc::new(eb.finish()), strs(&|c| c.red.as_deref()),
        Arc::new(machines.finish()), Arc::new(sources.finish()),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols).unwrap();
    fs::create_dir_all(out).unwrap();
    let tmp = out.join(format!("{month}.parquet.part"));
    let props = WriterProperties::builder().set_compression(Compression::ZSTD(ZstdLevel::try_new(1).unwrap())).build();
    let mut w = ArrowWriter::try_new(fs::File::create(&tmp).unwrap(), schema, Some(props)).unwrap();
    w.write(&batch).unwrap();
    w.into_inner().unwrap().sync_all().unwrap(); // the month is on disk before _state.tsv can point past it
    fs::rename(&tmp, &existing).unwrap(); // atomic: readers see the old month or the new one, never half
    (order.len(), copies)
}
