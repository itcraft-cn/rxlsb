use crate::error::Result;

pub struct BufferReader;
pub struct BufferWriter;

impl BufferWriter {
    pub fn write_varint(&mut self, _value: u32) -> usize { 0 }
}

impl BufferReader {
    pub fn read_varint(&mut self) -> Result<u32> { Ok(0) }
}