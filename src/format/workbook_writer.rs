use crate::io::BufferWriter;
use crate::format::RecordType;
use crate::data::SheetInfo;
use bytes::Bytes;
use crate::error::Result;

pub struct WorkbookWriter {
    sheets: Vec<String>,
}

impl WorkbookWriter {
    pub fn new() -> Self {
        Self { sheets: vec![] }
    }
    
    pub fn add_sheet(&mut self, name: &str) {
        self.sheets.push(name.to_string());
    }
    
    pub fn sheet_count(&self) -> usize {
        self.sheets.len()
    }
    
    pub fn serialize(&self) -> Result<Bytes> {
        let mut writer = BufferWriter::new(256);
        
        writer.write_u32_le(RecordType::BrtBeginBook.to_u32());
        writer.write_u32_le(0);
        
        for (i, sheet_name) in self.sheets.iter().enumerate() {
            let name_size = BufferWriter::utf16le_byte_length(sheet_name) 
                + BufferWriter::varint_size(sheet_name.encode_utf16().count() as u32);
            
            writer.write_u32_le(RecordType::BrtBundleSh.to_u32());
            writer.write_u32_le((16 + name_size) as u32);
            writer.write_u32_le(i as u32);
            writer.write_u32_le(0);
            writer.write_u32_le(0);
            writer.write_u32_le(0);
            writer.write_u32_le(0);
            writer.write_wide_string(sheet_name);
        }
        
        writer.write_u32_le(RecordType::BrtEndBook.to_u32());
        writer.write_u32_le(0);
        
        Ok(writer.freeze())
    }
}