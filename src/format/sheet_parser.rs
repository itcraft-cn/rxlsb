
#[derive(Debug, Clone)]
pub enum CellValue {
    Text(String),
    Number(f64),
    Bool(bool),
    #[allow(dead_code)]
    Blank,
}

#[derive(Debug, Clone)]
pub struct CellInfo {
    pub row: u32,
    pub col: u32,
    pub value: CellValue,
    pub style_index: u32,
}

#[derive(Debug, Clone)]
pub struct MergeCell {
    pub row_first: u32,
    pub row_last: u32,
    pub col_first: u32,
    pub col_last: u32,
}

pub struct SheetParser {
    pub cells: Vec<CellInfo>,
    pub merges: Vec<MergeCell>,
    pub max_row: u32,
    pub max_col: u32,
}

impl SheetParser {
    pub fn parse(data: &[u8], sst_strings: &[&str]) -> Self {
        let mut cells = Vec::with_capacity(256);
        let mut merges = Vec::with_capacity(16);
        let mut max_row = 0u32;
        let mut max_col = 0u32;
        let mut current_row = 0u32;
        let mut pos = 0;
        
        while pos + 2 < data.len() {
            let record_type = if data[pos] >= 128 {
                pos += 2;
                ((data[pos-2] & 0x7F) as u32) | ((data[pos-1] & 0x7F) << 7) as u32
            } else {
                pos += 1;
                data[pos-1] as u32
            };
            
            let record_size = if data[pos] >= 128 {
                pos += 2;
                ((data[pos-2] & 0x7F) as u32) | ((data[pos-1] as u32) << 7)
            } else {
                pos += 1;
                data[pos-1] as u32
            };
            
            if pos + record_size as usize > data.len() { break; }
            
            match record_type {
                0 => {
                    current_row = u32::from_le_bytes([
                        data[pos], data[pos+1], data[pos+2], data[pos+3]
                    ]);
                    if current_row > max_row { max_row = current_row; }
                }
                7 => {
                    let col = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                    let style_idx = (data[pos+4] as u32) | ((data[pos+5] as u32) << 8) | ((data[pos+6] as u32) << 16);
                    let sst_idx = u32::from_le_bytes([data[pos+8], data[pos+9], data[pos+10], data[pos+11]]);
                    if (sst_idx as usize) < sst_strings.len() {
                        cells.push(CellInfo {
                            row: current_row, col,
                            value: CellValue::Text(sst_strings[sst_idx as usize].to_string()),
                            style_index: style_idx,
                        });
                        if col > max_col { max_col = col; }
                    }
                }
                5 => {
                    let col = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                    let style_idx = (data[pos+4] as u32) | ((data[pos+5] as u32) << 8) | ((data[pos+6] as u32) << 16);
                    let val_bytes = [data[pos+8], data[pos+9], data[pos+10], data[pos+11],
                                    data[pos+12], data[pos+13], data[pos+14], data[pos+15]];
                    let value = f64::from_bits(u64::from_le_bytes(val_bytes));
                    cells.push(CellInfo {
                        row: current_row, col, value: CellValue::Number(value), style_index: style_idx,
                    });
                    if col > max_col { max_col = col; }
                }
                4 => {
                    let col = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                    let style_idx = (data[pos+4] as u32) | ((data[pos+5] as u32) << 8) | ((data[pos+6] as u32) << 16);
                    let value = data[pos+8] != 0;
                    cells.push(CellInfo {
                        row: current_row, col, value: CellValue::Bool(value), style_index: style_idx,
                    });
                    if col > max_col { max_col = col; }
                }
                176 => {
                    let rf = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                    let rl = u32::from_le_bytes([data[pos+4], data[pos+5], data[pos+6], data[pos+7]]);
                    let cf = u32::from_le_bytes([data[pos+8], data[pos+9], data[pos+10], data[pos+11]]);
                    let cl = u32::from_le_bytes([data[pos+12], data[pos+13], data[pos+14], data[pos+15]]);
                    merges.push(MergeCell { row_first: rf, row_last: rl, col_first: cf, col_last: cl });
                    if rl > max_row { max_row = rl; }
                    if cl > max_col { max_col = cl; }
                }
                _ => {}
            }
            
            pos += record_size as usize;
        }
        
        SheetParser { cells, merges, max_row, max_col }
    }
    
    pub fn find_text_cell(&self, marker: &str) -> Option<(u32, u32)> {
        for cell in &self.cells {
            if let CellValue::Text(text) = &cell.value {
                if text == marker {
                    return Some((cell.row, cell.col));
                }
            }
        }
        None
    }
}