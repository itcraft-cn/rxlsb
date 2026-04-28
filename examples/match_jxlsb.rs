use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/tmp/match_jxlsb.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    writer.write_batch("Test", |row, col| {
        if col == 0 { CellData::number((row + 1) as f64) }
        else if col == 1 { CellData::text(format!("Name{}", row)) }
        else { CellData::number(row as f64 * 100.5) }
    }, 10, 3).unwrap();
    writer.close().unwrap();
    println!("Created rxlsb: {}", path.display());
}
