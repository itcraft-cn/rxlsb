use rxlsb::{XlsbWriter, XlsbReader, CellData};
use tempfile::NamedTempFile;

const TEST_FILE: &str = "/tmp/rxlsb_features_test.xlsb";

fn setup_test_file() {
    let mut writer = XlsbWriter::builder()
        .path(TEST_FILE)
        .build()
        .unwrap();
    
    writer.write_batch("普通写Sheet", |row, col| {
        match col {
            0 => CellData::text(format!("Name_{}", row)),
            1 => CellData::number(row as f64 * 100.0),
            2 => CellData::bool(row % 2 == 0),
            3 => CellData::blank(),
            _ => CellData::blank(),
        }
    }, 20, 4).unwrap();
    
    writer.close().unwrap();
}

#[test]
fn test_普通写() {
    let path = "/tmp/test_normal_write.xlsb";
    
    let mut writer = XlsbWriter::builder()
        .path(path)
        .build()
        .unwrap();
    
    writer.write_batch("普通写", |row: usize, col: usize| {
        CellData::text(format!("R{}C{}", row, col))
    }, 10, 5).unwrap();
    
    writer.close().unwrap();
    
    let mut reader = XlsbReader::builder()
        .path(path)
        .build()
        .unwrap();
    
    let rows = reader.read_rows(0, 0, 10).unwrap();
    assert_eq!(rows.len(), 10);
}

#[test]
fn test_分页写() {
    let path = "/tmp/test_pagination_write.xlsb";
    
    let mut writer = XlsbWriter::builder()
        .path(path)
        .build()
        .unwrap();
    
    writer.start_sheet("分页写", 3).unwrap();
    
    writer.write_rows(|row: usize, col: usize| {
        CellData::text(format!("R{}C{}", row, col))
    }, 0, 5).unwrap();
    
    writer.end_sheet().unwrap();
    writer.close().unwrap();
    
    let mut reader = XlsbReader::builder()
        .path(path)
        .build()
        .unwrap();
    
    let rows = reader.read_rows(0, 0, 5).unwrap();
    assert_eq!(rows.len(), 5);
}

#[test]
fn test_流式写() {
    let path = "/tmp/test_streaming_write.xlsb";
    
    let mut writer = XlsbWriter::builder()
        .path(path)
        .build()
        .unwrap();
    
    writer.start_sheet("流式写", 4).unwrap();
    
    writer.write_rows(|row: usize, col: usize| {
        CellData::number(row as f64 + col as f64)
    }, 0, 30).unwrap();
    
    writer.end_sheet().unwrap();
    writer.close().unwrap();
    
    let mut reader = XlsbReader::builder()
        .path(path)
        .build()
        .unwrap();
    
    let mut count = 0;
    reader.for_each_row(0, |_idx, cells: &[CellData]| {
        count += 1;
        assert_eq!(cells.len(), 4);
    }).unwrap();
    
    assert_eq!(count, 30);
}

#[test]
fn test_普通读() {
    setup_test_file();
    
    let mut reader = XlsbReader::builder()
        .path(TEST_FILE)
        .build()
        .unwrap();
    
    let infos = reader.get_sheet_infos();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].name, "普通写Sheet");
    
    let mut row_count = 0;
    reader.for_each_row(0, |idx, cells: &[CellData]| {
        row_count += 1;
        assert_eq!(cells.len(), 4);
        if idx == 0 {
            assert!(matches!(cells[0], CellData::Text(_)));
        }
    }).unwrap();
    
    assert_eq!(row_count, 20);
}

#[test]
fn test_分页读() {
    setup_test_file();
    
    let mut reader = XlsbReader::builder()
        .path(TEST_FILE)
        .build()
        .unwrap();
    
    let page1 = reader.read_rows(0, 0, 5).unwrap();
    assert_eq!(page1.len(), 5);
    
    let page2 = reader.read_rows(0, 10, 5).unwrap();
    assert_eq!(page2.len(), 5);
    
    let page3 = reader.read_rows(0, 15, 5).unwrap();
    assert_eq!(page3.len(), 5);
    
    assert!(matches!(page1[0][0], CellData::Text(ref s) if s == "Name_0"));
}

#[test]
fn test_流式读() {
    setup_test_file();
    
    let mut reader = XlsbReader::builder()
        .path(TEST_FILE)
        .build()
        .unwrap();
    
    let mut collected: Vec<Vec<CellData>> = vec![];
    
    reader.for_each_row(0, |_idx, cells: &[CellData]| {
        collected.push(cells.to_vec());
    }).unwrap();
    
    assert_eq!(collected.len(), 20);
    assert_eq!(collected[0].len(), 4);
}