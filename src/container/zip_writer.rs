use zip::{ZipWriter, CompressionMethod, write::FileOptions};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use bytes::Bytes;
use crate::error::Result;

/// 默认压缩级别。
///
/// 与主流表格库（如 `spreadsheet`）保持一致，采用速度优先的 level 1，
/// 在保证兼容性的前提下获得最佳的写入吞吐。需要更小文件时可通过
/// [`XlsbWriter::set_compression_level`](crate::XlsbWriter::set_compression_level)
/// 调整（例如 6）。
pub const DEFAULT_COMPRESSION_LEVEL: i32 = 1;

/// 输出缓冲区大小，减少底层文件的小块 write 系统调用。
const OUTPUT_BUFFER_SIZE: usize = 256 * 1024;

pub struct XlsbContainerWriter {
    writer: ZipWriter<BufWriter<File>>,
    entries: Vec<String>,
    compression_level: i32,
}

impl XlsbContainerWriter {
    pub fn create(path: &Path) -> Result<Self> {
        Self::create_with_level(path, DEFAULT_COMPRESSION_LEVEL)
    }

    pub fn create_with_level(path: &Path, compression_level: i32) -> Result<Self> {
        let file = File::create(path)?;
        let buffered = BufWriter::with_capacity(OUTPUT_BUFFER_SIZE, file);
        Ok(Self {
            writer: ZipWriter::new(buffered),
            entries: vec![],
            compression_level,
        })
    }

    pub fn set_compression_level(&mut self, level: i32) {
        self.compression_level = level;
    }

    pub fn add_entry(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(self.compression_level));

        self.writer.start_file(name, options)?;
        self.writer.write_all(data)?;
        self.entries.push(name.to_string());
        Ok(())
    }

    pub fn add_entry_from_bytes(&mut self, name: &str, data: &Bytes) -> Result<()> {
        self.add_entry(name, data.as_ref())
    }

    pub fn add_entry_from_str(&mut self, name: &str, data: &str) -> Result<()> {
        self.add_entry(name, data.as_bytes())
    }

    pub fn finish(&mut self) -> Result<()> {
        let mut out = self.writer.finish()?;
        out.flush()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    #[allow(dead_code)]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}
