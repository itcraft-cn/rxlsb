use crate::error::Result;

pub struct XlsbContainerReader;

impl XlsbContainerReader {
    pub fn open(_path: &std::path::Path) -> Result<Self> { Ok(Self) }
}