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
    w.write_batch(
        "Perf",
        |r: usize, c: usize| {
            let cell = &data[r][c];
            match c {
                0 | 2 => CellData::Number(cell.parse::<i64>().unwrap() as f64),
                3 | 7 => CellData::Number(cell.parse::<f64>().unwrap()),
                5 => CellData::Bool(cell == "true"),
                _ => CellData::text(cell.clone()),
            }
        },
        ROWS,
        COLS,
    )
    .unwrap();
    w.close().unwrap();
    let elapsed = start.elapsed();

    let cells = (ROWS * COLS) as f64;
    eprintln!(
        "perf: {:?}, {:.0} cells/s, {:.1} ns/cell",
        elapsed,
        cells / elapsed.as_secs_f64(),
        elapsed.as_nanos() as f64 / cells
    );
}
