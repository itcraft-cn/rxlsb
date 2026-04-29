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
        
        writer.write_varint(RecordType::BrtBeginBook.to_u32());
        writer.write_varsize(0);
        
        self.write_brt_file_version(&mut writer);
        self.write_brt_wb_prop(&mut writer);
        
        writer.write_varint(RecordType::BrtBeginBookViews.to_u32());
        writer.write_varsize(0);
        
        self.write_brt_book_view(&mut writer);
        
        writer.write_varint(RecordType::BrtEndBookViews.to_u32());
        writer.write_varsize(0);
        
        writer.write_varint(RecordType::BrtBeginBundleShs.to_u32());
        writer.write_varsize(0);
        
        for (i, sheet_name) in self.sheets.iter().enumerate() {
            let rel_id = format!("rId{}", i + 1);
            let rel_id_chars = rel_id.encode_utf16().count();
            let name_chars = sheet_name.encode_utf16().count();
            
            let rel_id_size = 4 + rel_id_chars * 2;
            let name_size = 4 + name_chars * 2;
            
            writer.write_varint(RecordType::BrtBundleSh.to_u32());
            writer.write_varsize((8 + rel_id_size + name_size) as u32);
            writer.write_u32_le(0);
            writer.write_u32_le(i as u32 + 1);
            
            writer.write_varint(rel_id_chars as u32);
            for ch in rel_id.encode_utf16() {
                writer.write_u16_le(ch);
            }
            
            writer.write_varint(name_chars as u32);
            for ch in sheet_name.encode_utf16() {
                writer.write_u16_le(ch);
            }
        }
        
        writer.write_varint(RecordType::BrtEndBundleShs.to_u32());
        writer.write_varsize(0);
        
        writer.write_varint(RecordType::BrtEndBook.to_u32());
        writer.write_varsize(0);
        
        Ok(writer.freeze())
    }
    
    fn write_brt_file_version(&self, writer: &mut BufferWriter) {
        writer.write_varint(128);
        writer.write_varsize(47);
        writer.write_bytes(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x78, 0x00, 0x6c, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x33, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x35, 0x00, 0x04, 0x00, 0x00, 0x00, 0x39, 0x00,
            0x33, 0x00, 0x30, 0x00, 0x32, 0x00, 0x00
        ]);
    }
    
    fn write_brt_wb_prop(&self, writer: &mut BufferWriter) {
        writer.write_varint(153);
        writer.write_varsize(12);
        writer.write_bytes(&[
            0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00
        ]);
    }
    
    fn write_brt_book_view(&self, writer: &mut BufferWriter) {
        writer.write_varint(158);
        writer.write_varsize(29);
        writer.write_bytes(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x80, 0x70, 0x00, 0x00, 0xcf, 0x30, 0x00, 0x00,
            0x58, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x78
        ]);
    }
}