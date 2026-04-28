use crate::error::Result;

pub struct BufferReader;
pub struct BufferWriter;

impl BufferWriter {
    pub fn write_wide_string(&mut self, _s: &str) -> usize { 0 }
}

impl BufferReader {
    pub fn read_wide_string(&mut self) -> Result<String> { Ok(String::new()) }
}