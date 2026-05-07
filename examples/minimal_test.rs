use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/tmp/minimal.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("Test", |row, col| {
        if row == 0 {
            match col {
                0 => CellData::number(0.0),
                1 => CellData::number(1.5),
                2 => CellData::text("Test"),
                _ => CellData::blank(),
            }
        } else {
            CellData::blank()
        }
    }, 10, 4).unwrap();
    
    writer.close().unwrap();
    println!("Created: {}", path.display());
}
