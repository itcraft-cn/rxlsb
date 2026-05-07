use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    // 1. verify_basic
    let path = PathBuf::from("/tmp/verify_basic.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    writer.write_batch("Basic", |row, col| {
        match col {
            0 => CellData::text(format!("Row-{}", row)),
            1 => CellData::number(row as f64 * 100.5),
            2 => CellData::bool(row % 2 == 0),
            _ => CellData::blank(),
        }
    }, 50, 5).unwrap();
    writer.close().unwrap();
    println!("Created: verify_basic.xlsb (50 rows × 5 cols)");
    
    // 2. verify_formats
    let path = PathBuf::from("/tmp/verify_formats.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    writer.write_batch("Formats", |row, col| {
        let value = row as f64 * 123.456 - 500.0;
        match col {
            0 => CellData::text(format!("Row-{}", row)),
            1 => CellData::number(value),
            2 => CellData::percentage(value / 100.0),
            3 => CellData::number_with_comma(value),
            4 => CellData::number_negative_red(value),
            5 => CellData::currency(value),
            6 => CellData::date_from_timestamp(1714560000 + row as i64 * 86400),
            7 => CellData::time(1714560000 + row as i64 * 86400),
            _ => CellData::blank(),
        }
    }, 100, 8).unwrap();
    writer.close().unwrap();
    println!("Created: verify_formats.xlsb (100 rows × 8 cols, formats)");
    
    // 3. verify_streaming
    let path = PathBuf::from("/tmp/verify_streaming.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    writer.write_batch("Stream", |row, col| {
        match col {
            0 => CellData::number(row as f64),
            1 => CellData::number(row as f64 * 2.5),
            2 => CellData::text(format!("Text-{}", row)),
            3 => CellData::bool(row % 3 == 0),
            _ => CellData::blank(),
        }
    }, 200, 4).unwrap();
    writer.close().unwrap();
    println!("Created: verify_streaming.xlsb (200 rows × 4 cols)");
    
    // 4. verify_large
    let path = PathBuf::from("/tmp/verify_large.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    writer.write_batch("Large", |row, col| {
        match col {
            0 => CellData::number(row as f64),
            1 => CellData::number((row as f64).sqrt()),
            2 => CellData::text(format!("Item-{}", row)),
            _ => CellData::blank(),
        }
    }, 10000, 4).unwrap();
    writer.close().unwrap();
    println!("Created: verify_large.xlsb (10000 rows × 4 cols)");
    
    // 5. verify_mixed
    let path = PathBuf::from("/tmp/verify_mixed.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    writer.write_batch("Mixed", |row, col| {
        match col {
            0 => CellData::text(format!("Text-{}", row)),
            1 => CellData::number(row as f64 * 1.5),
            2 => CellData::number_with_format(row as f64 * 2.5, "#,##0.00"),
            3 => CellData::percentage(row as f64 / 100.0),
            4 => CellData::bool(row % 2 == 0),
            _ => CellData::blank(),
        }
    }, 500, 6).unwrap();
    writer.close().unwrap();
    println!("Created: verify_mixed.xlsb (500 rows × 6 cols, mixed types)");
}
