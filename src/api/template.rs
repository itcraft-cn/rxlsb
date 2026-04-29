use std::path::{PathBuf, Path};
use zip::ZipArchive;
use std::fs::File;
use crate::error::{XlsbError, Result};
use crate::format::{SheetParser, SstTable, CellInfo, CellValue, MergeCell};
use crate::api::{CellData, CellSupplier};
use crate::io::{BufferWriter, BufferReader};
use crate::container::XlsbContainerWriter;
use bytes::Bytes;

pub struct FillResult {
    pub start_row: u32,
    pub start_col: u32,
    pub row_count: u32,
    pub col_count: u32,
    pub data: Vec<Vec<CellData>>,
}

pub struct SheetData {
    pub name: String,
    pub parser: SheetParser,
    pub fill_result: Option<FillResult>,
}

pub struct StreamingState {
    sheet_index: usize,
    start_row: u32,
    start_col: u32,
    col_count: u32,
    template_cells: Vec<CellInfo>,
    template_merges: Vec<MergeCell>,
    template_max_row: u32,
    template_max_col: u32,
    accumulated_data: Vec<Vec<CellData>>,
}

pub struct TemplateFiller {
    template_path: PathBuf,
    output_path: PathBuf,
    sheets: Vec<SheetData>,
    sst: SstTable,
    streaming: Option<StreamingState>,
}

impl TemplateFiller {
    pub fn builder() -> TemplateFillerBuilder {
        TemplateFillerBuilder { template: None, output: None }
    }
    
    fn load_template(template_path: &Path) -> Result<(Vec<SheetData>, SstTable)> {
        let file = File::open(template_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        let sst_data = Self::load_entry(&mut archive, "xl/sharedStrings.bin")?;
        let sst = Self::parse_sst(&sst_data);
        
        let sst_strings: Vec<String> = (0..sst.count())
            .map(|i| sst.get_string(i as u32).unwrap_or("").to_string())
            .collect();
        let sst_refs: Vec<&str> = sst_strings.iter().map(|s| s.as_str()).collect();
        
        let wb_data = Self::load_entry(&mut archive, "xl/workbook.bin")?;
        let sheet_count = Self::count_sheets(&wb_data);
        
        let mut sheets = Vec::with_capacity(sheet_count);
        for i in 0..sheet_count {
            let path = format!("xl/worksheets/sheet{}.bin", i + 1);
            let sheet_data = Self::load_entry(&mut archive, &path)?;
            let parser = SheetParser::parse(&sheet_data, &sst_refs);
            
            sheets.push(SheetData {
                name: format!("Sheet{}", i + 1),
                parser,
                fill_result: None,
            });
        }
        
        Ok((sheets, sst))
    }
    
    fn load_entry(archive: &mut ZipArchive<File>, name: &str) -> Result<Vec<u8>> {
        let mut file = archive.by_name(name)?;
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut data)?;
        Ok(data)
    }
    
    fn parse_sst(data: &[u8]) -> SstTable {
        let mut sst = SstTable::new();
        let reader = BufferReader::new(Bytes::copy_from_slice(data));
        let mut pos = 0;
        
        while pos + 2 < data.len() {
            let record_type = if data[pos] >= 128 {
                pos += 2;
                ((data[pos-2] & 0x7F) as u32) | ((data[pos-1] & 0x7F) << 7) as u32
            } else {
                pos += 1;
                data[pos-1] as u32
            };
            
            let record_size = if data[pos] >= 128 {
                pos += 2;
                ((data[pos-2] & 0x7F) as u32) | ((data[pos-1] as u32) << 7)
            } else {
                pos += 1;
                data[pos-1] as u32
            };
            
            if pos + record_size as usize > data.len() { break; }
            
            if record_type == crate::format::RecordType::BrtSstItem.to_u32() {
                let flags = data[pos];
                let char_count = u32::from_le_bytes([data[pos+1], data[pos+2], data[pos+3], data[pos+4]]);
                let mut chars = Vec::with_capacity(char_count as usize);
                for j in 0..char_count as usize {
                    let ch = u16::from_le_bytes([data[pos+5+j*2], data[pos+6+j*2]]);
                    chars.push(ch);
                }
                let text = String::from_utf16(&chars).unwrap_or_default();
                sst.add_string(&text);
            }
            
            pos += record_size as usize;
        }
        
        sst
    }
    
