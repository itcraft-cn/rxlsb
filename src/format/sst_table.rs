use std::collections::HashMap;
use std::sync::Arc;
use crate::io::{BufferWriter, BufferReader};
use crate::format::RecordType;
use bytes::Bytes;
use crate::error::Result;

#[derive(Clone)]
pub struct SstTable {
    strings: Vec<Arc<str>>,
    hash_map: HashMap<Arc<str>, u32>,
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
        let shared: Arc<str> = Arc::from(s);
        self.strings.push(shared.clone());
        self.hash_map.insert(shared, idx);
        idx
    }

    /// 以「所有权」方式加入字符串，避免对已拥有的 `String` 再做一次拷贝。
    pub fn add_string_owned(&mut self, s: String) -> u32 {
        self.total_count += 1;

        if let Some(idx) = self.hash_map.get(s.as_str()) {
            return *idx;
        }

        let idx = self.strings.len() as u32;
        // Arc<str> 会一次性构建共享字符串；相比 `String` + `String` 的两份分配，
        // 这里只保留一份共享存储（Vec 与 HashMap 共享同一个 Arc）。
        let shared: Arc<str> = Arc::from(s);
        self.strings.push(shared.clone());
        self.hash_map.insert(shared, idx);
        idx
    }
    
    pub fn get_string(&self, idx: u32) -> Option<&str> {
        self.strings.get(idx as usize).map(|s| s.as_ref())
    }
    
    pub fn find_string(&self, s: &str) -> Option<u32> {
        self.hash_map.get(s).copied()
    }
    
    pub fn count(&self) -> usize { self.strings.len() }
    pub fn total_count(&self) -> u32 { self.total_count }

    /// 预留容量，减少写入过程中的 rehash 与扩容。
    pub fn reserve(&mut self, additional: usize) {
        self.strings.reserve(additional);
        self.hash_map.reserve(additional);
    }
    
    pub fn serialize(&self) -> Result<Bytes> {
        // 预估容量：UTF-16 上限约 2 字节/字符 + 每条记录头开销。
        let capacity = 64 + self.strings.iter().map(|s| s.len() * 2 + 8).sum::<usize>();
        let mut writer = BufferWriter::new(capacity);
        
        writer.write_varint(RecordType::BrtBeginSst.to_u32());
        writer.write_varsize(8);
        writer.write_u32_le(self.total_count);
        writer.write_u32_le(self.strings.len() as u32);
        
        // 复用一个 scratch 缓冲，避免每条字符串都分配一次 Vec<u16>。
        let mut utf16: Vec<u16> = Vec::new();
        for s in &self.strings {
            utf16.clear();
            utf16.extend(s.encode_utf16());
            let record_size = 1 + 4 + utf16.len() * 2;
            
            writer.write_varint(RecordType::BrtSstItem.to_u32());
            writer.write_varsize(record_size as u32);
            writer.write_u8(0);
            writer.write_u32_le(utf16.len() as u32);
            for &ch in &utf16 {
                writer.write_u16_le(ch);
            }
        }
        
        writer.write_varint(RecordType::BrtEndSst.to_u32());
        writer.write_varsize(0);
        
        Ok(writer.freeze())
    }
    
    pub fn deserialize(data: Bytes) -> Result<Self> {
        let mut reader = BufferReader::new(data);
        let mut strings: Vec<Arc<str>> = Vec::new();
        
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
                    strings.push(Arc::from(s));
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