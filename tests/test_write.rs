use rxlsb::{XlsbWriter, CellData};
use tempfile::NamedTempFile;

#[test]
fn test_write_simple() {
    let temp = NamedTempFile::new().unwrap();
    
    let mut writer = XlsbWriter::builder()
        .path(temp.path())
        .build()
        .unwrap();
    
    writer.write_batch("Sheet1", |row, col| {
        match col % 3 {
            0 => CellData::text(format!("Item-{}", row)),
            1 => CellData::number(row as f64 * 10.0),
            2 => CellData::bool(row % 2 == 0),
            _ => CellData::blank(),
        }
    }, 100, 3).unwrap();
    
    writer.close().unwrap();
    
    println!("XLSB file created successfully at {}", temp.path().display());
}

#[test]
fn test_write_large() {
    let temp = NamedTempFile::new().unwrap();
    
    let mut writer = XlsbWriter::builder()
        .path(temp.path())
        .build()
        .unwrap();
    
    writer.write_batch("Data", |row, col| {
        CellData::number(row as f64 * col as f64)
    }, 1000, 10).unwrap();
    
    writer.close().unwrap();
    
    println!("Large XLSB file created: 1000 rows x 10 cols");
}