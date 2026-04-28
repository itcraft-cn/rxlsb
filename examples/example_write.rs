use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    // 小文件：10行x5列，混合数据类型
    let path1 = PathBuf::from("test_small.xlsb");
    let mut writer1 = XlsbWriter::builder()
        .path(&path1)
        .build()
        .unwrap();
    
    writer1.write_batch("Sheet1", |row, col| {
        match col {
            0 => CellData::number(row as f64 + 1.0),
            1 => CellData::text(format!("Name{}", row)),
            2 => CellData::number((row as f64) * 100.5),
            3 => CellData::bool(row % 2 == 0),
            4 => CellData::text(format!("Data-{}-{}", row, col)),
            _ => CellData::blank(),
        }
    }, 10, 5).unwrap();
    
    writer1.close().unwrap();
    println!("Created: {} (10 rows x 5 cols, mixed types)", path1.display());
    
    // 中等文件：100行x10列
    let path2 = PathBuf::from("test_medium.xlsb");
    let mut writer2 = XlsbWriter::builder()
        .path(&path2)
        .build()
        .unwrap();
    
    writer2.write_batch("Data", |row, col| {
        CellData::number(row as f64 * 10.0 + col as f64)
    }, 100, 10).unwrap();
    
    writer2.close().unwrap();
    println!("Created: {} (100 rows x 10 cols)", path2.display());
    
    // 大文件：1000行x4列
    let path3 = PathBuf::from("output.xlsb");
    let mut writer3 = XlsbWriter::builder()
        .path(&path3)
        .build()
        .unwrap();
    
    writer3.write_batch("Sheet1", |row, col| {
        CellData::number(row as f64 * 100.0 + col as f64)
    }, 1000, 4).unwrap();
    
    writer3.close().unwrap();
    println!("Created: {} (1000 rows x 4 cols)", path3.display());
}