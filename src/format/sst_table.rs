use std::collections::HashMap;
use crate::io::BufferWriter;
use crate::format::RecordType;
use bytes::Bytes;
use crate::error::Result;

pub struct SstTable {
    strings: Vec<String>,
    hash_map: HashMap<String, u32>,
    total_count: u32,
}

impl SstTable {
    pub fn new() -> Self {
        Self { strings: vec![], hash_map: HashMap::new(), total_count: 0 }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            strings: Vec::with_capacity(capacity),
            hash_map: HashMap::with_capacity(capacity),
            total_count: 0,
        }
    }
    
    pub fn add_string(&mut self, s: &str) -> u32 {
        self.total_count += 1;
        
        if let Some(idx) = self.hash_map.get(s) { return *idx; }
        
        let idx = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.hash_map.insert(s.to_string(), idx);
        idx
    }
    
    pub fn get_string(&self, idx: u32) -> Option<&str> {
        self.strings.get(idx as usize).map(|s| s.as_str())
    }
    
    pub fn find_string(&self, s: &str) -> Option<u32> {
        self.hash_map.get(s).copied()
    }
    
    pub fn count(&self) -> usize { self.strings.len() }
    pub fn total_count(&self) -> u32 { self.total_count }
    
    pub fn serialize(&self) -> Result<Bytes> {
        let mut writer = BufferWriter::new(1024);
        
        writer.write_u32_le(RecordType::BrtBeginSst.to_u32());
        writer.write_u32_le(0);
        
        for s in &self.strings {
            let string_size = BufferWriter::utf16le_byte_length(s) + BufferWriter::varint_size(s.encode_utf16().count() as u32);
            writer.write_u32_le(RecordType::BrtSstItem.to_u32());
            writer.write_u32_le(string_size as u32);
            writer.write_wide_string(s);
        }
        
        writer.write_u32_le(RecordType::BrtEndSst.to_u32());
        writer.write_u32_le(0);
        
        Ok(writer.freeze())
    }
}