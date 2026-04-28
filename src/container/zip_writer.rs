use zip::{ZipWriter, CompressionMethod, write::FileOptions};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use bytes::Bytes;
use crate::error::Result;

pub struct XlsbContainerWriter {
    writer: ZipWriter<File>,
    entries: Vec<String>,
}

impl XlsbContainerWriter {
    pub fn create(path: &Path) -> Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: ZipWriter::new(file),
            entries: vec![],
        })
    }
    
    pub fn add_entry(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Stored);
        
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
        self.writer.finish()?;
        Ok(())
    }
    
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
    
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}