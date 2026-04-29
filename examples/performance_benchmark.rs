use rxlsb::{XlsbWriter, XlsbReader, CellData};
use std::path::PathBuf;
use std::time::Instant;
use std::fs;

const TOTAL_ROWS: usize = 10_000;
const COLUMN_COUNT: usize = 10;
const PAGE_SIZE: usize = 1000;

fn create_cell(row: usize, col: usize) -> CellData {
    match col {
        0 => CellData::text(format!("Name-{}", row)),
        1 => CellData::text(format!("Category-{}", row % 100)),
        2 => CellData::text(format!("Region-{}", row % 10)),
        3 => CellData::number(row as f64),
        4 => CellData::number(row as f64 * 10.0),
        5 => CellData::number(row as f64 * 100.0),
        6 => CellData::number((row % 1000) as f64),
        7 => CellData::number(row as f64 * 1.5),
        8 => CellData::number(row as f64 * 2.345),
        9 => CellData::number(row as f64 / 7.0),
        _ => CellData::blank(),
    }
}

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         rxlsb 性能基准测试 (100万行 × 10字段)                 ║");
    println!("║         字段类型: 3字符串 + 4整数 + 3浮点数                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    let batch_file = PathBuf::from("/tmp/rxlsb_batch_write.xlsb");
    let stream_file = PathBuf::from("/tmp/rxlsb_stream_write.xlsb");
    
    println!("═════════════════════════════════════════════════════════════");
    println!("1. 普通分页写 (start_sheet + write_rows + end_sheet)");
    println!("═════════════════════════════════════════════════════════════\n");
    
    let start = Instant::now();
    let mut batch_writer = XlsbWriter::builder().path(&batch_file).build().unwrap();
    batch_writer.start_sheet("Data", COLUMN_COUNT).unwrap();
    
    let mut offset = 0;
    while offset < TOTAL_ROWS {
        let batch_size = if offset + PAGE_SIZE > TOTAL_ROWS {
            TOTAL_ROWS - offset
        } else {
            PAGE_SIZE
        };
        batch_writer.write_rows(
            |row: usize, col: usize| create_cell(offset + row, col),
            offset,
            batch_size
        ).unwrap();
        offset += batch_size;
    }
    
    batch_writer.end_sheet().unwrap();
    batch_writer.close().unwrap();
    let batch_write_time = start.elapsed().as_millis();
    let batch_write_size = fs::metadata(&batch_file).unwrap().len();
    
    println!("写入耗时: {} ms", batch_write_time);
    println!("文件大小: {:.2} MB", batch_write_size as f64 / 1_000_000.0);
    println!("写入速度: {:.0} 行/秒", TOTAL_ROWS as f64 / (batch_write_time as f64 / 1000.0));
    
    println!("\n═════════════════════════════════════════════════════════════");
    println!("2. 普通分页读 (read_rows, 每次1000行)");
    println!("═════════════════════════════════════════════════════════════\n");
    
    let start = Instant::now();
    let mut batch_reader = XlsbReader::builder().path(&batch_file).build().unwrap();
    
    let mut offset = 0;
    while offset < TOTAL_ROWS {
        let batch = batch_reader.read_rows(0, offset, PAGE_SIZE).unwrap();
        if batch.is_empty() {
            break;
        }
        offset += batch.len();
    }
    
    let _ = batch_reader;
    let batch_read_time = start.elapsed().as_millis();
    
    println!("读取耗时: {} ms", batch_read_time);
    println!("读取速度: {:.0} 行/秒", TOTAL_ROWS as f64 / (batch_read_time as f64 / 1000.0));
    
    println!("\n═════════════════════════════════════════════════════════════");
    println!("3. 普通流式写 (write_batch)");
    println!("═════════════════════════════════════════════════════════════\n");
    
    let start = Instant::now();
    let mut stream_writer = XlsbWriter::builder().path(&stream_file).build().unwrap();
    
    stream_writer.write_batch("Data", |row: usize, col: usize| create_cell(row, col), TOTAL_ROWS, COLUMN_COUNT).unwrap();
    stream_writer.close().unwrap();
    let stream_write_time = start.elapsed().as_millis();
    let stream_write_size = fs::metadata(&stream_file).unwrap().len();
    
    println!("写入耗时: {} ms", stream_write_time);
    println!("文件大小: {:.2} MB", stream_write_size as f64 / 1_000_000.0);
    println!("写入速度: {:.0} 行/秒", TOTAL_ROWS as f64 / (stream_write_time as f64 / 1000.0));
    
    println!("\n═════════════════════════════════════════════════════════════");
    println!("4. 普通流式读 (for_each_row)");
    println!("═════════════════════════════════════════════════════════════\n");
    
    let start = Instant::now();
    let mut stream_reader = XlsbReader::builder().path(&stream_file).build().unwrap();
    
    let mut row_count = 0;
    stream_reader.for_each_row(0, |_idx: usize, _cells: &[CellData]| {
        row_count += 1;
    }).unwrap();
    
    let _ = stream_reader;
    let stream_read_time = start.elapsed().as_millis();
    
    println!("读取耗时: {} ms", stream_read_time);
    println!("读取速度: {:.0} 行/秒", TOTAL_ROWS as f64 / (stream_read_time as f64 / 1000.0));
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    性能测试结果汇总                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    println!("┌────────────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│ 操作类型           │ 耗时 (ms)    │ 文件大小(MB) │ 速度(行/秒) │");
    println!("├────────────────────┼──────────────┼──────────────┼──────────────┤");
    println!("│ 分页写             │ {:>12} │ {:>12.2} │ {:>12.0} │",
        batch_write_time,
        batch_write_size as f64 / 1_000_000.0,
        TOTAL_ROWS as f64 / (batch_write_time as f64 / 1000.0));
    println!("│ 分页读             │ {:>12} │ {:>12} │ {:>12.0} │",
        batch_read_time,
        "-",
        TOTAL_ROWS as f64 / (batch_read_time as f64 / 1000.0));
    println!("│ 流式写             │ {:>12} │ {:>12.2} │ {:>12.0} │",
        stream_write_time,
        stream_write_size as f64 / 1_000_000.0,
        TOTAL_ROWS as f64 / (stream_write_time as f64 / 1000.0));
    println!("│ 流式读             │ {:>12} │ {:>12} │ {:>12.0} │",
        stream_read_time,
        "-",
        TOTAL_ROWS as f64 / (stream_read_time as f64 / 1000.0));
    println!("└────────────────────┴──────────────┴──────────────┴──────────────┘");
}