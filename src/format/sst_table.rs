use std::collections::HashMap;
use crate::io::{BufferWriter, BufferReader};
use crate::format::RecordType;
use bytes::Bytes;
use crate::error::Result;

#[derive(Clone)]
pub struct SstTable {
    strings: Vec<String>,
    hash_map: HashMap<String, u32>,
    total_count: u32,
}

impl SstTable {
    pub fn new() -> Self {
        Self { strings: vec![], hash_map: HashMap::new(), total_count: 0 }
    }
    
    #[allow(dead_code)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            strings: Vec::with_capacity(capacity),
            hash_map: HashMap::with_capacity(capacity),
            total_count: 0,
        }
    }
    
    pub fn add_string(&mut self, s: &str) -> u32 {
        self.total_count += 1;
        
        if let Some(idx) = self.hash_map.get(s) {
            return *idx;
        }
        
        let idx = self.strings.len() as u32;
        let s_owned = s.to_string();
        self.strings.push(s_owned.clone());
        self.hash_map.insert(s_owned, idx);
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
        
        writer.write_varint(RecordType::BrtBeginSst.to_u32());
        writer.write_varsize(8);
        writer.write_u32_le(self.total_count);
        writer.write_u32_le(self.strings.len() as u32);
        
        for s in &self.strings {
            let str_len = s.encode_utf16().count();
            let record_size = 1 + 4 + str_len * 2;
            
            writer.write_varint(RecordType::BrtSstItem.to_u32());
            writer.write_varsize(record_size as u32);
            writer.write_u8(0);
            writer.write_u32_le(str_len as u32);
            for ch in s.encode_utf16() {
                writer.write_u16_le(ch);
            }
        }
        
        writer.write_varint(RecordType::BrtEndSst.to_u32());
        writer.write_varsize(0);
        
        Ok(writer.freeze())
    }
    
    pub fn deserialize(data: Bytes) -> Result<Self> {
        let mut reader = BufferReader::new(data);
        let mut strings = Vec::new();
        
        while reader.has_remaining() {
            let record_type_code = reader.read_varint()?;
            let size = reader.read_varsize()?;
            
            let record_type = RecordType::from_u32(record_type_code);
            
            match record_type {
                Some(RecordType::BrtBeginSst) => {
                    reader.skip(size as usize)?;
                }
                
                Some(RecordType::BrtSstItem) => {
                    reader.skip(1)?;
                    let char_count = reader.read_u32_le()? as usize;
                    let mut chars = Vec::with_capacity(char_count);
                    for _ in 0..char_count {
                        chars.push(reader.read_u16_le()?);
                    }
                    let s = String::from_utf16(&chars)
                        .map_err(|_| crate::error::XlsbError::InvalidUtf16)?;
                    strings.push(s);
                }
                
                Some(RecordType::BrtEndSst) => {
                    break;
                }
                
                _ => {
                    reader.skip(size as usize)?;
                }
            }
        }
        
        let hash_map = strings.iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i as u32))
            .collect();
        
        let total_count = strings.len() as u32;
        
        Ok(Self {
            strings,
            hash_map,
            total_count,
        })
    }
}