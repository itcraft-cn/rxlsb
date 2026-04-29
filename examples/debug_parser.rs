fn main() {
    let template = std::path::PathBuf::from("tests/resources/template/demo_template.xlsb");
    let file = std::fs::File::open(&template).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    
    let sst_strings = ["部门人员统计", "姓名", "籍贯", "年龄", "性别", "${data}"];
    
    let sheet_data: Vec<u8> = {
        let mut file = archive.by_name("xl/worksheets/sheet1.bin").unwrap();
        let mut d = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut d).unwrap();
        d
    };
    
    println!("Sheet data len: {} bytes", sheet_data.len());
    println!("First 100 bytes: {:02x?}", &sheet_data[..100]);
    
    // Manually parse a few records
    let mut pos = 0;
    let mut current_row = 0u32;
    let mut cells = vec![];
    
    while pos < sheet_data.len() - 2 {
        let t = if sheet_data[pos] >= 128 {
            pos += 2;
            ((sheet_data[pos-2] & 0x7F) as u32) | ((sheet_data[pos-1] & 0x7F) << 7) as u32
        } else {
            pos += 1;
            sheet_data[pos-1] as u32
        };
        
        let s = if sheet_data[pos] >= 128 {
            pos += 2;
            ((sheet_data[pos-2] & 0x7F) as u32) | ((sheet_data[pos-1] as u32) << 7)
        } else {
            pos += 1;
            sheet_data[pos-1] as u32
        };
        
        if t == 0 { // BrtRowHdr
            current_row = u32::from_le_bytes([sheet_data[pos], sheet_data[pos+1], sheet_data[pos+2], sheet_data[pos+3]]);
        }
        
        if t == 7 { // BrtCellIsst
            let col = u32::from_le_bytes([sheet_data[pos], sheet_data[pos+1], sheet_data[pos+2], sheet_data[pos+3]]);
            let sst_idx = u32::from_le_bytes([sheet_data[pos+8], sheet_data[pos+9], sheet_data[pos+10], sheet_data[pos+11]]);
            cells.push((current_row, col, sst_idx));
        }
        
        pos += s as usize;
    }
    
    println!("\nParsed {} cells:", cells.len());
    for (row, col, idx) in &cells {
        let text = if *idx < sst_strings.len() as u32 { sst_strings[*idx as usize] } else { "?" };
        println!("  row={}, col={}, sst={}, text='{}'", row, col, idx, text);
    }
}
