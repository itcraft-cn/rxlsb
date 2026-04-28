use rxlsb::{XlsbWriter, CellData};

fn main() {
    let mut writer = XlsbWriter::builder()
        .path("output.xlsb")
        .build()
        .unwrap();
    
    writer.write_batch("Sheet1", |row, col| {
        match col % 4 {
            0 => CellData::text(format!("Product-{}", row)),
            1 => CellData::number(row as f64 * 100.5),
            2 => CellData::bool(row % 2 == 0),
            3 => CellData::blank(),
            _ => CellData::blank(),
        }
    }, 1000, 4).unwrap();
    
    writer.close().unwrap();
    
    println!("XLSB file created: output.xlsb");
    println!("  - Sheet1: 1000 rows x 4 columns");
}