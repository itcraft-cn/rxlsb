use rxlsb::XlsbReader;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file.xlsb>", args[0]);
        std::process::exit(1);
    }
    
    let path = std::path::PathBuf::from(&args[1]);
    let mut reader = XlsbReader::builder().path(&path).build().unwrap();
    
    let rows = reader.read_rows(0, 0, 300).unwrap();
    println!("Total rows read: {}", rows.len());
    
    for (i, row) in rows.iter().enumerate().take(5) {
        println!("Row {}: {} cells", i, row.len());
    }
}
