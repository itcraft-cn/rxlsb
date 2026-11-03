use std::time::Instant;

use rxlsb::{CellData, XlsbWriter};

const ROWS: usize = 8000;
const COLS: usize = 8;

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
fn perf_batch_write() {
    let data = build();
    let start = Instant::now();
    let mut w = XlsbWriter::builder()
        .path("/tmp/perf_batch_write.xlsb")
        .compression_level(1)
        .build()
        .unwrap();
    w.write_batch("Perf", |r: usize, c: usize| cell(&data, r, c), ROWS, COLS)
        .unwrap();
    w.close().unwrap();
    report("batch", start.elapsed());
}

#[test]
#[ignore = "perf profiling only; run with --ignored"]
fn perf_stream_write() {
    let data = build();
    let start = Instant::now();
    let mut w = XlsbWriter::builder()
        .path("/tmp/perf_stream_write.xlsb")
        .compression_level(1)
        .build()
        .unwrap();
    w.start_sheet("Perf", COLS).unwrap();
    for chunk in 0..ROWS.div_ceil(1000) {
        let start_row = chunk * 1000;
        let rows = (ROWS - start_row).min(1000);
        w.write_rows(
            |r: usize, c: usize| cell(&data, start_row + r, c),
            start_row,
            rows,
        )
        .unwrap();
    }
    w.end_sheet().unwrap();
    w.close().unwrap();
    report("stream", start.elapsed());
}
