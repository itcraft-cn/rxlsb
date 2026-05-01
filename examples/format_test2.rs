use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/tmp/format_test2.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("Formats2", |row: usize, col: usize| {
        let value = row as f64 * 123.456 - 500.0;
        let timestamp: i64 = 1714560000 + (row as i64) * 86400;
        
        match col {
            0 => CellData::date_from_timestamp(timestamp),
            1 => CellData::time(timestamp),
            2 => CellData::currency(value),
            3 => CellData::number_negative_red(value),
            _ => CellData::blank(),
        }
    }, 10, 4).unwrap();
    
    writer.close().unwrap();
    println!("Generated {}", path.display());
}