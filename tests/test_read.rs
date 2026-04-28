use rxlsb::{XlsbReader, CellData};
use tempfile::NamedTempFile;
use std::fs;

#[test]
fn test_read_after_write() {
    let temp = NamedTempFile::new().unwrap();
    
    let mut writer = rxlsb::XlsbWriter::builder()
        .path(temp.path())
        .build()
        .unwrap();
    
    writer.write_batch("TestSheet", |row, col| {
        match col {
            0 => CellData::text(format!("Row{}", row)),
            1 => CellData::number(row as f64),
            _ => CellData::blank(),
        }
    }, 50, 2).unwrap();
    
    writer.close().unwrap();
    
    let mut reader = XlsbReader::builder()
        .path(temp.path())
        .build()
        .unwrap();
    
    let infos = reader.get_sheet_infos();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].name, "TestSheet");
    
    let mut row_count = 0;
    reader.for_each_row(0, |idx, cells: &[rxlsb::CellData]| {
        row_count += 1;
        assert_eq!(cells.len(), 2);
    }).unwrap();
    
    assert_eq!(row_count, 50);
}

#[test]
fn test_roundtrip() {
    let temp1 = NamedTempFile::new().unwrap();
    let temp2 = NamedTempFile::new().unwrap();
    
    let mut writer = rxlsb::XlsbWriter::builder()
        .path(temp1.path())
        .build()
        .unwrap();
    
    writer.write_batch("Data", |row, col| {
        CellData::number(row as f64 * 100.0 + col as f64)
    }, 100, 5).unwrap();
    
    writer.close().unwrap();
    
    fs::copy(temp1.path(), temp2.path()).unwrap();
    
    let mut reader = XlsbReader::builder()
        .path(temp2.path())
        .build()
        .unwrap();
    
    let mut row_count = 0;
    reader.for_each_row(0, |_idx, cells: &[rxlsb::CellData]| {
        row_count += 1;
        assert_eq!(cells.len(), 5);
    }).unwrap();
    
    assert_eq!(row_count, 100);
}