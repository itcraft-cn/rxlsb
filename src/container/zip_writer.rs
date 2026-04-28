use crate::error::Result;

pub struct XlsbContainerWriter;

impl XlsbContainerWriter {
    pub fn create(_path: &std::path::Path) -> Result<Self> { Ok(Self) }
    pub fn add_entry(&mut self, _name: &str, _data: &[u8]) -> Result<()> { Ok(()) }
    pub fn finish(&mut self) -> Result<()> { Ok(()) }
}