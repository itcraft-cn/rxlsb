use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/tmp/format_test.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    // Match jxlsb NumberFormatTest.java exactly
    writer.write_batch("Formats", |row: usize, col: usize| {
        let value = row as f64 * 123.456 - 500.0;
        
        match col {
            0 => CellData::text(format!("Row-{}", row)),
            1 => CellData::number(value),           // 普通数值
            2 => CellData::percentage(value / 100.0), // 百分比 "0.00%"
            3 => CellData::number_with_comma(value),  // 千分位 "#,##0.00"
            4 => CellData::number_negative_red(value), // 负红 "#,##0.00;[Red]-#,##0.00"
            5 => CellData::currency(value),          // 货币 "￥#,##0.00"
            6 => {
                // 日期 - 使用内置格式 ifmt=22 ("m/d/yy h:mm")
                let excel_date = 46142.31823571759; // Excel serial date
                CellData::number_with_format(excel_date, "m/d/yy h:mm")
            },
            7 => {
                // 时间 - 使用内置格式 ifmt=21 ("h:mm:ss")
                let excel_date = 46142.31823571759; // Excel serial date
                CellData::number_with_format(excel_date, "h:mm:ss")
            },
            _ => CellData::blank(),
        }
    }, 20, 8).unwrap();
    
    writer.close().unwrap();
    println!("Generated {}", path.display());
}