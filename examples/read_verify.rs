use rxlsb::{XlsbReader, CellData};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("/tmp/rxlsb_batch_write.xlsb")
    };
    
    let mut reader = XlsbReader::builder().path(&path).build().unwrap();
    println!("读取文件: {}", path.display());
    println!("前10行数据:");
    reader.for_each_row(0, |idx: usize, cells: &[CellData]| {
        if idx < 10 {
            println!("Row {}: {:?}", idx, cells);
        }
    }).unwrap();
    
    println!("文件大小: {} MB", std::fs::metadata(&path).unwrap().len() / 1_000_000);
}