use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    // 测试 start_sheet + write_rows + end_sheet API
    let path = PathBuf::from("/tmp/test_stream_api.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.start_sheet("TestSheet", 4).unwrap();
    writer.write_rows(|row, col| {
        match col {
            0 => CellData::number(row as f64),
            1 => CellData::text(format!("Row-{}", row)),
            2 => CellData::bool(row % 2 == 0),
            3 => CellData::number(row as f64 * 100.0),
            _ => CellData::blank(),
        }
    }, 0, 100).unwrap();
    writer.end_sheet().unwrap();
    
    writer.close().unwrap();
    println!("Created: {}", path.display());
}
