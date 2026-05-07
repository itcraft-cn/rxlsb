use crate::io::BufferReader;
use crate::format::RecordType;
use crate::error::Result;
use bytes::Bytes;

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
        let result = Self::parse_with_reader(Bytes::copy_from_slice(data), sst_strings);
        result.unwrap_or_else(|_| Self {
            cells: Vec::new(),
            merges: Vec::new(),
            max_row: 0,
            max_col: 0,
        })
    }
    
    fn parse_with_reader(data: Bytes, sst_strings: &[&str]) -> Result<Self> {
        let mut reader = BufferReader::new(data);
        let mut cells = Vec::with_capacity(256);
        let mut merges = Vec::with_capacity(16);
        let mut max_row = 0u32;
        let mut max_col = 0u32;
        
        while reader.has_remaining() {
            let record_type_code = reader.read_varint()?;
            let size = reader.read_varsize()?;
            
            let record_type = RecordType::from_u32(record_type_code);
            
            match record_type {
                Some(RecordType::BrtRowHdr) => {
                    let row = reader.read_u32_le()?;
                    max_row = max_row.max(row);
                    reader.skip(size as usize - 4)?;
                }
                
                Some(RecordType::BrtCellBlank) => {
                    let col = reader.read_u32_le()?;
                    let _style_index = reader.read_u32_le()?;
                    reader.skip(size as usize - 8)?;
                    
                    max_col = max_col.max(col);
                }
                
                Some(RecordType::BrtCellRk) => {
                    let col = reader.read_u32_le()?;
                    let style_index = reader.read_u32_le()?;
                    let rk_bytes = reader.read_u32_le()?;
                    
                    max_col = max_col.max(col);
                    
                    let value = Self::decode_rk(rk_bytes);
                    cells.push(CellInfo {
                        row: max_row,
                        col,
                        value: CellValue::Number(value),
                        style_index,
                    });
                }
                
                Some(RecordType::BrtCellReal) => {
                    let col = reader.read_u32_le()?;
                    let style_index = reader.read_u32_le()?;
                    let value = reader.read_f64_le()?;
                    
                    max_col = max_col.max(col);
                    cells.push(CellInfo {
                        row: max_row,
                        col,
                        value: CellValue::Number(value),
                        style_index,
                    });
                }
                
                Some(RecordType::BrtCellBool) => {
                    let col = reader.read_u32_le()?;
                    let style_index = reader.read_u32_le()?;
                    let bool_value = reader.read_u8()?;
                    reader.skip(size as usize - 9)?;
                    
                    max_col = max_col.max(col);
                    cells.push(CellInfo {
                        row: max_row,
                        col,
                        value: CellValue::Bool(bool_value != 0),
                        style_index,
                    });
                }
                
                Some(RecordType::BrtCellIsst) => {
                    let col = reader.read_u32_le()?;
                    let style_index = reader.read_u32_le()?;
                    let sst_index = reader.read_u32_le()?;
                    reader.skip(size as usize - 12)?;
                    
                    max_col = max_col.max(col);
                    
                    let text = sst_strings.get(sst_index as usize)
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    
                    cells.push(CellInfo {
                        row: max_row,
                        col,
                        value: CellValue::Text(text),
                        style_index,
                    });
                }
                
                Some(RecordType::BrtCellSt) => {
                    let col = reader.read_u32_le()?;
                    let style_index = reader.read_u32_le()?;
                    
                    reader.skip(1)?;
                    let char_count = reader.read_u32_le()? as usize;
                    let mut chars = Vec::with_capacity(char_count);
                    for _ in 0..char_count {
                        chars.push(reader.read_u16_le()?);
                    }
                    
                    let text = String::from_utf16(&chars).unwrap_or_default();
                    
                    max_col = max_col.max(col);
                    cells.push(CellInfo {
                        row: max_row,
                        col,
                        value: CellValue::Text(text),
                        style_index,
                    });
                }
                
                Some(RecordType::BrtMergeCell) => {
                    let row_first = reader.read_u32_le()?;
                    let row_last = reader.read_u32_le()?;
                    let col_first = reader.read_u32_le()?;
                    let col_last = reader.read_u32_le()?;
                    
                    merges.push(MergeCell {
                        row_first,
                        row_last,
                        col_first,
                        col_last,
                    });
                    
                    max_row = max_row.max(row_last);
                    max_col = max_col.max(col_last);
                }
                
                Some(RecordType::BrtEndSheetData) => {
                    break;
                }
                
                _ => {
                    reader.skip(size as usize)?;
                }
            }
        }
        
        Ok(Self {
            cells,
            merges,
            max_row,
            max_col,
        })
    }
    
    fn decode_rk(rk: u32) -> f64 {
        let is_int = (rk & 0x02) != 0;
        let div_100 = (rk & 0x01) != 0;
        
        if is_int {
            let value = ((rk as i32) >> 2) as f64;
            if div_100 {
                value / 100.0
            } else {
                value
            }
        } else {
            let bits = ((rk as u64 & 0xFFFFFFFC) as u64) << 32;
            let value = f64::from_bits(bits);
            if div_100 {
                value / 100.0
            } else {
                value
            }
        }
    }
    
    pub fn get_cells(&self) -> &[CellInfo] {
        &self.cells
    }
    
    pub fn get_merges(&self) -> &[MergeCell] {
        &self.merges
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