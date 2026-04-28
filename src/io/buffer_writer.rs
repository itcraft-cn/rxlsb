use bytes::{BytesMut, BufMut, Bytes};

#[derive(Clone)]
pub struct BufferWriter {
    buffer: BytesMut,
}

impl BufferWriter {
    pub fn new(capacity: usize) -> Self {
        Self { buffer: BytesMut::with_capacity(capacity) }
    }
    
    pub fn write_u8(&mut self, v: u8) {
        self.buffer.put_u8(v);
    }
    
    pub fn write_u16_le(&mut self, v: u16) {
        self.buffer.put_u16_le(v);
    }
    
    pub fn write_u32_le(&mut self, v: u32) {
        self.buffer.put_u32_le(v);
    }
    
    pub fn write_u64_le(&mut self, v: u64) {
        self.buffer.put_u64_le(v);
    }
    
    pub fn write_f64_le(&mut self, v: f64) {
        self.buffer.put_f64_le(v);
    }
    
    pub fn write_i32_le(&mut self, v: i32) {
        self.buffer.put_i32_le(v);
    }
    
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer.put_slice(bytes);
    }
    
    pub fn write_varint(&mut self, value: u32) -> usize {
        if value >= 128 {
            self.write_u8(((value & 0x7F) as u8) | 0x80);
            self.write_u8(((value >> 7) as u8) & 0x7F);
            2
        } else {
            self.write_u8(value as u8);
            1
        }
    }
    
    pub fn write_varsize(&mut self, value: u32) -> usize {
        if value < 128 {
            self.write_u8(value as u8);
            1
        } else if value < 16384 {
            self.write_u8(((value & 0x7F) as u8) | 0x80);
            self.write_u8(((value >> 7) as u8) & 0x7F);
            2
        } else if value < 2097152 {
            self.write_u8(((value & 0x7F) as u8) | 0x80);
            self.write_u8((((value >> 7) & 0x7F) as u8) | 0x80);
            self.write_u8(((value >> 14) as u8) & 0x7F);
            3
        } else {
            self.write_u8(((value & 0x7F) as u8) | 0x80);
            self.write_u8((((value >> 7) & 0x7F) as u8) | 0x80);
            self.write_u8((((value >> 14) & 0x7F) as u8) | 0x80);
            self.write_u8(((value >> 21) as u8) & 0x7F);
            4
        }
    }
    
    pub fn write_wide_string(&mut self, s: &str) -> usize {
        let utf16_chars: Vec<u16> = s.encode_utf16().collect();
        let char_count = utf16_chars.len() as u32;
        
        let varint_size = self.write_varint(char_count);
        
        for ch in utf16_chars {
            self.write_u16_le(ch);
        }
        
        varint_size + char_count as usize * 2
    }
    
    pub fn utf16le_byte_length(s: &str) -> usize {
        s.encode_utf16().count() * 2
    }
    
    pub fn varint_size(value: u32) -> usize {
        if value < 0x80 { 1 }
        else if value < 0x4000 { 2 }
        else if value < 0x200000 { 3 }
        else if value < 0x10000000 { 4 }
        else { 5 }
    }
    
    pub fn len(&self) -> usize { self.buffer.len() }
    pub fn capacity(&self) -> usize { self.buffer.capacity() }
    pub fn remaining_capacity(&self) -> usize { self.buffer.capacity() - self.buffer.len() }
    
    pub fn ensure_capacity(&mut self, additional: usize) {
        if self.remaining_capacity() < additional {
            self.buffer.reserve(additional);
        }
    }
    
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    
    pub fn freeze(self) -> Bytes {
        self.buffer.freeze()
    }
}