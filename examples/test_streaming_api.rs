use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    println!("=== 流式API测试 (start/write/end) ===\n");
    
    // 生成和test_stream_api.xlsb相同数据，但用流式API
    let path = PathBuf::from("/tmp/stream_api_test.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    // 流式API：start_sheet + write_rows + end_sheet
    writer.start_sheet("StreamSheet", 4).unwrap();
    
    writer.write_rows(|row, col| {
        match col {
            0 => CellData::number(row as f64),          // A: 0,1,2,...,99
            1 => CellData::number(row as f64 * 2.5),    // B: 0,2.5,5,...,247.5
            2 => CellData::text(format!("Row-{}", row)), // C: Row-0, Row-1,...
            3 => CellData::bool(row % 3 == 0),          // D: TRUE/FALSE交替
            _ => CellData::blank(),
        }
    }, 0, 100).unwrap();
    
    writer.end_sheet().unwrap();
    writer.close().unwrap();
    
    println!("Created: {} (100 rows, 4 cols)", path.display());
    println!("使用API: start_sheet → write_rows → end_sheet");
    
    // 对照：write_batch生成的文件
    let path2 = PathBuf::from("/tmp/batch_api_test.xlsb");
    let mut writer2 = XlsbWriter::builder().path(&path2).build().unwrap();
    
    writer2.write_batch("BatchSheet", |row, col| {
        match col {
            0 => CellData::number(row as f64),
            1 => CellData::number(row as f64 * 2.5),
            2 => CellData::text(format!("Row-{}", row)),
            3 => CellData::bool(row % 3 == 0),
            _ => CellData::blank(),
        }
    }, 100, 4).unwrap();
    
    writer2.close().unwrap();
    println!("Created: {} (100 rows, 4 cols)", path2.display());
    println!("使用API: write_batch");
    
    // 对比hex
    println!("\n对比两个API生成的sheet1.bin...");
}
