use crate::io::BufferWriter;
use crate::format::{RecordType, SstTable, StylesRegistry};
use crate::api::{CellData, CellSupplier};
use crate::error::Result;
use bytes::Bytes;

pub struct SheetWriter<'a> {
    buffer: BufferWriter,
    sst: &'a mut SstTable,
    styles: &'a mut StylesRegistry,
    max_row: usize,
    max_col: usize,
    streaming_mode: bool,
    streaming_col_count: usize,
    default_date_style_id: u32,
}

impl<'a> SheetWriter<'a> {
    pub fn new(sst: &'a mut SstTable, styles: &'a mut StylesRegistry) -> Self {
        let default_date_style_id = styles.get_style_id_for_format("m/d/yy h:mm");
        Self {
            buffer: BufferWriter::new(1024),
            sst,
            styles,
            max_row: 0,
            max_col: 0,
            streaming_mode: false,
            streaming_col_count: 0,
            default_date_style_id,
        }
    }
    
    #[allow(dead_code)]
    pub fn start_streaming(&mut self, col_count: usize) -> Result<()> {
        self.streaming_mode = true;
        self.streaming_col_count = col_count;
        self.max_col = col_count.saturating_sub(1);
        self.buffer.clear();
        
        self.write_empty_record(RecordType::BrtBeginSheet)?;
        self.write_ws_prop()?;
        self.write_view_records()?;
        self.write_empty_record(RecordType::BrtBeginSheetData)?;
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn append_rows(&mut self, supplier: impl CellSupplier, start_row: usize, row_count: usize) -> Result<()> {
        let col_count = self.streaming_col_count;
        
        for row in start_row..start_row + row_count {
            self.write_row_header(row, col_count)?;
            for col in 0..col_count {
                let cell_data = supplier.get_cell(row, col);
                self.write_cell(row as u32, col as u32, cell_data)?;
            }
            if row > self.max_row {
                self.max_row = row;
            }
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn finalize_streaming(&mut self, row_count: usize, col_count: usize) -> Result<Bytes> {
        self.write_empty_record(RecordType::BrtEndSheetData)?;
        self.write_page_setup_records()?;
        self.write_empty_record(RecordType::BrtEndSheet)?;
        
        let mut final_buffer = BufferWriter::new(self.buffer.len() + 50);
        
        final_buffer.write_varint(RecordType::BrtBeginSheet.to_u32());
        final_buffer.write_varsize(0);
        final_buffer.write_bytes(&[
            0xC9, 0x04, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00
        ]);
        
        self.write_dimension_to(&mut final_buffer, 0, row_count.saturating_sub(1), 0, col_count.saturating_sub(1))?;
        
        final_buffer.write_bytes(&self.buffer.clone().freeze());
        
        self.streaming_mode = false;
        Ok(final_buffer.freeze())
    }
    
    #[allow(dead_code)]
    fn write_dimension_to(&self, writer: &mut BufferWriter, first_row: usize, first_col: usize,
                          last_row: usize, last_col: usize) -> Result<()> {
        writer.write_varint(RecordType::BrtWsDim.to_u32());
        writer.write_varsize(16);
        writer.write_u32_le(first_row as u32);
        writer.write_u32_le(last_row as u32);
        writer.write_u32_le(first_col as u32);
        writer.write_u32_le(last_col as u32);
        Ok(())
    }
    
    pub fn write_batch(&mut self, supplier: impl CellSupplier,
                       row_count: usize, col_count: usize) -> Result<()> {
        self.buffer.ensure_capacity(row_count * col_count * 30 + 2048);
        
        self.write_empty_record(RecordType::BrtBeginSheet)?;
        self.write_ws_prop()?;
        self.write_dimension(0, 0, row_count.saturating_sub(1), col_count.saturating_sub(1))?;
        self.write_view_records()?;
        self.write_empty_record(RecordType::BrtBeginSheetData)?;
        
        for row in 0..row_count {
            self.write_row_header(row, col_count)?;
            for col in 0..col_count {
                let cell_data = supplier.get_cell(row, col);
                self.write_cell(row as u32, col as u32, cell_data)?;
            }
        }
        
        self.write_empty_record(RecordType::BrtEndSheetData)?;
        self.write_empty_record(RecordType::BrtEndSheet)?;
        
        self.max_row = row_count;
        self.max_col = col_count;
        Ok(())
    }
    
    fn write_ws_prop(&mut self) -> Result<()> {
        self.buffer.write_varint(RecordType::BrtWsProp.to_u32());
        self.buffer.write_varsize(23);
        self.buffer.write_bytes(&[
            0xC9, 0x04, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00
        ]);
        Ok(())
    }
    
    fn write_view_records(&mut self) -> Result<()> {
        self.write_empty_record(RecordType::BrtBeginWsViews)?;
        
        self.buffer.write_varint(RecordType::BrtBeginWsView.to_u32());
        self.buffer.write_varsize(30);
        self.buffer.write_bytes(&[
            0xDC, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x40, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]);
        
        self.buffer.write_varint(RecordType::BrtSel.to_u32());
        self.buffer.write_varsize(36);
        self.buffer.write_bytes(&[
            0x03, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
            0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00,
            0x07, 0x00, 0x00, 0x00
        ]);
        
        self.write_empty_record(RecordType::BrtEndWsView)?;
        self.write_empty_record(RecordType::BrtEndWsViews)?;
        
        self.buffer.write_varint(RecordType::BrtWsFmtInfo.to_u32());
        self.buffer.write_varsize(12);
        self.buffer.write_bytes(&[
            0x00, 0x09, 0x00, 0x00, 0x08, 0x00, 0x0E, 0x01,
            0x00, 0x00, 0x00, 0x00
        ]);
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn write_page_setup_records(&mut self) -> Result<()> {
        self.buffer.write_varint(RecordType::BrtDrawing.to_u32());
        self.buffer.write_varsize(62);
        self.buffer.write_bytes(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00
        ]);
        
        self.buffer.write_varint(RecordType::BrtPageSetupView.to_u32());
        self.buffer.write_varsize(2);
        self.buffer.write_bytes(&[0x10, 0x00]);
        
        self.buffer.write_varint(RecordType::BrtPageSetup.to_u32());
        self.buffer.write_varsize(47);
        self.buffer.write_bytes(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0xE8, 0x3F, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0xE8, 0x3F, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x3F, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x3F
        ]);
        
        Ok(())
    }
    
    fn write_empty_record(&mut self, record_type: RecordType) -> Result<()> {
        self.buffer.write_varint(record_type.to_u32());
        self.buffer.write_varsize(0);
        Ok(())
    }
    
    fn write_dimension(&mut self, first_row: usize, first_col: usize,
                       last_row: usize, last_col: usize) -> Result<()> {
        self.buffer.write_varint(RecordType::BrtWsDim.to_u32());
        self.buffer.write_varsize(16);
        self.buffer.write_u32_le(first_row as u32);
        self.buffer.write_u32_le(last_row as u32);
        self.buffer.write_u32_le(first_col as u32);
        self.buffer.write_u32_le(last_col as u32);
        Ok(())
    }
    
    fn write_row_header(&mut self, row: usize, col_count: usize) -> Result<()> {
        let num_spans = if col_count > 0 {
            (col_count - 1) / 1024 + 1
        } else {
            0
        };
        
        let record_size = 4 + 4 + 2 + 3 + 4 + (num_spans * 8);
        
        self.buffer.write_varint(RecordType::BrtRowHdr.to_u32());
        self.buffer.write_varsize(record_size as u32);
        self.buffer.write_u32_le(row as u32);
        self.buffer.write_u32_le(0);
        self.buffer.write_u16_le(270);
        self.buffer.write_bytes(&[0x00, 0x00, 0x00]);
        self.buffer.write_u32_le(num_spans as u32);
        
        for seg in 0..num_spans {
            let seg_start_col = seg * 1024;
            let seg_end_col = std::cmp::min((seg + 1) * 1024 - 1, col_count - 1);
            self.buffer.write_u32_le(seg_start_col as u32);
            self.buffer.write_u32_le(seg_end_col as u32);
        }
        
        Ok(())
    }
    
    fn write_cell(&mut self, row: u32, col: u32, data: CellData) -> Result<()> {
        match data {
            CellData::Text(s) => {
                let sst_idx = self.sst.add_string(&s);
                self.write_cell_isst(row, col, sst_idx)?;
            }
            CellData::Number(n) => {
                self.write_cell_real(row, col, n)?;
            }
            CellData::NumberWithFormat(n, format) => {
                let style_idx = self.styles.get_style_id_for_format(&format);
                self.write_cell_real_with_style(row, col, n, style_idx)?;
            }
            CellData::Bool(b) => {
                self.write_cell_bool(row, col, b)?;
            }
            CellData::Blank => {
                self.write_cell_blank(row, col)?;
            }
            CellData::Error(_) => {
                self.write_cell_blank(row, col)?;
            }
            CellData::Date(d) => {
                let excel_serial = self.excel_date_serial(&d);
                self.write_cell_real_with_style(row, col, excel_serial, self.default_date_style_id)?;
            }
            CellData::DateWithFormat(timestamp, format) => {
                let style_idx = self.styles.get_style_id_for_format(&format);
                let excel_serial = self.timestamp_to_excel_serial(timestamp);
                self.write_cell_real_with_style(row, col, excel_serial, style_idx)?;
            }
        }
        Ok(())
    }
    
    fn write_cell_real(&mut self, _row: u32, col: u32, value: f64) -> Result<()> {
        self.buffer.write_varint(RecordType::BrtCellReal.to_u32());
        self.buffer.write_varsize(16);
        self.buffer.write_u32_le(col);
        self.buffer.write_bytes(&[0x00, 0x00, 0x00, 0x00]);
        self.buffer.write_f64_le(value);
        Ok(())
    }
    
fn write_cell_real_with_style(&mut self, _row: u32, col: u32, value: f64, style_idx: u32) -> Result<()> {
        self.buffer.write_varint(RecordType::BrtCellReal.to_u32());
        self.buffer.write_varsize(16);
        self.buffer.write_u32_le(col);        // 4字节
        self.buffer.write_u24_le(style_idx);  // 3字节
        self.buffer.write_u8(0x00);           // 1字节
        self.buffer.write_f64_le(value);      // 8字节
        Ok(())
    }
    
    #[allow(dead_code)]
    fn write_cell_st(&mut self, _row: u32, col: u32, s: &str) -> Result<()> {
        let string_size = 4 + BufferWriter::utf16le_byte_length(s);
        self.buffer.write_varint(RecordType::BrtCellSt.to_u32());
        self.buffer.write_varsize((8 + string_size) as u32);
        self.buffer.write_u32_le(col);
        self.buffer.write_bytes(&[0x00, 0x00, 0x00, 0x00]);
        self.buffer.write_wide_string_u32(s);
        Ok(())
    }
    
    fn write_cell_isst(&mut self, _row: u32, col: u32, sst_idx: u32) -> Result<()> {
        self.buffer.write_varint(RecordType::BrtCellIsst.to_u32());
        self.buffer.write_varsize(12);
        self.buffer.write_u32_le(col);
        self.buffer.write_bytes(&[0x00, 0x00, 0x00, 0x00]);
        self.buffer.write_u32_le(sst_idx);
        Ok(())
    }
    
    fn write_cell_bool(&mut self, _row: u32, col: u32, value: bool) -> Result<()> {
        self.buffer.write_varint(RecordType::BrtCellBool.to_u32());
        self.buffer.write_varsize(9);
        self.buffer.write_u32_le(col);
        self.buffer.write_bytes(&[0x00, 0x00, 0x00]);
        self.buffer.write_u8(value as u8);
        Ok(())
    }
    
    fn write_cell_blank(&mut self, _row: u32, col: u32) -> Result<()> {
        self.buffer.write_varint(RecordType::BrtCellBlank.to_u32());
        self.buffer.write_varsize(8);
        self.buffer.write_u32_le(col);
        self.buffer.write_bytes(&[0x00, 0x00, 0x00, 0x00]);
        Ok(())
    }
    
    fn excel_date_serial(&self, dt: &chrono::DateTime<chrono::Utc>) -> f64 {
        use chrono::TimeZone;
        let excel_epoch = chrono::Utc.with_ymd_and_hms(1899, 12, 30, 0, 0, 0).unwrap();
        let duration = dt.signed_duration_since(excel_epoch);
        let days = duration.num_days() as f64;
        let seconds = duration.num_seconds() % 86400;
        let fractional_day = seconds as f64 / 86400.0;
        days + fractional_day
    }
    
    fn timestamp_to_excel_serial(&self, timestamp: i64) -> f64 {
        use chrono::TimeZone;
        let dt = chrono::Utc.timestamp_opt(timestamp, 0).unwrap();
        self.excel_date_serial(&dt)
    }
    
    pub fn serialize(&self) -> Bytes {
        self.buffer.clone().freeze()
    }
}