    fn count_sheets(data: &[u8]) -> usize {
        let mut count = 0;
        for i in 0..data.len().saturating_sub(1) {
            if data[i] == 156 && data[i+1] == 0 {
                count += 1;
            }
        }
        if count == 0 { count = 1; }
        count
    }
    
    pub fn get_sheet_count(&self) -> usize { self.sheets.len() }
    
    pub fn get_sheet_names(&self) -> Vec<&str> {
        self.sheets.iter().map(|s| s.name.as_str()).collect()
    }
    
    pub fn find_marker(&self, sheet_index: usize, marker: &str) -> Option<(u32, u32)> {
        self.sheets.get(sheet_index)?.parser.find_text_cell(marker)
    }
    
    pub fn fill_batch<S: CellSupplier>(&mut self, sheet_index: usize, start_row: u32, start_col: u32,
                                        supplier: S, row_count: u32, col_count: u32) -> Result<()> {
        if sheet_index >= self.sheets.len() {
            return Err(XlsbError::InvalidArgument("sheet_index out of range"));
        }
        
        let mut data = Vec::with_capacity(row_count as usize);
        for r in 0..row_count {
            let mut row_data = Vec::with_capacity(col_count as usize);
            for c in 0..col_count {
                row_data.push(supplier.get_cell(r as usize, c as usize));
            }
            data.push(row_data);
        }
        
        self.sheets[sheet_index].fill_result = Some(FillResult {
            start_row, start_col, row_count, col_count, data,
        });
        
        Ok(())
    }
    
    pub fn fill_at_marker<S: CellSupplier>(&mut self, sheet_index: usize, marker: &str,
                                            supplier: S, row_count: u32, col_count: u32) -> Result<()> {
        let pos = self.find_marker(sheet_index, marker)
            .ok_or_else(|| XlsbError::MarkerNotFound(marker.to_string()))?;
        
        self.fill_batch(sheet_index, pos.0, pos.1, supplier, row_count, col_count)
    }
    
    pub fn start_fill(&mut self, sheet_index: usize, start_row: u32, start_col: u32, col_count: u32) -> Result<()> {
        if sheet_index >= self.sheets.len() {
            return Err(XlsbError::InvalidArgument("sheet_index out of range"));
        }
        
        if self.streaming.is_some() {
            return Err(XlsbError::InvalidArgument("Previous fill not ended, call end_fill first"));
        }
        
        let sheet = &self.sheets[sheet_index];
        let parser = &sheet.parser;
        
        self.streaming = Some(StreamingState {
            sheet_index,
            start_row,
            start_col,
            col_count,
            template_cells: parser.cells.clone(),
            template_merges: parser.merges.clone(),
            template_max_row: parser.max_row,
            template_max_col: parser.max_col,
            accumulated_data: Vec::new(),
        });
        
        Ok(())
    }
    
    pub fn fill_rows(&mut self, data: Vec<Vec<CellData>>) -> Result<()> {
        if self.streaming.is_none() {
            return Err(XlsbError::InvalidArgument("Fill not started, call start_fill first"));
        }
        
        let streaming = self.streaming.as_mut().unwrap();
        streaming.accumulated_data.extend(data);
        
        Ok(())
    }
    
    pub fn end_fill(&mut self) -> Result<()> {
        if self.streaming.is_none() {
            return Err(XlsbError::InvalidArgument("Fill not started"));
        }
        
        let streaming = self.streaming.take().unwrap();
        
        if streaming.accumulated_data.is_empty() {
            return Ok(());
        }
        
        let row_count = streaming.accumulated_data.len() as u32;
        
        self.sheets[streaming.sheet_index].fill_result = Some(FillResult {
            start_row: streaming.start_row,
            start_col: streaming.start_col,
            row_count,
            col_count: streaming.col_count,
            data: streaming.accumulated_data,
        });
        
        Ok(())
    }
    
