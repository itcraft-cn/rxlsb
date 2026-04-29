use rxlsb::{XlsbWriter, XlsbReader, CellData};

#[test]
fn test_streaming_write() {
    let mut writer = XlsbWriter::builder()
        .path("/tmp/streaming_test.xlsb")
        .build()
        .unwrap();
    
    writer.start_sheet("Sheet1", 3).unwrap();
    
    writer.write_rows(|row, col| {
        match col {
            0 => CellData::text(format!("Name_{}", row)),
            1 => CellData::number(row as f64 * 100.0),
            2 => CellData::bool(row % 2 == 0),
            _ => CellData::blank(),
        }
    }, 0, 5).unwrap();
    
    writer.write_rows(|row, col| {
        match col {
            0 => CellData::text(format!("Name_{}", row)),
            1 => CellData::number(row as f64 * 100.0),
            2 => CellData::bool(row % 2 == 0),
            _ => CellData::blank(),
        }
    }, 5, 5).unwrap();
    
    writer.end_sheet().unwrap();
    writer.close().unwrap();
    
    let mut reader = XlsbReader::builder()
        .path("/tmp/streaming_test.xlsb")
        .build()
        .unwrap();
    
    let rows = reader.read_rows(0, 0, 3).unwrap();
    assert_eq!(rows.len(), 3);
    
    let rows = reader.read_rows(0, 5, 3).unwrap();
    assert_eq!(rows.len(), 3);
}

#[test]
fn test_pagination_read() {
    let mut writer = XlsbWriter::builder()
        .path("/tmp/pagination_test.xlsb")
        .build()
        .unwrap();
    
    writer.write_batch("Data", |row, col| {
        CellData::text(format!("R{}C{}", row, col))
    }, 20, 5).unwrap();
    
    writer.close().unwrap();
    
    let mut reader = XlsbReader::builder()
        .path("/tmp/pagination_test.xlsb")
        .build()
        .unwrap();
    
    let page1 = reader.read_rows(0, 0, 5).unwrap();
    assert_eq!(page1.len(), 5);
    
    let page2 = reader.read_rows(0, 10, 5).unwrap();
    assert_eq!(page2.len(), 5);
}