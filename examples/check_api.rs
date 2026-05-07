use rxlsb::XlsbReader;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = std::path::PathBuf::from(&args[1]);
    let mut reader = XlsbReader::builder().path(&path).build().unwrap();
    
    let rows = reader.read_rows(0, 0, 200).unwrap();
    println!("Total rows: {}", rows.len());
    for (i, row) in rows.iter().enumerate().take(5) {
        println!("Row {}: {} cells - {:?}", i, row.len(), row);
    }
}
