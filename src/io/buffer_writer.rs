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
    
    pub fn write_u24_le(&mut self, v: u32) {
        self.buffer.put_u8((v & 0xFF) as u8);
        self.buffer.put_u8(((v >> 8) & 0xFF) as u8);
        self.buffer.put_u8(((v >> 16) & 0xFF) as u8);
    }
    
    #[allow(dead_code)]
    pub fn write_u64_le(&mut self, v: u64) {
        self.buffer.put_u64_le(v);
    }
    
    pub fn write_f64_le(&mut self, v: f64) {
        self.buffer.put_f64_le(v);
    }
    
    #[allow(dead_code)]
    pub fn write_i32_le(&mut self, v: i32) {
        self.buffer.put_i32_le(v);
    }
    
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer.put_slice(bytes);
    }

    /// 一次性写入一段 UTF-16LE（小端平台直接整段 memcpy）。
    #[inline]
    pub fn write_u16_le_slice(&mut self, values: &[u16]) {
        #[cfg(target_endian = "little")]
        {
            // 小端平台上 [u16] 的内存布局即 UTF-16LE 字节序列。
            // SAFETY: u16 与 u8 对齐要求不同，这里按字节切片读取同一块内存，
            // 长度按元素数 * 2 计算，生命周期受 `values` 约束。
            let bytes = unsafe {
                std::slice::from_raw_parts(values.as_ptr() as *const u8, values.len() * 2)
            };
            self.buffer.put_slice(bytes);
        }
        #[cfg(not(target_endian = "little"))]
        {
            for &v in values {
                self.buffer.put_u16_le(v);
            }
        }
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
    
    #[allow(dead_code)]
    pub fn write_wide_string(&mut self, s: &str) -> usize {
        let utf16_chars: Vec<u16> = s.encode_utf16().collect();
        let char_count = utf16_chars.len() as u32;
        
        let varint_size = self.write_varint(char_count);
        
        for ch in utf16_chars {
            self.write_u16_le(ch);
        }
        
        varint_size + char_count as usize * 2
    }
    
    pub fn write_wide_string_u32(&mut self, s: &str) -> usize {
        let utf16_chars: Vec<u16> = s.encode_utf16().collect();
        let char_count = utf16_chars.len() as u32;
        
        self.write_u32_le(char_count);
        
        for ch in utf16_chars {
            self.write_u16_le(ch);
        }
        
        4 + char_count as usize * 2
    }
    
    pub fn utf16le_byte_length(s: &str) -> usize {
        s.encode_utf16().count() * 2
    }
    
    #[allow(dead_code)]
    pub fn varint_size(value: u32) -> usize {
        if value < 0x80 { 1 }
        else if value < 0x4000 { 2 }
        else if value < 0x200000 { 3 }
        else if value < 0x10000000 { 4 }
        else { 5 }
    }
    
    pub fn len(&self) -> usize { self.buffer.len() }
    #[allow(dead_code)]
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

    /// 消费自身，取出内部字节缓冲区（零额外拷贝）。
    pub fn into_bytes(self) -> Bytes {
        self.buffer.freeze()
    }
}