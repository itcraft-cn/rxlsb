use rxlsb::{TemplateFiller, CellData};

fn main() {
    let template = std::path::PathBuf::from("tests/resources/template/demo_template.xlsb");
    let output = std::path::PathBuf::from("/tmp/template_filled.xlsb");
    
    println!("Template: {}", template.display());
    
    let mut filler = TemplateFiller::builder()
        .template(&template)
        .output(&output)
        .build()
        .unwrap();
    
    println!("Sheet count: {}", filler.get_sheet_count());
    println!("Sheet names: {:?}", filler.get_sheet_names());
    
    let people: [(&str, &str, i32, &str); 5] = [
        ("张三", "杭州", 32, "男"),
        ("李四", "北京", 28, "女"),
        ("王五", "上海", 35, "男"),
        ("赵六", "广州", 24, "女"),
        ("钱七", "深圳", 30, "男"),
    ];
    
    filler.fill_batch(0, 12, 8,
        |row, col| {
            let person: &(&str, &str, i32, &str) = &people[row];
            match col {
                0 => CellData::text(person.0),
                1 => CellData::text(person.1),
                2 => CellData::number(person.2 as f64),
                3 => CellData::text(person.3),
                _ => CellData::blank(),
            }
        }, 5, 4
    ).unwrap();
    
    filler.save().unwrap();
    
    println!("Created: {}", output.display());
}
