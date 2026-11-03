use std::path::PathBuf;

use rxlsb::{
    set_column_overflow, set_row_overflow, CellData, OverflowBehavior, XlsbReader, XlsbWriter,
    MAX_COLUMNS,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let cols: usize = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);
    let mode = args.next().unwrap_or_else(|| "error".to_string());
    let rows = 3usize;

    // 演示全局配置：列超限按需 error / cut
    let behavior = if mode == "cut" {
        OverflowBehavior::Cut
    } else {
        OverflowBehavior::Error
    };
    set_column_overflow(behavior);
    set_row_overflow(OverflowBehavior::Error);

    let effective_cols = cols.min(MAX_COLUMNS);
    let path = PathBuf::from(format!("/tmp/wide_{cols}_{mode}.xlsb"));

    let mut w = XlsbWriter::builder().path(&path).build().unwrap();
    let result = w.write_batch(
        "Wide",
        |r: usize, c: usize| {
            if c % 500 == 0 {
                CellData::text(format!("R{r}C{c}"))
            } else {
                CellData::Number((r * cols + c) as f64)
            }
        },
        rows,
        cols,
    );

    match result {
        Ok(()) => {
            w.close().unwrap();
            let mut reader = XlsbReader::builder().path(&path).build().unwrap();
            let read = reader.read_rows(0, 0, rows).unwrap();
            println!(
                "cols={cols} mode={mode} effective={effective_cols} read_row0_cells={} size={}",
                read.first().map(|r| r.len()).unwrap_or(0),
                std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
            );
            println!("Created: {}", path.display());
        }
        Err(e) => {
            let _ = std::fs::remove_file(&path);
            println!("cols={cols} mode={mode} -> error: {e}");
        }
    }
}
