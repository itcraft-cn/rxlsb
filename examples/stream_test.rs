use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/tmp/stream_test.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    writer.write_batch("Sheet1", |row, col| {
        match col {
            0 => CellData::text(format!("Name-{}", row)),
            1 => CellData::text(format!("Cat-{}", row % 100)),
            2 => CellData::text(format!("R-{}", row % 10)),
            3 => CellData::number(row as f64),
            _ => CellData::blank(),
        }
    }, 100, 4).unwrap();
    writer.close().unwrap();
    println!("Generated {}", path.display());
}