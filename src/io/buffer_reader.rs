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
}