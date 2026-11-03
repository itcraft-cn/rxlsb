use std::path::PathBuf;

use rxlsb::{CellData, XlsbReader, XlsbWriter};

fn main() {
    let cols: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);
    let rows = 3usize;
    let path = PathBuf::from(format!("/tmp/wide_{cols}.xlsb"));

    let mut w = XlsbWriter::builder().path(&path).build().unwrap();
    w.write_batch(
        "Wide",
        |r: usize, c: usize| {
            if c % 500 == 0 {
                CellData::text(format!("R{r}C{c}"))
            } else {
                CellData::Number((r * cols + c) as f64)
            }
        },
        rows,
        cols,
    )
    .unwrap();
    w.close().unwrap();

    let mut reader = XlsbReader::builder().path(&path).build().unwrap();
    let read = reader.read_rows(0, 0, rows).unwrap();
    println!(
        "cols={cols} rows={} cells_in_row0={} size={}",
        read.len(),
        read.first().map(|r| r.len()).unwrap_or(0),
        std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
    );
    println!("Created: {}", path.display());
}
