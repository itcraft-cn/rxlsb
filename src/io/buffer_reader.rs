use bytes::Bytes;
use crate::error::{XlsbError, Result};

pub struct BufferReader {
    buffer: Bytes,
    position: usize,
}

impl BufferReader {
    pub fn new(buffer: Bytes) -> Self {
        Self { buffer, position: 0 }
    }
    
    pub fn read_u8(&mut self) -> Result<u8> {
        if self.position >= self.buffer.len() {
            return Err(XlsbError::BufferOverflow {
                position: self.position,
                length: self.buffer.len(),
            });
        }
        let b = self.buffer[self.position];
        self.position += 1;
        Ok(b)
    }
    
    pub fn read_u16_le(&mut self) -> Result<u16> {
        if self.position + 2 > self.buffer.len() {
            return Err(XlsbError::BufferOverflow {
                position: self.position,
                length: self.buffer.len(),
            });
        }
        let bytes = &self.buffer[self.position..self.position + 2];
        let value = u16::from_le_bytes([bytes[0], bytes[1]]);
        self.position += 2;
        Ok(value)
    }
    
    pub fn read_u32_le(&mut self) -> Result<u32> {
        if self.position + 4 > self.buffer.len() {
            return Err(XlsbError::BufferOverflow {
                position: self.position,
                length: self.buffer.len(),
            });
        }
        let bytes = &self.buffer[self.position..self.position + 4];
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        self.position += 4;
        Ok(value)
    }
    
    pub fn read_u64_le(&mut self) -> Result<u64> {
        if self.position + 8 > self.buffer.len() {
            return Err(XlsbError::BufferOverflow {
                position: self.position,
                length: self.buffer.len(),
            });
        }
        let bytes = &self.buffer[self.position..self.position + 8];
        let value = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        self.position += 8;
        Ok(value)
    }
    
    pub fn read_f64_le(&mut self) -> Result<f64> {
        let bits = self.read_u64_le()?;
        Ok(f64::from_bits(bits))
    }
    
    #[allow(dead_code)]
    pub fn read_i32_le(&mut self) -> Result<i32> {
        let u = self.read_u32_le()?;
        Ok(u as i32)
    }
    
    pub fn read_bytes(&mut self, len: usize) -> Result<Bytes> {
        if self.position + len > self.buffer.len() {
            return Err(XlsbError::BufferOverflow {
                position: self.position,
                length: self.buffer.len(),
            });
        }
        let slice = self.buffer.slice(self.position..self.position + len);
        self.position += len;
        Ok(slice)
    }
    
    pub fn skip(&mut self, len: usize) -> Result<()> {
        if self.position + len > self.buffer.len() {
            return Err(XlsbError::BufferOverflow {
                position: self.position,
                length: self.buffer.len(),
            });
        }
        self.position += len;
        Ok(())
    }
    
    pub fn read_varint(&mut self) -> Result<u32> {
        let b0 = self.read_u8()?;
        if (b0 & 0x80) == 0 {
            return Ok(b0 as u32);
        }
        let b1 = self.read_u8()?;
        Ok(((b0 & 0x7F) as u32) | ((b1 as u32) << 7))
    }
    
    pub fn read_varsize(&mut self) -> Result<u32> {
        let b0 = self.read_u8()?;
        if (b0 & 0x80) == 0 {
            return Ok(b0 as u32);
        }
        let b1 = self.read_u8()?;
        if (b1 & 0x80) == 0 {
            return Ok(((b0 & 0x7F) as u32) | ((b1 as u32) << 7));
        }
        let b2 = self.read_u8()?;
        if (b2 & 0x80) == 0 {
            return Ok(((b0 & 0x7F) as u32) | (((b1 & 0x7F) as u32) << 7) | ((b2 as u32) << 14));
        }
        let b3 = self.read_u8()?;
        Ok(((b0 & 0x7F) as u32) | (((b1 & 0x7F) as u32) << 7) | (((b2 & 0x7F) as u32) << 14) | ((b3 as u32) << 21))
    }
    
#[allow(dead_code)]
    pub fn read_wide_string(&mut self) -> Result<String> {
        let char_count = self.read_varint()? as usize;
        let byte_count = char_count * 2;
        
        let bytes = self.read_bytes(byte_count)?;
        
        let chars: Vec<u16> = bytes.chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        
        String::from_utf16(&chars)
            .map_err(|_| XlsbError::InvalidUtf16)
    }
    
    pub fn read_wide_string_u32(&mut self) -> Result<String> {
        let char_count = self.read_u32_le()? as usize;
        self.read_utf16_le_string(char_count)
    }

    /// 读取 `char_count` 个 UTF-16LE 码元并解码为 `String`。
    ///
    /// 只做一次边界检查、直接借用内部切片并顺序填充，避免逐码元调用
    /// `read_u16_le` 的重复边界检查，也避免 `Bytes::slice` 的引用计数开销。
    pub fn read_utf16_le_string(&mut self, char_count: usize) -> Result<String> {
        let byte_len = char_count
            .checked_mul(2)
            .ok_or(XlsbError::BufferOverflow {
                position: self.position,
                length: self.buffer.len(),
            })?;
        let end = self.position + byte_len;
        if end > self.buffer.len() {
            return Err(XlsbError::BufferOverflow {
                position: self.position,
                length: self.buffer.len(),
            });
        }

        let src = &self.buffer[self.position..end];
        let mut units: Vec<u16> = Vec::with_capacity(char_count);
        let mut i = 0;
        while i < byte_len {
            units.push(u16::from_le_bytes([src[i], src[i + 1]]));
            i += 2;
        }
        self.position = end;

        String::from_utf16(&units).map_err(|_| XlsbError::InvalidUtf16)
    }
    
    #[allow(dead_code)]
    pub fn position(&self) -> usize { self.position }
    #[allow(dead_code)]
    pub fn remaining(&self) -> usize { self.buffer.len() - self.position }
    pub fn has_remaining(&self) -> bool { self.position < self.buffer.len() }
    #[allow(dead_code)]
    pub fn len(&self) -> usize { self.buffer.len() }
}