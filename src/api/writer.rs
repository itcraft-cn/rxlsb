use std::path::PathBuf;
use crate::container::{XlsbContainerWriter};
use crate::format::{SstTable, StylesRegistry, WorkbookWriter, SheetWriter};
use crate::api::{CellData, CellSupplier};
use crate::container::{XmlGen, RelsGen};
use crate::error::{XlsbError, Result};

pub struct XlsbWriter {
    #[allow(dead_code)]
    path: PathBuf,
    container: XlsbContainerWriter,
    sst: SstTable,
    styles: StylesRegistry,
    workbook: WorkbookWriter,
    sheets_data: Vec<Vec<u8>>,
    streaming: Option<StreamingState>,
}

struct StreamingState {
    sheet_name: String,
    col_count: usize,
    max_row: usize,
    rows_data: Vec<Vec<CellData>>,
}

impl XlsbWriter {
    pub fn builder() -> XlsbWriterBuilder {
        XlsbWriterBuilder { path: None }
    }
    
    fn new(path: &std::path::Path) -> Result<Self> {
        let container = XlsbContainerWriter::create(path)?;
        let sst = SstTable::new();
        let styles = StylesRegistry::new();
        let workbook = WorkbookWriter::new();
        
        Ok(Self {
            path: path.to_path_buf(),
            container,
            sst,
            styles,
            workbook,
            sheets_data: vec![],
            streaming: None,
        })
    }
    
    pub fn write_batch(&mut self, sheet_name: &str,
                       supplier: impl CellSupplier,
                       row_count: usize, col_count: usize) -> Result<()> {
        let mut sheet_writer = SheetWriter::new(&mut self.sst, &mut self.styles);
        sheet_writer.write_batch(supplier, row_count, col_count)?;
        
        self.workbook.add_sheet(sheet_name);
        self.sheets_data.push(sheet_writer.serialize().as_ref().to_vec());
        Ok(())
    }
    
    pub fn start_sheet(&mut self, sheet_name: &str, col_count: usize) -> Result<()> {
        if self.streaming.is_some() {
            return Err(XlsbError::InvalidState("Previous sheet not ended, call end_sheet() first".into()));
        }
        
        self.streaming = Some(StreamingState {
            sheet_name: sheet_name.to_string(),
            col_count,
            max_row: 0,
            rows_data: Vec::new(),
        });
        Ok(())
    }
    
    pub fn write_rows(&mut self, supplier: impl CellSupplier, start_row: usize, row_count: usize) -> Result<()> {
        let streaming = self.streaming.as_mut()
            .ok_or_else(|| XlsbError::InvalidState("Sheet not started, call start_sheet() first".into()))?;
        
        let end_row = start_row + row_count;
        if end_row > streaming.rows_data.len() {
            streaming.rows_data.resize(end_row, vec![CellData::Blank; streaming.col_count]);
        }
        
        for row_idx in 0..row_count {
            let abs_row = start_row + row_idx;
            for col in 0..streaming.col_count {
                let cell = supplier.get_cell(row_idx, col);
                streaming.rows_data[abs_row][col] = cell;
            }
        }
        
        streaming.max_row = streaming.max_row.max(end_row);
        Ok(())
    }
    
    pub fn end_sheet(&mut self) -> Result<()> {
        let streaming = self.streaming.take()
            .ok_or_else(|| XlsbError::InvalidState("Sheet not started".into()))?;
        
        struct VecSupplier {
            data: Vec<Vec<CellData>>,
        }
        impl CellSupplier for VecSupplier {
            fn get_cell(&self, row: usize, col: usize) -> CellData {
                self.data.get(row)
                    .and_then(|r| r.get(col))
                    .cloned()
                    .unwrap_or(CellData::Blank)
            }
        }
        
        let supplier = VecSupplier { data: streaming.rows_data };
        let mut sheet_writer = SheetWriter::new(&mut self.sst, &mut self.styles);
        sheet_writer.write_batch(supplier, streaming.max_row, streaming.col_count)?;
        
        self.workbook.add_sheet(&streaming.sheet_name);
        self.sheets_data.push(sheet_writer.serialize().as_ref().to_vec());
        
        Ok(())
    }
    
    pub fn close(&mut self) -> Result<()> {
        let has_sst = self.sst.count() > 0;
        let sheet_count = self.workbook.sheet_count();
        
        self.container.add_entry_from_bytes("[Content_Types].xml",
            &XmlGen::content_types(sheet_count, has_sst))?;
        self.container.add_entry_from_str("_rels/.rels", &String::from_utf8_lossy(&RelsGen::root_rels()))?;
        self.container.add_entry_from_bytes("docProps/app.xml",
            &XmlGen::app_xml(sheet_count))?;
        self.container.add_entry_from_bytes("docProps/core.xml",
            &XmlGen::core_xml())?;
        self.container.add_entry_from_bytes("xl/theme/theme1.xml",
            &XmlGen::theme_xml())?;
        
        let workbook_data = self.workbook.serialize()?;
        self.container.add_entry_from_bytes("xl/workbook.bin", &workbook_data)?;
        
        self.container.add_entry_from_bytes("xl/_rels/workbook.bin.rels",
            &RelsGen::workbook_rels(sheet_count, has_sst))?;
        
        let styles_data = self.styles.serialize()?;
        self.container.add_entry_from_bytes("xl/styles.bin", &styles_data)?;
        
        if has_sst {
            let sst_data = self.sst.serialize()?;
            self.container.add_entry_from_bytes("xl/sharedStrings.bin", &sst_data)?;
        }
        
        for (i, sheet_data) in self.sheets_data.iter().enumerate() {
            self.container.add_entry(&format!("xl/worksheets/sheet{}.bin", i + 1), sheet_data)?;
        }
        
        self.container.finish()?;
        Ok(())
    }
}

pub struct XlsbWriterBuilder {
    path: Option<PathBuf>,
}

impl XlsbWriterBuilder {
    pub fn path(mut self, p: impl Into<PathBuf>) -> Self {
        self.path = Some(p.into());
        self
    }
    
    pub fn build(self) -> Result<XlsbWriter> {
        let path = self.path.ok_or(XlsbError::PathNotSet)?;
        XlsbWriter::new(&path)
    }
}