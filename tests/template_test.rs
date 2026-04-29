use rxlsb::{TemplateFiller, CellData};
use tempfile::NamedTempFile;

const TEMPLATE_PATH: &str = "tests/resources/template/demo_template.xlsb";

#[test]
fn test_按坐标写() {
    let temp = NamedTempFile::new().unwrap();
    
    let mut filler = TemplateFiller::builder()
        .template(TEMPLATE_PATH)
        .output(temp.path())
        .build()
        .unwrap();
    
    filler.fill_batch(0, 2, 0, |row: usize, col: usize| {
        match col {
            0 => CellData::text(format!("姓名{}", row)),
            1 => CellData::text(format!("城市{}", row)),
            2 => CellData::number(row as f64 * 10.0),
            3 => CellData::text(if row % 2 == 0 { "男" } else { "女" }),
            _ => CellData::blank(),
        }
    }, 3, 4).unwrap();
    filler.save().unwrap();
    
    assert!(temp.path().exists());
}

#[test]
fn test_按模板位写() {
    let temp = NamedTempFile::new().unwrap();
    
    let mut filler = TemplateFiller::builder()
        .template(TEMPLATE_PATH)
        .output(temp.path())
        .build()
        .unwrap();
    
    let marker_pos = filler.find_marker(0, "${data}");
    assert!(marker_pos.is_some());
    
    filler.fill_at_marker(0, "${data}", |row: usize, col: usize| {
        match col {
            0 => CellData::text(format!("姓名{}", row)),
            1 => CellData::text(format!("城市{}", row)),
            2 => CellData::number(row as f64 * 10.0),
            3 => CellData::text(if row % 2 == 0 { "男" } else { "女" }),
            _ => CellData::blank(),
        }
    }, 2, 4).unwrap();
    filler.save().unwrap();
    
    assert!(temp.path().exists());
}

#[test]
fn test_模板流式写() {
    let temp = NamedTempFile::new().unwrap();
    
    let mut filler = TemplateFiller::builder()
        .template(TEMPLATE_PATH)
        .output(temp.path())
        .build()
        .unwrap();
    
    filler.start_fill(0, 2, 0, 4).unwrap();
    
    filler.fill_rows(vec![
        vec![CellData::text("流式1"), CellData::text("城市1"), CellData::number(20.0), CellData::text("男")],
    ]).unwrap();
    
    filler.fill_rows(vec![
        vec![CellData::text("流式2"), CellData::text("城市2"), CellData::number(22.0), CellData::text("女")],
    ]).unwrap();
    
    filler.end_fill().unwrap();
    filler.save().unwrap();
    
    assert!(temp.path().exists());
}