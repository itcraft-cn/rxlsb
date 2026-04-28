use crate::error::Result;

pub struct XlsbReader;

impl XlsbReader {
    pub fn builder() -> XlsbReaderBuilder { XlsbReaderBuilder }
}

pub struct XlsbReaderBuilder;

impl XlsbReaderBuilder {
    pub fn path(self, _p: impl Into<std::path::PathBuf>) -> Self { self }
    pub fn build(self) -> Result<XlsbReader> { Ok(XlsbReader) }
}