    pub fn save(&mut self) -> Result<()> {
        let file = File::open(&self.template_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut container = XlsbContainerWriter::create(&self.output_path)?;
        
        let static_entries = [
            "[Content_Types].xml", "_rels/.rels",
            "docProps/app.xml", "docProps/core.xml",
            "xl/_rels/workbook.bin.rels", "xl/styles.bin",
            "xl/theme/theme1.xml", "xl/workbook.bin",
        ];
        
        for entry in &static_entries {
            let data = Bytes::copy_from_slice(&Self::load_entry(&mut archive, entry)?);
            container.add_entry_from_bytes(entry, &data)?;
        }
        
        let sheet_count = self.sheets.len();
        let mut sheet_datas = Vec::with_capacity(sheet_count);
        for i in 0..sheet_count {
            let sheet_data = self.generate_sheet(i)?;
            sheet_datas.push(sheet_data);
        }
        
        let sst_data = self.generate_sst()?;
        container.add_entry_from_bytes("xl/sharedStrings.bin", &sst_data)?;
        
        for (i, sheet_data) in sheet_datas.iter().enumerate() {
            let path = format!("xl/worksheets/sheet{}.bin", i + 1);
            container.add_entry_from_bytes(&path, sheet_data)?;
        }
        
        container.finish()?;
        Ok(())
    }
    
    fn generate_sst(&self) -> Result<Bytes> {
        let mut writer = BufferWriter::new(1024);
        
        writer.write_varint(crate::format::RecordType::BrtBeginSst.to_u32());
        writer.write_varsize(8);
        writer.write_u32_le(self.sst.total_count());
        writer.write_u32_le(self.sst.count() as u32);
        
        for i in 0..self.sst.count() {
            let text = self.sst.get_string(i as u32).unwrap_or("");
            let char_count = text.encode_utf16().count();
            
            writer.write_varint(crate::format::RecordType::BrtSstItem.to_u32());
            writer.write_varsize((1 + 4 + char_count * 2) as u32);
            writer.write_u8(0);
            writer.write_u32_le(char_count as u32);
            for ch in text.encode_utf16() { writer.write_u16_le(ch); }
        }
        
        writer.write_varint(crate::format::RecordType::BrtEndSst.to_u32());
        writer.write_varsize(0);
        
        Ok(writer.freeze())
    }
    
    fn generate_sheet(&mut self, sheet_index: usize) -> Result<Bytes> {
        let sheet = &self.sheets[sheet_index];
        let parser = &sheet.parser;
        let fill = sheet.fill_result.as_ref();
        
        let mut writer = BufferWriter::new(4096);
        
        writer.write_varint(crate::format::RecordType::BrtBeginSheet.to_u32());
        writer.write_varsize(0);
        
        Self::write_ws_prop(&mut writer);
        
        let max_row = parser.max_row.max(
            fill.map(|f| f.start_row + f.row_count - 1).unwrap_or(0)
        );
        let max_col = parser.max_col.max(
            fill.map(|f| f.start_col + f.col_count - 1).unwrap_or(0)
        );
        
        Self::write_ws_dim(&mut writer, 0, max_row, 0, max_col);
        Self::write_ws_views(&mut writer);
        Self::write_ws_fmt_info(&mut writer);
        
        writer.write_varint(crate::format::RecordType::BrtBeginSheetData.to_u32());
        writer.write_varsize(0);
        
        for row in 0..=max_row {
            Self::write_row_hdr(&mut writer, row, max_col + 1);
            
            if let Some(f) = fill {
                if row >= f.start_row && row < f.start_row + f.row_count {
                    for cell in &parser.cells {
                        if cell.row == row {
                            if cell.col < f.start_col || cell.col >= f.start_col + f.col_count {
                                Self::write_template_cell(&mut writer, &cell, &mut self.sst);
                            }
                        }
                    }
                    
                    let data_row = (row - f.start_row) as usize;
                    for col in f.start_col..f.start_col + f.col_count {
                        let data_col = (col - f.start_col) as usize;
                        if data_row < f.data.len() && data_col < f.data[data_row].len() {
                            Self::write_cell(&mut writer, col, &f.data[data_row][data_col], &mut self.sst);
                        }
                    }
                    continue;
                }
            }
            
            for cell in &parser.cells {
                if cell.row == row {
                    Self::write_template_cell(&mut writer, &cell, &mut self.sst);
                }
            }
        }
        
        writer.write_varint(crate::format::RecordType::BrtEndSheetData.to_u32());
        writer.write_varsize(0);
        
        if !parser.merges.is_empty() {
            Self::write_merges(&mut writer, &parser.merges);
        }
        
        writer.write_varint(crate::format::RecordType::BrtEndSheet.to_u32());
        writer.write_varsize(0);
        
        Ok(writer.freeze())
    }
    
    fn write_ws_prop(writer: &mut BufferWriter) {
        writer.write_varint(crate::format::RecordType::BrtWsProp.to_u32());
        writer.write_varsize(23);
        writer.write_bytes(&[
            0xC9, 0x04, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00
        ]);
    }
    
    fn write_ws_dim(writer: &mut BufferWriter, r1: u32, r2: u32, c1: u32, c2: u32) {
        writer.write_varint(crate::format::RecordType::BrtWsDim.to_u32());
        writer.write_varsize(16);
        writer.write_u32_le(r1);
        writer.write_u32_le(r2);
        writer.write_u32_le(c1);
        writer.write_u32_le(c2);
    }
    
    fn write_ws_views(writer: &mut BufferWriter) {
        writer.write_varint(crate::format::RecordType::BrtBeginWsViews.to_u32());
        writer.write_varsize(0);
        
        writer.write_varint(crate::format::RecordType::BrtBeginWsView.to_u32());
        writer.write_varsize(30);
        writer.write_bytes(&[
            0xDC, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x40, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]);
        
        writer.write_varint(crate::format::RecordType::BrtSel.to_u32());
        writer.write_varsize(36);
        writer.write_bytes(&[
            0x03, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
            0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00,
            0x07, 0x00, 0x00, 0x00
        ]);
        
        writer.write_varint(crate::format::RecordType::BrtEndWsView.to_u32());
        writer.write_varsize(0);
        writer.write_varint(crate::format::RecordType::BrtEndWsViews.to_u32());
        writer.write_varsize(0);
    }
    
    fn write_ws_fmt_info(writer: &mut BufferWriter) {
        writer.write_varint(crate::format::RecordType::BrtWsFmtInfo.to_u32());
        writer.write_varsize(12);
        writer.write_bytes(&[
            0x00, 0x09, 0x00, 0x00, 0x08, 0x00, 0x0E, 0x01,
            0x00, 0x00, 0x00, 0x00
        ]);
    }
    
    fn write_row_hdr(writer: &mut BufferWriter, row: u32, col_count: u32) {
        let num_spans = if col_count > 0 { (col_count - 1) / 1024 + 1 } else { 1 };
        let record_size = 17 + num_spans * 8;
        
        writer.write_varint(crate::format::RecordType::BrtRowHdr.to_u32());
        writer.write_varsize(record_size as u32);
        writer.write_u32_le(row);
        writer.write_bytes(&[0x00, 0x00, 0x00, 0x00]);
        writer.write_bytes(&[0x0E, 0x01]);
        writer.write_bytes(&[0x00, 0x00, 0x00]);
        writer.write_u32_le(num_spans);
        
        for seg in 0..num_spans {
            let mic = seg * 1024;
            let mac = std::cmp::min(mic + 1023, col_count - 1);
            writer.write_u32_le(mic);
            writer.write_u32_le(mac);
        }
    }
    
    fn write_cell(writer: &mut BufferWriter, col: u32, data: &CellData, sst: &mut SstTable) {
        match data {
            CellData::Text(s) if s.len() <= 3 => Self::write_cell_st(writer, col, 0, s),
            CellData::Text(s) => Self::write_cell_isst(writer, col, 0, sst.add_string(s)),
            CellData::Number(n) => Self::write_cell_real(writer, col, 0, *n),
            CellData::Bool(b) => Self::write_cell_bool(writer, col, 0, *b),
            CellData::Blank => Self::write_cell_blank(writer, col, 0),
            CellData::Date(d) => Self::write_cell_real(writer, col, 0, Self::excel_date(d)),
            CellData::Error(_) => Self::write_cell_blank(writer, col, 0),
        }
    }
    
    fn write_template_cell(writer: &mut BufferWriter, cell: &CellInfo, sst: &mut SstTable) {
        match &cell.value {
            CellValue::Text(s) => Self::write_cell_isst(writer, cell.col, cell.style_index,
                sst.find_string(s).unwrap_or_else(|| sst.add_string(s))),
            CellValue::Number(n) => Self::write_cell_real(writer, cell.col, cell.style_index, *n),
            CellValue::Bool(b) => Self::write_cell_bool(writer, cell.col, cell.style_index, *b),
            CellValue::Blank => Self::write_cell_blank(writer, cell.col, cell.style_index),
        }
    }
    
    fn write_cell_real(writer: &mut BufferWriter, col: u32, xf: u32, val: f64) {
        writer.write_varint(crate::format::RecordType::BrtCellReal.to_u32());
        writer.write_varsize(16);
        writer.write_u32_le(col);
        writer.write_u24_le(xf);
        writer.write_u8(0);
        writer.write_f64_le(val);
    }
    
    fn write_cell_isst(writer: &mut BufferWriter, col: u32, xf: u32, idx: u32) {
        writer.write_varint(crate::format::RecordType::BrtCellIsst.to_u32());
        writer.write_varsize(12);
        writer.write_u32_le(col);
        writer.write_u24_le(xf);
        writer.write_u8(0);
        writer.write_u32_le(idx);
    }
    
    fn write_cell_st(writer: &mut BufferWriter, col: u32, xf: u32, s: &str) {
        let char_count = s.encode_utf16().count();
        writer.write_varint(crate::format::RecordType::BrtCellSt.to_u32());
        writer.write_varsize((8 + 4 + char_count * 2) as u32);
        writer.write_u32_le(col);
        writer.write_u24_le(xf);
        writer.write_u8(0);
        writer.write_u32_le(char_count as u32);
        for ch in s.encode_utf16() { writer.write_u16_le(ch); }
    }
    
    fn write_cell_bool(writer: &mut BufferWriter, col: u32, xf: u32, val: bool) {
        writer.write_varint(crate::format::RecordType::BrtCellBool.to_u32());
        writer.write_varsize(9);
        writer.write_u32_le(col);
        writer.write_u24_le(xf);
        writer.write_u8(val as u8);
    }
    
    fn write_cell_blank(writer: &mut BufferWriter, col: u32, xf: u32) {
        writer.write_varint(crate::format::RecordType::BrtCellBlank.to_u32());
        writer.write_varsize(8);
        writer.write_u32_le(col);
        writer.write_u24_le(xf);
        writer.write_u8(0);
    }
    
    fn write_merges(writer: &mut BufferWriter, merges: &[MergeCell]) {
        writer.write_varint(crate::format::RecordType::BrtBeginMergeCells.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(merges.len() as u32);
        
        for mc in merges {
            writer.write_varint(crate::format::RecordType::BrtMergeCell.to_u32());
            writer.write_varsize(16);
            writer.write_u32_le(mc.row_first);
            writer.write_u32_le(mc.row_last);
            writer.write_u32_le(mc.col_first);
            writer.write_u32_le(mc.col_last);
        }
        
        writer.write_varint(crate::format::RecordType::BrtEndMergeCells.to_u32());
        writer.write_varsize(0);
    }
    
    fn excel_date(dt: &chrono::DateTime<chrono::Utc>) -> f64 {
        use chrono::TimeZone;
        let epoch = chrono::Utc.with_ymd_and_hms(1899, 12, 30, 0, 0, 0).unwrap();
        let dur = dt.signed_duration_since(epoch);
        dur.num_days() as f64 + (dur.num_seconds() % 86400) as f64 / 86400.0
    }
}

pub struct TemplateFillerBuilder {
    template: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl TemplateFillerBuilder {
    pub fn template(mut self, t: impl Into<PathBuf>) -> Self {
        self.template = Some(t.into());
        self
    }
    
    pub fn output(mut self, o: impl Into<PathBuf>) -> Self {
        self.output = Some(o.into());
        self
    }
    
    pub fn build(self) -> Result<TemplateFiller> {
        let template = self.template.ok_or(XlsbError::PathNotSet)?;
        let output = self.output.ok_or(XlsbError::PathNotSet)?;
        
        let (sheets, sst) = TemplateFiller::load_template(&template)?;
        
        Ok(TemplateFiller { template_path: template, output_path: output, sheets, sst, streaming: None })
    }
}