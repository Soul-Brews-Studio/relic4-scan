//! Raw rows spilled to Arrow parts while the parse runs (one folder per month, one file per thread and spill).

use super::*;


pub(super) fn raw_schema() -> Arc<Schema> {
    let u = |n: &str| Field::new(n, DataType::Utf8, true);
    Arc::new(Schema::new(vec![
        u("s"), Field::new("h", DataType::Int64, false), Field::new("idx", DataType::Int64, false), u("engine"), u("role"),
        u("type"), u("sid"), u("ts"), u("ts_th"), u("text"), u("kind"), u("call_id"), u("name"), u("body"),
        Field::new("err", DataType::Boolean, true), u("red"), u("file"), Field::new("line", DataType::Int64, false),
        u("model"), u("msg_id"), Field::new("out_tokens", DataType::Int64, true),
        Field::new("in_tokens", DataType::Int64, true), Field::new("cache_read", DataType::Int64, true), Field::new("cache_write", DataType::Int64, true),
    ]))
}

pub(super) fn raw_batch(rows: &[Raw]) -> RecordBatch {
    let st = |f: &dyn Fn(&Raw) -> Option<&str>| -> ArrayRef { Arc::new(rows.iter().map(f).collect::<StringArray>()) };
    let cols: Vec<ArrayRef> = vec![
        st(&|r| Some(r.s)), Arc::new(rows.iter().map(|r| r.h).collect::<Int64Array>()),
        Arc::new(rows.iter().map(|r| r.idx).collect::<Int64Array>()), st(&|r| Some(r.engine)), st(&|r| r.role.as_deref()),
        st(&|r| r.typ.as_deref()), st(&|r| Some(r.sid.as_str())), st(&|r| Some(r.ts.as_str())), st(&|r| Some(r.ts_th.as_str())),
        st(&|r| r.text.as_deref()), st(&|r| r.kind), st(&|r| r.call_id.as_deref()), st(&|r| r.name.as_deref()),
        st(&|r| r.body.as_deref()), Arc::new(rows.iter().map(|r| r.err).collect::<BooleanArray>()), st(&|r| r.red.as_deref()),
        st(&|r| Some(&*r.file)), Arc::new(rows.iter().map(|r| r.line).collect::<Int64Array>()),
        st(&|r| r.model.as_deref()), st(&|r| r.msg_id.as_deref()), Arc::new(rows.iter().map(|r| r.out_tokens).collect::<Int64Array>()),
        Arc::new(rows.iter().map(|r| r.in_tokens).collect::<Int64Array>()), Arc::new(rows.iter().map(|r| r.cache_read).collect::<Int64Array>()),
        Arc::new(rows.iter().map(|r| r.cache_write).collect::<Int64Array>()),
    ];
    RecordBatch::try_new(raw_schema(), cols).expect("raw batch")
}

pub(super) struct Spiller {
    pub(super) dir: PathBuf,
    pub(super) tid: usize,
    pub(super) n: usize,
    pub(super) buf: HashMap<String, Vec<Raw>>,
}

impl Spiller {
    pub(super) fn push(&mut self, r: Raw) {
        let full = match self.buf.get_mut(&r.month) {
            Some(v) => {
                v.push(r);
                (v.len() >= SPILL_ROWS).then(|| std::mem::take(v))
            }
            None => {
                self.buf.insert(r.month.clone(), vec![r]);
                None
            }
        };
        if let Some(rows) = full {
            let m = rows[0].month.clone();
            self.write(&m, &rows);
        }
    }
    pub(super) fn write(&mut self, month: &str, rows: &[Raw]) {
        let d = self.dir.join(month);
        fs::create_dir_all(&d).unwrap();
        let p = d.join(format!("t{:02}-{:05}.arrow", self.tid, self.n));
        self.n += 1;
        let f = fs::File::create(&p).unwrap();
        let mut w = FileWriter::try_new(f, &raw_schema()).unwrap();
        w.write(&raw_batch(rows)).unwrap();
        w.finish().unwrap();
    }
    pub(super) fn flush(&mut self) {
        let all: Vec<(String, Vec<Raw>)> = self.buf.drain().collect();
        for (m, rows) in all {
            if !rows.is_empty() {
                self.write(&m, &rows);
            }
        }
    }
}
