use rxlsb::{CellData, XlsbReader, XlsbWriter};

#[test]
fn rk_boundary_roundtrip() {
    let path = "/tmp/rxlsb_rk_boundary.xlsb";
    let _ = std::fs::remove_file(path);

    let values = [
        0.0,
        -536_870_912.0,
        536_870_911.0,
        1.5,
        123_456_789.0,
        536_870_912.0,
        std::f64::consts::PI,
    ];

    let mut writer = XlsbWriter::builder().path(path).build().unwrap();
    writer
        .write_batch(
            "RK",
            |_row, col| CellData::number(values[col]),
            1,
            values.len(),
        )
        .unwrap();
    writer.close().unwrap();

    let mut reader = XlsbReader::builder().path(path).build().unwrap();
    let rows = reader.read_rows(0, 0, 1).unwrap();
    assert_eq!(rows.len(), 1);

    for (i, expected) in values.iter().enumerate() {
        match &rows[0][i] {
            CellData::Number(actual) => {
                assert_eq!(actual, expected, "col {i}: expected {expected}, got {actual}");
            }
            other => panic!("col {i}: expected Number, got {other:?}"),
        }
    }

    let _ = std::fs::remove_file(path);
}
