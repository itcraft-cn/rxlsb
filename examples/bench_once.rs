use rxlsb::{CellData, XlsbWriter};
use std::path::PathBuf;

const ROWS: usize = 5000;
const COLS: usize = 7;
const TAIL: &str = " z polskimi znakami: ąęśćńźółĄĘŚĆŃŹÓŁ oraz dłuższy tekst testowy.";

fn headers() -> Vec<String> {
    ["ID", "Name", "Count", "Score", "Date", "Active", "Description"]
        .iter().map(|s| s.to_string()).collect()
}

fn main() {
    let level: i32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(6);
    let h = headers();
    let data: Vec<Vec<String>> = (0..ROWS)
        .map(|i| (0..COLS).map(|c| match c {
            0 | 2 => (i as i64).to_string(),
            1 => format!("Produkt {i} żółć"),
            3 => format!("{}", ((i * 104729) % 100000) as f64 / 1000.0),
            4 => format!("{}", 1705320000 + i as i64),
            5 => format!("{}", i % 2 == 0),
            _ => format!("Opis produktu {i}{TAIL}"),
        }).collect())
        .collect();

    let path = PathBuf::from("/tmp/bench_once.xlsb");
    let mut w = XlsbWriter::builder().path(&path).compression_level(level).build().unwrap();
    w.write_batch("Benchmark", |r: usize, c: usize| {
        if r == 0 {
            return CellData::text(h[c].clone());
        }
        let cell = &data[r - 1][c];
        match c {
            0 | 2 => CellData::Number(cell.parse::<i64>().unwrap() as f64),
            3 => CellData::Number(cell.parse::<f64>().unwrap()),
            5 => CellData::Bool(cell == "true"),
            _ => CellData::text(cell.clone()),
        }
    }, ROWS + 1, COLS).unwrap();
    w.close().unwrap();
    eprintln!("done size={}", std::fs::metadata(&path).unwrap().len());
}
