use rxlsb::{XlsbWriter, XlsbReader, CellData};
use std::path::PathBuf;

fn main() {
    // 生成测试文件
    let path = PathBuf::from("/tmp/rxlsb_verify.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    writer.write_batch("Test", |row: usize, col: usize| {
        match col {
            0 => CellData::text(format!("Name-{}", row)),
            1 => CellData::text(format!("Cat-{}", row % 100)),
            2 => CellData::text(format!("R-{}", row % 10)),
            3 => CellData::number(row as f64),
            4 => CellData::number(row as f64 * 10.0),
            _ => CellData::blank(),
        }
    }, 5, 5).unwrap();
    writer.close().unwrap();
    
    // 读取验证
    let mut reader = XlsbReader::builder().path(&path).build().unwrap();
    println!("前5行数据:");
    reader.for_each_row(0, |idx: usize, cells: &[CellData]| {
        if idx < 5 {
            println!("Row {}: {:?}", idx, cells);
        }
    }).unwrap();
    
    println!("文件大小: {} bytes", std::fs::metadata(&path).unwrap().len());
}