use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/tmp/test_wsfmt.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("Test", |row, col| {
        CellData::number(row as f64)
    }, 1, 1).unwrap();
    
    writer.close().unwrap();
    println!("Created: {}", path.display());
}
