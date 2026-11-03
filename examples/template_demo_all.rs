use std::path::PathBuf;

use rxlsb::{CellData, TemplateFiller, XlsbReader};

const TEMPLATE: &str = "tests/resources/template/demo_template.xlsb";

fn row_cells(row: usize) -> Vec<CellData> {
    let names = ["张三", "李四", "王五", "赵六", "钱七", "孙八", "周九", "吴十"];
    let cities = ["杭州", "北京", "上海", "广州", "深圳", "成都", "南京", "武汉"];
    vec![
        CellData::text(names[row % names.len()].to_string()),
        CellData::text(cities[row % cities.len()].to_string()),
        CellData::number(20.0 + row as f64),
        CellData::text(if row % 2 == 0 { "男" } else { "女" }.to_string()),
    ]
}

fn supplier(row: usize, col: usize) -> CellData {
    row_cells(row).get(col).cloned().unwrap_or(CellData::blank())
}

fn dump_template() {
    println!("=== 模板原始内容 ({TEMPLATE}) ===");
    let mut reader = XlsbReader::builder().path(TEMPLATE).build().unwrap();
    reader
        .for_each_row(0, |idx: usize, cells: &[CellData]| {
            println!("  row {idx}: {cells:?}");
        })
        .unwrap();
}

fn main() {
    dump_template();

    // 1. 按坐标批量填充
    let out1 = PathBuf::from("/tmp/template_batch.xlsb");
    let mut f1 = TemplateFiller::builder().template(TEMPLATE).output(&out1).build().unwrap();
    f1.fill_batch(0, 12, 8, supplier, 8, 4).unwrap();
    f1.save().unwrap();
    println!("Created: {}", out1.display());

    // 2. 按标记 ${data} 填充
    let out2 = PathBuf::from("/tmp/template_marker.xlsb");
    let mut f2 = TemplateFiller::builder().template(TEMPLATE).output(&out2).build().unwrap();
    if f2.find_marker(0, "${data}").is_some() {
        f2.fill_at_marker(0, "${data}", supplier, 8, 4).unwrap();
        f2.save().unwrap();
        println!("Created: {} (marker ${{data}})", out2.display());
    } else {
        println!("template has no ${{data}} marker; skipped {}", out2.display());
    }

    // 3. 流式模板写（分 3 批）
    let out3 = PathBuf::from("/tmp/template_stream.xlsb");
    let mut f3 = TemplateFiller::builder().template(TEMPLATE).output(&out3).build().unwrap();
    f3.start_fill(0, 12, 8, 4).unwrap();
    for batch in 0..3 {
        let start = batch * 3;
        let rows: Vec<Vec<CellData>> = (start..start + 3).map(row_cells).collect();
        f3.fill_rows(rows).unwrap();
    }
    f3.end_fill().unwrap();
    f3.save().unwrap();
    println!("Created: {} (stream, 9 rows)", out3.display());
}
