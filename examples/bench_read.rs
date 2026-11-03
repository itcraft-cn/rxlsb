use std::path::PathBuf;
use std::time::Instant;

use rxlsb::{CellData, XlsbReader, XlsbWriter};

const ROWS: usize = 8000;
const COLS: usize = 8;
const ITERS: usize = 21;
const FILE: &str = "/tmp/bench_read.xlsb";

fn build() -> Vec<Vec<String>> {
    (0..ROWS)
        .map(|i| {
            (0..COLS)
                .map(|c| match c {
                    0 => format!("{}", i),
                    1 => format!("Produkt {i} żółć"),
                    2 => format!("{}", (i * 7919) % 10000),
                    3 => format!("{:.3}", ((i * 104729) % 100000) as f64 / 1000.0),
                    4 => format!("Opis produktu {i} z polskimi znakami: ąęśćńźół oraz dłuższy tekst."),
                    5 => format!("{}", i % 2 == 0),
                    6 => format!("C{c}-{i} żółć"),
                    _ => format!("{:.2}", ((i * (c + 3)) % 100000) as f64 / 100.0),
                })
                .collect()
        })
        .collect()
}

fn cell(data: &[Vec<String>], r: usize, c: usize) -> CellData {
    let s = &data[r][c];
    match c {
        0 | 2 => CellData::Number(s.parse::<i64>().unwrap() as f64),
        3 | 7 => CellData::Number(s.parse::<f64>().unwrap()),
        5 => CellData::Bool(s == "true"),
        _ => CellData::text(s.clone()),
    }
}

fn ensure_file() {
    if PathBuf::from(FILE).exists() {
        return;
    }
    let data = build();
    let mut w = XlsbWriter::builder().path(FILE).compression_level(1).build().unwrap();
    w.write_batch("Perf", |r: usize, c: usize| cell(&data, r, c), ROWS, COLS)
        .unwrap();
    w.close().unwrap();
}

fn run_read_rows() -> (f64, usize) {
    let t = Instant::now();
    let mut reader = XlsbReader::builder().path(FILE).build().unwrap();
    let rows = reader.read_rows(0, 0, ROWS).unwrap();
    let n = rows.len();
    (t.elapsed().as_secs_f64() * 1000.0, n)
}

fn run_for_each() -> (f64, usize) {
    let t = Instant::now();
    let mut reader = XlsbReader::builder().path(FILE).build().unwrap();
    let mut n = 0usize;
    reader
        .for_each_row(0, |_idx, _cells: &[CellData]| {
            n += 1;
        })
        .unwrap();
    (t.elapsed().as_secs_f64() * 1000.0, n)
}

fn run_open() -> f64 {
    let t = Instant::now();
    let reader = XlsbReader::builder().path(FILE).build().unwrap();
    std::hint::black_box(&reader);
    t.elapsed().as_secs_f64() * 1000.0
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    ensure_file();

    // warmup
    let _ = run_read_rows();
    let _ = run_for_each();
    let _ = run_open();

    // 交错测量，降低热节流带来的顺序偏差
    let mut batch = Vec::with_capacity(ITERS);
    let mut stream = Vec::with_capacity(ITERS);
    let mut open = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        batch.push(run_read_rows().0);
        stream.push(run_for_each().0);
        open.push(run_open());
    }

    let (_, rows) = run_read_rows();
    let min = |v: &[f64]| v.iter().cloned().fold(f64::INFINITY, f64::min);
    println!(
        "RESULT open_min={:.4} read_rows_min={:.4} for_each_min={:.4} open_med={:.4} read_rows_med={:.4} for_each_med={:.4} rows={rows}",
        min(&open),
        min(&batch),
        min(&stream),
        median(open),
        median(batch),
        median(stream)
    );
}
