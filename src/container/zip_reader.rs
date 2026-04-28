use zip::ZipArchive;
use std::fs::File;
use std::path::Path;
use bytes::Bytes;
use crate::error::Result;

pub struct XlsbContainerReader {
    archive: ZipArchive<File>,
}

impl XlsbContainerReader {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let archive = ZipArchive::new(file)?;
        Ok(Self { archive })
    }
    
    pub fn has_entry(&mut self, name: &str) -> bool {
        self.archive.by_name(name).is_ok()
    }
    
    pub fn read_entry(&mut self, name: &str) -> Result<Bytes> {
        let mut file = self.archive.by_name(name)?;
        let size = file.size() as usize;
        let mut buffer = Vec::with_capacity(size);
        std::io::copy(&mut file, &mut buffer)?;
        Ok(Bytes::from(buffer))
    }
    
    pub fn get_workbook_data(&mut self) -> Result<Bytes> {
        self.read_entry("xl/workbook.bin")
    }
    
    pub fn get_styles_data(&mut self) -> Result<Bytes> {
        self.read_entry("xl/styles.bin")
    }
    
    pub fn get_sst_data(&mut self) -> Result<Option<Bytes>> {
        if self.has_entry("xl/sharedStrings.bin") {
            self.read_entry("xl/sharedStrings.bin").map(Some)
        } else {
            Ok(None)
        }
    }
    
    pub fn get_sheet_data(&mut self, sheet_idx: usize) -> Result<Bytes> {
        let name = format!("xl/worksheets/sheet{}.bin", sheet_idx + 1);
        self.read_entry(&name)
    }
    
    pub fn entry_names(&self) -> Vec<String> {
        self.archive.file_names().map(|s| s.to_string()).collect()
    }
    
    pub fn entry_count(&self) -> usize {
        self.archive.len()
    }
}