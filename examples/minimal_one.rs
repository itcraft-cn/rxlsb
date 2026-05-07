use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/tmp/minimal_one.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("Test", |_row, _col| {
        CellData::number(0.0)  // row=0, col=0, value=0.0
    }, 1, 1).unwrap();  // 1行1列
    
    writer.close().unwrap();
    println!("Created: {} (1 row, 1 col)", path.display());
}
