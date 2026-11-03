use std::path::PathBuf;
use std::time::Instant;

use rxlsb::{CellData, XlsbReader, XlsbWriter};

const ROWS: usize = 12000;
const COLS: usize = 8;
const FILE: &str = "/tmp/perf_read.xlsb";

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

fn ensure_file() {
    if PathBuf::from(FILE).exists() {
        return;
    }
    let data = build();
    let mut w = XlsbWriter::builder().path(FILE).compression_level(1).build().unwrap();
    w.write_batch(
        "Perf",
        |r: usize, c: usize| {
            let s = &data[r][c];
            match c {
                0 | 2 => CellData::Number(s.parse::<i64>().unwrap() as f64),
                3 | 7 => CellData::Number(s.parse::<f64>().unwrap()),
                5 => CellData::Bool(s == "true"),
                _ => CellData::text(s.clone()),
            }
        },
        ROWS,
        COLS,
    )
    .unwrap();
    w.close().unwrap();
}

fn report(name: &str, elapsed: std::time::Duration) {
    let cells = (ROWS * COLS) as f64;
    eprintln!(
        "{name}: {:?}, {:.0} cells/s, {:.1} ns/cell",
        elapsed,
        cells / elapsed.as_secs_f64(),
        elapsed.as_nanos() as f64 / cells
    );
}

#[test]
#[ignore = "perf profiling only; run with --ignored"]
fn perf_read_rows() {
    ensure_file();
    let t = Instant::now();
    let mut reader = XlsbReader::builder().path(FILE).build().unwrap();
    let rows = reader.read_rows(0, 0, ROWS).unwrap();
    std::hint::black_box(rows.len());
    report("read_rows", t.elapsed());
}

#[test]
#[ignore = "perf profiling only; run with --ignored"]
fn perf_read_stream() {
    ensure_file();
    let t = Instant::now();
    let mut reader = XlsbReader::builder().path(FILE).build().unwrap();
    let mut n = 0usize;
    reader
        .for_each_row(0, |_idx, _cells: &[CellData]| {
            n += 1;
        })
        .unwrap();
    std::hint::black_box(n);
    report("for_each_row", t.elapsed());
}
