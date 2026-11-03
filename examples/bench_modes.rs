use std::path::PathBuf;
use std::time::Instant;

use rxlsb::{CellData, XlsbWriter};

const ROWS: usize = 8000;
const COLS: usize = 8;
const ITERS: usize = 9;

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

fn run_batch(data: &[Vec<String>], path: &PathBuf) -> f64 {
    let _ = std::fs::remove_file(path);
    let t = Instant::now();
    let mut w = XlsbWriter::builder().path(path).compression_level(1).build().unwrap();
    w.write_batch("Perf", |r: usize, c: usize| cell(data, r, c), ROWS, COLS)
        .unwrap();
    w.close().unwrap();
    t.elapsed().as_secs_f64() * 1000.0
}

fn run_stream(data: &[Vec<String>], path: &PathBuf) -> f64 {
    let _ = std::fs::remove_file(path);
    let t = Instant::now();
    let mut w = XlsbWriter::builder().path(path).compression_level(1).build().unwrap();
    w.start_sheet("Perf", COLS).unwrap();
    let mut start = 0;
    while start < ROWS {
        let rows = (ROWS - start).min(1000);
        w.write_rows(|r: usize, c: usize| cell(data, start + r, c), start, rows)
            .unwrap();
        start += rows;
    }
    w.end_sheet().unwrap();
    w.close().unwrap();
    t.elapsed().as_secs_f64() * 1000.0
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let data = build();
    let pb = PathBuf::from("/tmp/bench_modes_batch.xlsb");
    let ps = PathBuf::from("/tmp/bench_modes_stream.xlsb");

    // warmup
    run_batch(&data, &pb);
    run_stream(&data, &ps);

    let batch: Vec<f64> = (0..ITERS).map(|_| run_batch(&data, &pb)).collect();
    let stream: Vec<f64> = (0..ITERS).map(|_| run_stream(&data, &ps)).collect();

    let mb = median(batch);
    let ms = median(stream);
    let size = std::fs::metadata(&pb).map(|m| m.len()).unwrap_or(0);
    println!("RESULT batch_ms={mb:.4} stream_ms={ms:.4} size={size}");
}
