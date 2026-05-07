use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    println!("=== 流式API完整测试 ===\n");
    
    // 测试1: 单sheet流式写入
    test_single_sheet();
    
    // 测试2: 多sheet流式写入
    test_multi_sheets();
    
    // 测试3: 分批写入（模拟append）
    test_batch_append();
    
    // 测试4: 大数据量流式写入
    test_large_streaming();
    
    println!("\n=== 所有流式API测试文件生成完成 ===");
    println!("\n文件列表：");
    println!("  stream_single.xlsb    - 单sheet（100行×4列）");
    println!("  stream_multi.xlsb     - 多sheet（3个sheet）");
    println!("  stream_batch.xlsb     - 分批写入（300行分3批）");
    println!("  stream_large.xlsb     - 大数据量（5000行）");
    println!("\n请用WPS验证流式API生成的文件！");
}

fn test_single_sheet() {
    println!("1. 单sheet流式写入");
    
    let path = PathBuf::from("/tmp/stream_single.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.start_sheet("Sheet1", 4).unwrap();
    writer.write_rows(|row, col| {
        match col {
            0 => CellData::number(row as f64),
            1 => CellData::number(row as f64 * 10.0),
            2 => CellData::text(format!("Text-{}", row)),
            3 => CellData::bool(row % 5 == 0),
            _ => CellData::blank(),
        }
    }, 0, 100).unwrap();
    writer.end_sheet().unwrap();
    writer.close().unwrap();
    
    println!("  ✓ Created: stream_single.xlsb (100 rows × 4 cols)");
}

fn test_multi_sheets() {
    println!("2. 多sheet流式写入");
    
    let path = PathBuf::from("/tmp/stream_multi.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    // Sheet1: 数值
    writer.start_sheet("Numbers", 3).unwrap();
    writer.write_rows(|row, col| {
        CellData::number(row as f64 * (col + 1) as f64)
    }, 0, 50).unwrap();
    writer.end_sheet().unwrap();
    
    // Sheet2: 文本
    writer.start_sheet("Texts", 2).unwrap();
    writer.write_rows(|row, col| {
        CellData::text(format!("Sheet2-R{}C{}", row, col))
    }, 0, 30).unwrap();
    writer.end_sheet().unwrap();
    
    // Sheet3: 布尔
    writer.start_sheet("Bools", 4).unwrap();
    writer.write_rows(|row, col| {
        CellData::bool((row + col) % 2 == 0)
    }, 0, 20).unwrap();
    writer.end_sheet().unwrap();
    
    writer.close().unwrap();
    
    println!("  ✓ Created: stream_multi.xlsb (3 sheets)");
    println!("    Sheet1: Numbers (50 rows × 3 cols)");
    println!("    Sheet2: Texts (30 rows × 2 cols)");
    println!("    Sheet3: Bools (20 rows × 4 cols)");
}

fn test_batch_append() {
    println!("3. 分批写入（模拟append）");
    
    let path = PathBuf::from("/tmp/stream_batch.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.start_sheet("BatchData", 5).unwrap();
    
    // 第1批：0-100行
    writer.write_rows(|row, col| {
        CellData::number(row as f64 + col as f64)
    }, 0, 100).unwrap();
    
    // 第2批：100-200行
    writer.write_rows(|row, col| {
        CellData::text(format!("Batch2-R{}C{}", row, col))
    }, 100, 100).unwrap();
    
    // 第3批：200-300行
    writer.write_rows(|row, col| {
        CellData::number((row as f64) * (col as f64))
    }, 200, 100).unwrap();
    
    writer.end_sheet().unwrap();
    writer.close().unwrap();
    
    println!("  ✓ Created: stream_batch.xlsb (300 rows × 5 cols)");
    println!("    Batch1: Row 0-99   (数值: row+col)");
    println!("    Batch2: Row 100-199 (文本)");
    println!("    Batch3: Row 200-299 (数值: row*col)");
}

fn test_large_streaming() {
    println!("4. 大数据量流式写入");
    
    let path = PathBuf::from("/tmp/stream_large.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.start_sheet("LargeData", 4).unwrap();
    writer.write_rows(|row, col| {
        match col {
            0 => CellData::number(row as f64),
            1 => CellData::number((row as f64).sqrt()),
            2 => CellData::text(format!("Item-{}", row)),
            3 => CellData::bool(row % 10 == 0),
            _ => CellData::blank(),
        }
    }, 0, 5000).unwrap();
    writer.end_sheet().unwrap();
    writer.close().unwrap();
    
    println!("  ✓ Created: stream_large.xlsb (5000 rows × 4 cols)");
}
