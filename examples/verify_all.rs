use rxlsb::{XlsbWriter, XlsbReader, CellData};
use std::path::PathBuf;

fn main() {
    println!("=== 完整读写验证测试 ===\n");
    
    // 1. 基础读写测试
    test_basic_write_read();
    
    // 2. 格式测试
    test_formats();
    
    // 3. 流式读写测试
    test_streaming();
    
    // 4. 大数据量测试
    test_large_data();
    
    // 5. 混合类型测试
    test_mixed_types();
    
    println!("\n=== 所有测试完成 ===");
    println!("生成文件列表：");
    println!("  /tmp/verify_basic.xlsb      - 基础读写");
    println!("  /tmp/verify_formats.xlsb    - 格式验证");
    println!("  /tmp/verify_streaming.xlsb  - 流式读写");
    println!("  /tmp/verify_large.xlsb      - 大数据量(10000行)");
    println!("  /tmp/verify_mixed.xlsb      - 混合类型");
    println!("\n请在WPS中打开验证！");
}

fn test_basic_write_read() {
    println!("1. 基础读写测试");
    
    let path = PathBuf::from("/tmp/verify_basic.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("Sheet1", |row, col| {
        match col {
            0 => CellData::text(format!("Row-{}", row)),
            1 => CellData::number(row as f64 * 100.5),
            2 => CellData::bool(row % 2 == 0),
            _ => CellData::blank(),
        }
    }, 50, 5).unwrap();
    
    writer.close().unwrap();
    println!("  ✓ 写入完成: {} rows", 50);
    
    // 读取验证
    let mut reader = XlsbReader::builder().path(&path).build().unwrap();
    let rows = reader.read_rows(0, 0, 10).unwrap();
    println!("  ✓ 读取验证: {} rows read", rows.len());
    
    if rows.len() > 0 {
        println!("  ✓ 第1行数据: {} cells", rows[0].len());
    }
}

fn test_formats() {
    println!("\n2. 格式测试");
    
    let path = PathBuf::from("/tmp/verify_formats.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("Formats", |row: usize, col: usize| {
        let value = row as f64 * 123.456 - 500.0;
        let timestamp = 1714560000 + row as i64 * 86400;
        
        match col {
            0 => CellData::text(format!("Row-{}", row)),
            1 => CellData::percentage(value / 100.0),
            2 => CellData::number_with_comma(value),
            3 => CellData::number_negative_red(value),
            4 => CellData::currency(value),
            5 => CellData::date_from_timestamp(timestamp),
            6 => CellData::time(timestamp),
            _ => CellData::blank(),
        }
    }, 100, 8).unwrap();
    
    writer.close().unwrap();
    println!("  ✓ 格式文件生成: {} rows, 8 columns", 100);
    println!("    Col 1: 百分比 (0.00%)");
    println!("    Col 2: 千分位 (#,##0.00)");
    println!("    Col 3: 负红 (#,##0.00;[Red]-#,##0.00)");
    println!("    Col 4: 货币 (￥#,##0.00)");
    println!("    Col 5: 日期 (m/d/yy h:mm)");
    println!("    Col 6: 时间 (h:mm:ss)");
}

fn test_streaming() {
    println!("\n3. 流式读写测试");
    
    let path = PathBuf::from("/tmp/verify_streaming.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("StreamSheet", |row, col| {
        match col {
            0 => CellData::number(row as f64),
            1 => CellData::number((row as f64) * 2.5),
            2 => CellData::text(format!("Text-{}", row)),
            3 => CellData::bool(row % 3 == 0),
            _ => CellData::blank(),
        }
    }, 200, 4).unwrap();
    
    writer.close().unwrap();
    println!("  ✓ 流式写入完成: {} rows", 200);
    
    // 分页读取验证
    let mut reader = XlsbReader::builder().path(&path).build().unwrap();
    let rows = reader.read_rows(0, 0, 10).unwrap();
    println!("  ✓ 分页读取验证: {} rows read", rows.len());
}

fn test_large_data() {
    println!("\n4. 大数据量测试 (10000 rows)");
    
    let path = PathBuf::from("/tmp/verify_large.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("LargeData", |row, col| {
        match col {
            0 => CellData::number(row as f64),
            1 => CellData::number((row as f64).sqrt()),
            2 => CellData::text(format!("Item-{}", row)),
            _ => CellData::blank(),
        }
    }, 10000, 4).unwrap();
    
    writer.close().unwrap();
    
    let file_size = std::fs::metadata(&path).unwrap().len();
    println!("  ✓ 写入完成: 10000 rows, file size: {} KB", file_size / 1024);
}

fn test_mixed_types() {
    println!("\n5. 混合类型测试");
    
    let path = PathBuf::from("/tmp/verify_mixed.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("MixedTypes", |row, col| {
        match col {
            0 => CellData::text(format!("Text-{}", row)),
            1 => CellData::number(row as f64 * 1.5),
            2 => CellData::number_with_format(row as f64 * 2.5, "#,##0.00"),
            3 => CellData::percentage(row as f64 / 100.0),
            4 => CellData::bool(row % 2 == 0),
            5 => CellData::blank(),
            _ => CellData::blank(),
        }
    }, 500, 6).unwrap();
    
    writer.close().unwrap();
    println!("  ✓ 混合类型写入完成: {} rows", 500);
}