use rxlsb::{CellData, XlsbWriter};
use std::path::PathBuf;
use std::time::Instant;

const ROWS: usize = 5000;
const COLS: usize = 7;

fn headers() -> Vec<String> {
    ["ID", "Name", "Count", "Score", "Date", "Active", "Description"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[derive(Clone)]
enum V {
    Int(i64),
    Num(f64),
    Text(String),
    Bool(bool),
    Date(chrono::DateTime<chrono::Utc>),
}

const TAIL: &str = " z polskimi znakami: ąęśćńźółĄĘŚĆŃŹÓŁ oraz dłuższy tekst testowy.";

fn build() -> Vec<Vec<V>> {
    (0..ROWS)
        .map(|i| {
            (0..COLS)
                .map(|c| match c {
                    0 => V::Int(i as i64),
                    1 => V::Text(format!("Produkt {i} żółć")),
                    2 => V::Int(((i * 7919) % 10000) as i64),
                    3 => V::Num(((i * 104729) % 100000) as f64 / 1000.0),
                    4 => V::Date(chrono::TimeZone::timestamp_opt(&chrono::Utc, 1705320000 + i as i64, 0).unwrap()),
                    5 => V::Bool(i % 2 == 0),
                    _ => V::Text(format!("Opis produktu {i}{TAIL}")),
                })
                .collect()
        })
        .collect()
}

fn to_cell(row: usize, col: usize, h: &[String], data: &[Vec<V>]) -> CellData {
    if row == 0 {
        return CellData::text(h[col].clone());
    }
    match &data[row - 1][col] {
        V::Int(v) => CellData::Number(*v as f64),
        V::Num(v) => CellData::Number(*v),
        V::Text(v) => CellData::text(v.clone()),
        V::Bool(v) => CellData::Bool(*v),
        V::Date(v) => CellData::date(*v),
    }
}

fn run(level: i32, h: &[String], data: &[Vec<V>], label: &str) {
    let path = PathBuf::from(format!("/tmp/bench_phases_{level}.xlsb"));

    // Phase A: closure only -> build Vec<CellData> (isolates clone cost)
    let t = Instant::now();
    let mut cache: Vec<Vec<CellData>> = Vec::with_capacity(ROWS + 1);
    for r in 0..=ROWS {
        let mut row = Vec::with_capacity(COLS);
        for c in 0..COLS {
            row.push(to_cell(r, c, h, data));
        }
        cache.push(row);
    }
    let phase_a = t.elapsed();

    // Phase B: write_batch (sheet build, no zip) using cached cells
    let t = Instant::now();
    let mut w = XlsbWriter::builder().path(&path).compression_level(level).build().unwrap();
    w.write_batch("Benchmark", |r: usize, c: usize| cache[r][c].clone(), ROWS + 1, COLS).unwrap();
    let phase_b = t.elapsed();

    // Phase C: close (zip compression + file write)
    let t = Instant::now();
    w.close().unwrap();
    let phase_c = t.elapsed();

    // Phase D: full from-scratch write via closure (like the harness)
    let path2 = PathBuf::from(format!("/tmp/bench_phases_full_{level}.xlsb"));
    let t = Instant::now();
    let mut w2 = XlsbWriter::builder().path(&path2).compression_level(level).build().unwrap();
    w2.write_batch("Benchmark", |r, c| to_cell(r, c, h, data), ROWS + 1, COLS).unwrap();
    w2.close().unwrap();
    let phase_d = t.elapsed();

    let size = std::fs::metadata(&path2).unwrap().len();
    println!(
        "{label} level={level}: A(clone)={:?} B(sheet)={:?} C(zip)={:?} D(full)={:?} size={}",
        phase_a, phase_b, phase_c, phase_d, size
    );
}

fn main() {
    let h = headers();
    let data = build();
    // warmup
    run(1, &h, &data, "warmup");
    run(1, &h, &data, "measured");
    run(6, &h, &data, "measured");
}
