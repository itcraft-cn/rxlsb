use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/tmp/format_test.xlsb");
    let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
    
    writer.write_batch("Formats", |row, col| {
        match row {
            0 => match col {
                0 => CellData::text("格式类型"),
                1 => CellData::text("示例值"),
                _ => CellData::blank(),
            },
            1 => match col {
                0 => CellData::text("普通数值"),
                1 => CellData::number(12345.67),
                _ => CellData::blank(),
            },
            2 => match col {
                0 => CellData::text("百分比"),
                1 => CellData::percentage(0.1234),
                _ => CellData::blank(),
            },
            3 => match col {
                0 => CellData::text("百分比(1位小数)"),
                1 => CellData::percentage_with_decimals(0.1234, 1),
                _ => CellData::blank(),
            },
            4 => match col {
                0 => CellData::text("千分位"),
                1 => CellData::number_with_comma(1234567.89),
                _ => CellData::blank(),
            },
            5 => match col {
                0 => CellData::text("负数红色"),
                1 => CellData::number_negative_red(-1234.56),
                _ => CellData::blank(),
            },
            6 => match col {
                0 => CellData::text("货币"),
                1 => CellData::currency(1234.56),
                _ => CellData::blank(),
            },
            7 => match col {
                0 => CellData::text("货币(美元)"),
                1 => CellData::currency_with_symbol(1234.56, "$"),
                _ => CellData::blank(),
            },
            8 => match col {
                0 => CellData::text("时间"),
                1 => CellData::time(1234567890),
                _ => CellData::blank(),
            },
            9 => match col {
                0 => CellData::text("日期"),
                1 => CellData::date_from_timestamp(1234567890),
                _ => CellData::blank(),
            },
            _ => CellData::blank(),
        }
    }, 10, 2).unwrap();
    
    writer.close().unwrap();
    println!("Generated {}", path.display());
}