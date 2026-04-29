use std::path::PathBuf;
use std::collections::HashMap;
use bytes::Bytes;
use crate::container::{XlsbContainerReader, XlsbContainerWriter};
use crate::format::{SstTable, StylesRegistry, WorkbookReader, WorkbookWriter, SheetWriter, SheetReader};
use crate::data::SheetInfo;
use crate::api::{CellData, CellSupplier, RowHandler};
use crate::container::{XmlGen, RelsGen};
use crate::error::{XlsbError, Result};

pub struct XlsbReader {
    container: XlsbContainerReader,
    sst: Option<SstTable>,
    styles: StylesRegistry,
    workbook: WorkbookReader,
    sheet_cache: HashMap<usize, Bytes>,
}

impl XlsbReader {
    pub fn builder() -> XlsbReaderBuilder {
        XlsbReaderBuilder { path: None }
    }
    
    fn new(path: &std::path::Path) -> Result<Self> {
        let mut container = XlsbContainerReader::open(path)?;
        
        let workbook_data = container.get_workbook_data()?;
        let workbook = WorkbookReader::deserialize(workbook_data)?;
        
        let styles = StylesRegistry::new();
        
        let sst = if container.has_entry("xl/sharedStrings.bin") {
            let sst_data = container.get_sst_data()?.unwrap();
            Some(SstTable::deserialize(sst_data)?)
        } else {
            None
        };
        
        Ok(Self { 
            container, 
            sst, 
            styles, 
            workbook,
            sheet_cache: HashMap::new(),
        })
    }
    
    fn get_cached_sheet(&mut self, sheet_idx: usize) -> Result<Bytes> {
        if let Some(cached) = self.sheet_cache.get(&sheet_idx) {
            return Ok(cached.clone());
        }
        
        let sheet_data = self.container.get_sheet_data(sheet_idx)?;
        self.sheet_cache.insert(sheet_idx, sheet_data.clone());
        Ok(sheet_data)
    }
    
    pub fn get_sheet_infos(&self) -> &[SheetInfo] {
        self.workbook.get_sheet_infos()
    }
    
    pub fn for_each_row(&mut self, sheet_idx: usize, handler: impl RowHandler) -> Result<()> {
        if sheet_idx >= self.workbook.sheet_count() {
            return Err(XlsbError::InvalidSheetIndex(sheet_idx));
        }
        
        let sheet_data = self.container.get_sheet_data(sheet_idx)?;
        let mut sheet_reader = SheetReader::new(sheet_data, self.sst.as_ref());
        sheet_reader.for_each_row(handler)?;
        Ok(())
    }
    
    pub fn read_rows(&mut self, sheet_idx: usize, start_row: usize, row_count: usize) -> Result<Vec<Vec<CellData>>> {
        if sheet_idx >= self.workbook.sheet_count() {
            return Err(XlsbError::InvalidSheetIndex(sheet_idx));
        }
        
        let sheet_data = self.get_cached_sheet(sheet_idx)?;
        let mut sheet_reader = SheetReader::new(sheet_data, self.sst.as_ref());
        sheet_reader.read_rows(start_row, row_count)
    }
}

pub struct XlsbReaderBuilder {
    path: Option<PathBuf>,
}

impl XlsbReaderBuilder {
    pub fn path(mut self, p: impl Into<PathBuf>) -> Self {
        self.path = Some(p.into());
        self
    }
    
    pub fn build(self) -> Result<XlsbReader> {
        let path = self.path.ok_or(XlsbError::PathNotSet)?;
        XlsbReader::new(&path)
    }
}