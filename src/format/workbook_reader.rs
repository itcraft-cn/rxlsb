use crate::io::BufferReader;
use crate::format::RecordType;
use crate::data::SheetInfo;
use crate::error::Result;
use bytes::Bytes;

pub struct WorkbookReader {
    sheets: Vec<SheetInfo>,
}

impl WorkbookReader {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { sheets: vec![] }
    }
    
    pub fn deserialize(data: Bytes) -> Result<Self> {
        let mut reader = BufferReader::new(data);
        let mut sheets = vec![];
        
        while reader.has_remaining() {
            let record_type_code = reader.read_varint()?;
            let size = reader.read_varsize()?;
            
            let record_type = RecordType::from_u32(record_type_code);
            
            match record_type {
                Some(RecordType::BrtBeginBook) => {
                }
                
                Some(RecordType::BrtBundleSh) => {
                    reader.skip(8)?;
                    let _rel_id = reader.read_wide_string_u32()?;
                    let name = reader.read_wide_string_u32()?;
                    let index = sheets.len();
                    sheets.push(SheetInfo { index, name });
                }
                
                Some(RecordType::BrtEndBook) => {
                    break;
                }
                
                _ => {
                    reader.skip(size as usize)?;
                }
            }
        }
        
        Ok(Self { sheets })
    }
    
    pub fn get_sheet_infos(&self) -> &[SheetInfo] {
        &self.sheets
    }
    
    pub fn sheet_count(&self) -> usize {
        self.sheets.len()
    }
}