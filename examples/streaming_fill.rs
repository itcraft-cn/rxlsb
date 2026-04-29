use rxlsb::{TemplateFiller, CellData};

fn main() {
    let template = std::path::PathBuf::from("tests/resources/template/demo_template.xlsb");
    let output = std::path::PathBuf::from("/tmp/streaming_filled.xlsb");
    
    println!("Template: {}", template.display());
    
    let mut filler = TemplateFiller::builder()
        .template(&template)
        .output(&output)
        .build()
        .unwrap();
    
    println!("=== Streaming Fill Test ===");
    
    // Start streaming fill at row 12, col 8, 4 columns
    filler.start_fill(0, 12, 8, 4).unwrap();
    println!("Started fill at row 12, col 8");
    
    // Batch 1: 2 rows
    let batch1 = vec![
        vec![CellData::text("张三"), CellData::text("杭州"), CellData::number(32.0), CellData::text("男")],
        vec![CellData::text("李四"), CellData::text("北京"), CellData::number(28.0), CellData::text("女")],
    ];
    filler.fill_rows(batch1).unwrap();
    println!("Filled batch 1: 2 rows");
    
    // Batch 2: 2 rows
    let batch2 = vec![
        vec![CellData::text("王五"), CellData::text("上海"), CellData::number(35.0), CellData::text("男")],
        vec![CellData::text("赵六"), CellData::text("广州"), CellData::number(24.0), CellData::text("女")],
    ];
    filler.fill_rows(batch2).unwrap();
    println!("Filled batch 2: 2 rows");
    
    // Batch 3: 1 row
    let batch3 = vec![
        vec![CellData::text("钱七"), CellData::text("深圳"), CellData::number(30.0), CellData::text("男")],
    ];
    filler.fill_rows(batch3).unwrap();
    println!("Filled batch 3: 1 row");
    
    // End fill
    filler.end_fill().unwrap();
    println!("Ended fill, total 5 rows");
    
    filler.save().unwrap();
    println!("Created: {}", output.display());
}
