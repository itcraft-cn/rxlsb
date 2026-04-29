use rxlsb::{TemplateFiller, CellData};
use std::path::PathBuf;

fn main() {
    let template = PathBuf::from("tests/resources/template/demo_template.xlsb");
    let output = PathBuf::from("/tmp/template_filled.xlsb");
    
    println!("Template: {}", template.display());
    
    let mut filler = TemplateFiller::builder()
        .template(&template)
        .output(&output)
        .build()
        .unwrap();
    
    println!("Sheet count: {}", filler.get_sheet_count());
    println!("Sheet names: {:?}", filler.get_sheet_names());
    
    filler.fill_batch(0, 12, 8,
        |row, col| {
            if col == 0 { CellData::number((row + 1) as f64) }
            else if col == 1 { CellData::text(format!("Item{}", row + 1)) }
            else { CellData::number((row + 1) as f64 * 100.5) }
        }, 5, 3
    ).unwrap();
    
    filler.save().unwrap();
    
    println!("Created: {}", output.display());
}
