use rxlsb::{
    column_overflow, row_overflow, set_column_overflow, set_overflow, set_row_overflow, CellData,
    OverflowBehavior, TemplateFiller, XlsbError, XlsbReader, XlsbWriter, MAX_COLUMNS, MAX_ROWS,
};

const TEMPLATE: &str = "tests/resources/template/demo_template.xlsb";

fn supplier(_r: usize, c: usize) -> CellData {
    CellData::number(c as f64)
}

#[test]
fn limits_strategy_and_override() {
    // 默认：Error
    set_overflow(OverflowBehavior::Error, OverflowBehavior::Error);
    assert_eq!(column_overflow(), OverflowBehavior::Error);
    assert_eq!(row_overflow(), OverflowBehavior::Error);

    // 列超限 -> Error
    {
        let path = "/tmp/limits_col_error.xlsb";
        let _ = std::fs::remove_file(path);
        let mut w = XlsbWriter::builder().path(path).build().unwrap();
        let err = w
            .write_batch("S", supplier, 1, MAX_COLUMNS + 1)
            .unwrap_err();
        match err {
            XlsbError::LimitExceeded { dimension, .. } => assert_eq!(dimension, "columns"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    // 列超限 -> Cut（截断到 MAX_COLUMNS）
    {
        set_column_overflow(OverflowBehavior::Cut);
        assert_eq!(column_overflow(), OverflowBehavior::Cut);
        assert_eq!(row_overflow(), OverflowBehavior::Error); // 行不受影响

        let path = "/tmp/limits_col_cut.xlsb";
        let _ = std::fs::remove_file(path);
        let mut w = XlsbWriter::builder().path(path).build().unwrap();
        w.write_batch("S", supplier, 1, MAX_COLUMNS + 100).unwrap();
        w.close().unwrap();

        let mut r = XlsbReader::builder().path(path).build().unwrap();
        let rows = r.read_rows(0, 0, 1).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), MAX_COLUMNS);
    }

    // 总 API 同时设置；按维度后设置覆盖
    set_overflow(OverflowBehavior::Error, OverflowBehavior::Error);
    assert_eq!(column_overflow(), OverflowBehavior::Error);
    assert_eq!(row_overflow(), OverflowBehavior::Error);

    set_overflow(OverflowBehavior::Cut, OverflowBehavior::Cut);
    assert_eq!(column_overflow(), OverflowBehavior::Cut);
    assert_eq!(row_overflow(), OverflowBehavior::Cut);

    set_column_overflow(OverflowBehavior::Error); // 只覆盖列
    assert_eq!(column_overflow(), OverflowBehavior::Error);
    assert_eq!(row_overflow(), OverflowBehavior::Cut);

    // 行超限：Error
    set_row_overflow(OverflowBehavior::Error);
    {
        let path = "/tmp/limits_row_error.xlsb";
        let _ = std::fs::remove_file(path);
        let mut w = XlsbWriter::builder().path(path).build().unwrap();
        w.start_sheet("S", 2).unwrap();
        let err = w.write_rows(supplier, MAX_ROWS - 1, 5).unwrap_err();
        match err {
            XlsbError::LimitExceeded { dimension, .. } => assert_eq!(dimension, "rows"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    // 行超限：Cut（只写 1 行）
    set_row_overflow(OverflowBehavior::Cut);
    {
        let path = "/tmp/limits_row_cut.xlsb";
        let _ = std::fs::remove_file(path);
        let mut w = XlsbWriter::builder().path(path).build().unwrap();
        w.start_sheet("S", 2).unwrap();
        w.write_rows(supplier, MAX_ROWS - 1, 5).unwrap();
        w.end_sheet().unwrap();
        w.close().unwrap();

        let mut r = XlsbReader::builder().path(path).build().unwrap();
        let rows = r.read_rows(0, MAX_ROWS - 2, 10).unwrap();
        assert_eq!(rows.len(), 1);
    }

    // 快照语义：start_sheet 后再改配置，不影响该 sheet
    set_row_overflow(OverflowBehavior::Error);
    {
        let path = "/tmp/limits_snapshot.xlsb";
        let _ = std::fs::remove_file(path);
        let mut w = XlsbWriter::builder().path(path).build().unwrap();
        w.start_sheet("S", 2).unwrap(); // 快照 Error
        set_row_overflow(OverflowBehavior::Cut); // 之后才改为 Cut
        let err = w.write_rows(supplier, MAX_ROWS - 1, 5).unwrap_err();
        assert!(matches!(err, XlsbError::LimitExceeded { .. }));
    }

    // 模板填充同样走底层限制：Cut 截断、Error 报错
    set_overflow(OverflowBehavior::Cut, OverflowBehavior::Error);
    {
        let out = "/tmp/limits_template_cut.xlsb";
        let _ = std::fs::remove_file(out);
        let mut f = TemplateFiller::builder()
            .template(TEMPLATE)
            .output(out)
            .build()
            .unwrap();
        f.fill_batch(0, 12, MAX_COLUMNS as u32 - 2, supplier, 1, 10).unwrap();
        f.save().unwrap();
        assert!(std::path::Path::new(out).exists());
        let _ = std::fs::remove_file(out);
    }

    set_column_overflow(OverflowBehavior::Error);
    {
        let out = "/tmp/limits_template_error.xlsb";
        let _ = std::fs::remove_file(out);
        let mut f = TemplateFiller::builder()
            .template(TEMPLATE)
            .output(out)
            .build()
            .unwrap();
        f.fill_batch(0, 12, MAX_COLUMNS as u32 - 2, supplier, 1, 10).unwrap();
        let err = f.save().unwrap_err();
        match err {
            XlsbError::LimitExceeded { dimension, .. } => assert_eq!(dimension, "columns"),
            other => panic!("unexpected error: {other:?}"),
        }
        let _ = std::fs::remove_file(out);
    }

    // 还原默认，避免影响其他测试
    set_overflow(OverflowBehavior::Error, OverflowBehavior::Error);
}
