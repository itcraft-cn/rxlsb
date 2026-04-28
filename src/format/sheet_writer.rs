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
}

impl<'a> SheetWriter<'a> {
    pub fn new(sst: &'a mut SstTable, styles: &'a mut StylesRegistry) -> Self {
        Self {
            buffer: BufferWriter::new(1024),
            sst,
            styles,
            max_row: 0,
            max_col: 0,
        }
    }
    
    pub fn write_batch(&mut self, supplier: impl CellSupplier,
                       row_count: usize, col_count: usize) -> Result<()> {
        self.buffer.ensure_capacity(row_count * col_count * 20);
        
        self.write_empty_record(RecordType::BrtBeginSheet)?;
        self.write_dimension(0, 0, row_count - 1, col_count - 1)?;
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
        self.buffer.write_u32_le(first_col as u32);
        self.buffer.write_u32_le(last_row as u32);
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
                if s.len() <= 3 {
                    self.write_cell_st(row, col, &s)?;
                } else {
                    let sst_idx = self.sst.add_string(&s);
                    self.write_cell_isst(row, col, sst_idx)?;
                }
            }
            CellData::Number(n) => {
                self.write_cell_real(row, col, n)?;
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
                self.write_cell_real(row, col, excel_serial)?;
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
    
    fn write_cell_st(&mut self, _row: u32, col: u32, s: &str) -> Result<()> {
        let string_size = BufferWriter::utf16le_byte_length(s) + BufferWriter::varint_size(s.encode_utf16().count() as u32);
        self.buffer.write_varint(RecordType::BrtCellSt.to_u32());
        self.buffer.write_varsize((8 + string_size) as u32);
        self.buffer.write_u32_le(col);
        self.buffer.write_bytes(&[0x00, 0x00, 0x00, 0x00]);
        self.buffer.write_wide_string(s);
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
    
    pub fn serialize(&self) -> Bytes {
        self.buffer.clone().freeze()
    }
}