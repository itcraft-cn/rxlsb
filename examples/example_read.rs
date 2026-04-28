use rxlsb::XlsbReader;

fn main() {
    let mut reader = XlsbReader::builder()
        .path("output.xlsb")
        .build()
        .unwrap();
    
    let infos = reader.get_sheet_infos();
    println!("Sheet count: {}", infos.len());
    
    for info in infos {
        println!("Sheet {}: {}", info.index, info.name);
    }
    
    let mut row_count = 0;
    reader.for_each_row(0, |idx, cells: &[rxlsb::CellData]| {
        row_count += 1;
        if row_count <= 5 {
            println!("Row {}: {} cells", idx, cells.len());
        }
    }).unwrap();
    
    println!("Total rows read: {}", row_count);
}