use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/tmp/pure_numeric_rxlsb.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    writer.write_batch("Test", |_row, _col| CellData::number(123.45), 3, 2).unwrap();
    writer.close().unwrap();
    println!("Created: {}", path.display());
}
