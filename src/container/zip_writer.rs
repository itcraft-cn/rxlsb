use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use bytes::Bytes;
use zip::{CompressionMethod, ZipWriter, write::FileOptions};

use crate::error::Result;

/// 默认压缩级别（速度优先）。
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
            entries: Vec::new(),
            compression_level,
        })
    }

    pub fn set_compression_level(&mut self, level: i32) {
        self.compression_level = level;
    }

    fn options(&self) -> FileOptions {
        let (method, level) = if self.compression_level > 0 {
            (CompressionMethod::Deflated, Some(self.compression_level))
        } else {
            (CompressionMethod::Stored, None)
        };
        FileOptions::default()
            .compression_method(method)
            .compression_level(level)
    }

    /// 一次性写入条目。
    pub fn add_entry(&mut self, name: &str, data: &[u8]) -> Result<()> {
        self.begin_entry(name)?;
        self.writer.write_all(data)?;
        self.end_entry()
    }

    /// 开始一个条目（随后可多次 `write_entry_chunk` 流式写入）。
    pub fn begin_entry(&mut self, name: &str) -> Result<()> {
        self.writer.start_file(name, self.options())?;
        self.entries.push(name.to_string());
        Ok(())
    }

    /// 追加一段数据到当前条目。
    pub fn write_entry_chunk(&mut self, data: &[u8]) -> Result<()> {
        if !data.is_empty() {
            self.writer.write_all(data)?;
        }
        Ok(())
    }

    /// 结束当前条目。
    ///
    /// `zip` crate 会在下一个 `start_file` 或 `finish` 时自动收尾，这里无需额外动作。
    pub fn end_entry(&mut self) -> Result<()> {
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
