use crate::error::Result;

pub struct XlsbWriter;

impl XlsbWriter {
    pub fn builder() -> XlsbWriterBuilder { XlsbWriterBuilder }
}

pub struct XlsbWriterBuilder;

impl XlsbWriterBuilder {
    pub fn path(self, _p: impl Into<std::path::PathBuf>) -> Self { self }
    pub fn build(self) -> Result<XlsbWriter> { Ok(XlsbWriter) }
}