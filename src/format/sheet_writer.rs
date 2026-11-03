use crate::io::BufferWriter;
use crate::format::{RecordType, SstTable, StylesRegistry};
use crate::api::{CellData, CellSupplier};
use crate::error::{Result, XlsbError};
use bytes::Bytes;
use chrono::{TimeZone, Utc, DateTime};
use once_cell::sync::Lazy;

const MIN_RK_INTEGER: i32 = -536870912;
const MAX_RK_INTEGER: i32 = 536870911;

#[inline]
fn put_u24(dst: &mut [u8], v: u32) {
    dst[0] = v as u8;
    dst[1] = (v >> 8) as u8;
    dst[2] = (v >> 16) as u8;
}

static EXCEL_EPOCH: Lazy<DateTime<Utc>> = Lazy::new(|| {
    Utc.with_ymd_and_hms(1899, 12, 30, 0, 0, 0)
        .single()
        .expect("Excel epoch (1899-12-30 00:00:00 UTC) must exist")
});

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
                if !matches!(cell_data, CellData::Blank) {
                    self.write_cell(row as u32, col as u32, cell_data)?;
                }
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
        self.buffer.ensure_capacity(row_count * col_count * 22 + 2048);
        self.write_prologue(row_count, col_count)?;

        for row in 0..row_count {
            self.write_row_header(row, col_count)?;
            for col in 0..col_count {
                let cell_data = supplier.get_cell(row, col);
                if !matches!(cell_data, CellData::Blank) {
                    self.write_cell(row as u32, col as u32, cell_data)?;
                }
            }
        }

        self.write_epilogue();
        self.max_row = row_count;
        self.max_col = col_count;
        Ok(())
    }

    /// 从「按所有权持有的行数据」直接写入，避免每个单元格再 clone 一次。
    ///
    /// 供流式写入在 `end_sheet` 时使用：`rows` 会被消费，文本串直接移入 SST。
    pub fn write_batch_owned(&mut self, rows: Vec<Vec<CellData>>, col_count: usize) -> Result<()> {
        let row_count = rows.len();
        self.buffer.ensure_capacity(row_count * col_count * 22 + 2048);
        self.write_prologue(row_count, col_count)?;

        for (row, row_cells) in rows.into_iter().enumerate() {
            self.write_row_header(row, col_count)?;
            for (col, cell_data) in row_cells.into_iter().enumerate() {
                if col >= col_count {
                    break;
                }
                if !matches!(cell_data, CellData::Blank) {
                    self.write_cell(row as u32, col as u32, cell_data)?;
                }
            }
        }

        self.write_epilogue();
        self.max_row = row_count;
        self.max_col = col_count;
        Ok(())
    }

    /// 写入 sheet 前导记录（BeginSheet、属性、维度、视图、BeginSheetData）。
    /// 写入 sheet 前导记录，返回「维度 last_row 字段」的字节偏移，便于事后回填。
    pub(crate) fn write_prologue(&mut self, row_count: usize, col_count: usize) -> Result<usize> {
        self.write_empty_record(RecordType::BrtBeginSheet)?;
        self.write_ws_prop()?;
        let dim_offset = self.buffer.len();
        self.write_dimension(0, 0, row_count.saturating_sub(1), col_count.saturating_sub(1))?;
        // BrtWsDim: varint(type=148, 2 字节) + varsize(16, 1 字节) + first_row(u32, 4 字节)
        let last_row_offset = dim_offset + 2 + 1 + 4;
        self.write_view_records()?;
        self.write_empty_record(RecordType::BrtBeginSheetData)?;
        Ok(last_row_offset)
    }

    /// 写入 sheet 收尾记录（EndSheetData、EndSheet）。
    pub(crate) fn write_epilogue(&mut self) {
        let _ = self.write_empty_record(RecordType::BrtEndSheetData);
        let _ = self.write_empty_record(RecordType::BrtEndSheet);
    }

    /// 以外部缓冲区构造（用于流式增量写入）。
    pub(crate) fn with_buffer(
        buffer: BufferWriter,
        sst: &'a mut SstTable,
        styles: &'a mut StylesRegistry,
    ) -> Self {
        let default_date_style_id = styles.get_style_id_for_format("m/d/yy h:mm");
        Self {
            buffer,
            sst,
            styles,
            max_row: 0,
            max_col: 0,
            streaming_mode: false,
            streaming_col_count: 0,
            default_date_style_id,
        }
    }

    /// 取出内部缓冲区。
    pub(crate) fn into_buffer(self) -> BufferWriter {
        self.buffer
    }

    /// 追加原始字节（用于把预写的行记录拼接到前导/收尾之间）。
    pub(crate) fn append_raw(&mut self, bytes: &[u8]) {
        self.buffer.write_bytes(bytes);
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
    
    pub(crate) fn write_row_header(&mut self, row: usize, col_count: usize) -> Result<()> {
        let num_spans = if col_count > 0 {
            (col_count - 1) / 1024 + 1
        } else {
            0
        };

        // 常见路径：单 span（列数 <= 1024），一次性写入固定 27 字节。
        if num_spans == 1 {
            let mut b = [0u8; 27];
            b[0] = RecordType::BrtRowHdr as u8;
            b[1] = 25; // record size
            b[2..6].copy_from_slice(&(row as u32).to_le_bytes());
            // b[6..10] ixfe = 0
            b[10..12].copy_from_slice(&270u16.to_le_bytes());
            // b[12..15] flags = 0
            b[15..19].copy_from_slice(&1u32.to_le_bytes()); // ccolspan = 1
            // span: colMic = 0, colLast = col_count-1
            b[23..27].copy_from_slice(&((col_count as u32).saturating_sub(1)).to_le_bytes());
            self.buffer.write_bytes(&b);
            return Ok(());
        }

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
    
    pub(crate) fn write_cell(&mut self, row: u32, col: u32, data: CellData) -> Result<()> {
        match data {
            CellData::Text(s) => {
                let sst_idx = self.sst.add_string_owned(s);
                self.write_cell_isst(row, col, sst_idx)?;
            }
            CellData::Number(n) => {
                if self.can_use_rk(n) {
                    self.write_cell_rk(row, col, n as i32)?;
                } else {
                    self.write_cell_real(row, col, n)?;
                }
            }
            CellData::NumberWithFormat(n, format) => {
                let style_idx = self.styles.get_style_id_for_format(&format);
                if self.can_use_rk(n) {
                    self.write_cell_rk_with_style(row, col, n as i32, style_idx)?;
                } else {
                    self.write_cell_real_with_style(row, col, n, style_idx)?;
                }
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
                let excel_serial = self.timestamp_to_excel_serial(timestamp)?;
                self.write_cell_real_with_style(row, col, excel_serial, style_idx)?;
            }
        }
        Ok(())
    }
    
    fn write_cell_real(&mut self, _row: u32, col: u32, value: f64) -> Result<()> {
        self.write_cell_real_inner(col, value, 0)
    }

    fn write_cell_real_with_style(&mut self, _row: u32, col: u32, value: f64, style_idx: u32) -> Result<()> {
        self.write_cell_real_inner(col, value, style_idx)
    }

    #[inline]
    fn write_cell_real_inner(&mut self, col: u32, value: f64, style_idx: u32) -> Result<()> {
        let mut b = [0u8; 18];
        b[0] = RecordType::BrtCellReal as u8;
        b[1] = 16;
        b[2..6].copy_from_slice(&col.to_le_bytes());
        put_u24(&mut b[6..9], style_idx);
        b[10..18].copy_from_slice(&value.to_le_bytes());
        self.buffer.write_bytes(&b);
        Ok(())
    }
    
    fn can_use_rk(&self, value: f64) -> bool {
        if value.fract() != 0.0 {
            return false;
        }
        let int_val = value as i32;
        int_val >= MIN_RK_INTEGER && int_val <= MAX_RK_INTEGER
    }
    
    fn write_cell_rk(&mut self, _row: u32, col: u32, value: i32) -> Result<()> {
        self.write_cell_rk_inner(col, value, 0)
    }

    fn write_cell_rk_with_style(&mut self, _row: u32, col: u32, value: i32, style_idx: u32) -> Result<()> {
        self.write_cell_rk_inner(col, value, style_idx)
    }

    #[inline]
    fn write_cell_rk_inner(&mut self, col: u32, value: i32, style_idx: u32) -> Result<()> {
        let mut b = [0u8; 14];
        b[0] = RecordType::BrtCellRk as u8;
        b[1] = 12;
        b[2..6].copy_from_slice(&col.to_le_bytes());
        put_u24(&mut b[6..9], style_idx);
        let rk = ((value as f64).to_bits() >> 32) as u32;
        b[10..14].copy_from_slice(&rk.to_le_bytes());
        self.buffer.write_bytes(&b);
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
        let mut b = [0u8; 14];
        b[0] = RecordType::BrtCellIsst as u8;
        b[1] = 12;
        b[2..6].copy_from_slice(&col.to_le_bytes());
        b[10..14].copy_from_slice(&sst_idx.to_le_bytes());
        self.buffer.write_bytes(&b);
        Ok(())
    }
    
    fn write_cell_bool(&mut self, _row: u32, col: u32, value: bool) -> Result<()> {
        let mut b = [0u8; 11];
        b[0] = RecordType::BrtCellBool as u8;
        b[1] = 9;
        b[2..6].copy_from_slice(&col.to_le_bytes());
        b[10] = value as u8;
        self.buffer.write_bytes(&b);
        Ok(())
    }
    
    fn write_cell_blank(&mut self, _row: u32, col: u32) -> Result<()> {
        let mut b = [0u8; 10];
        b[0] = RecordType::BrtCellBlank as u8;
        b[1] = 8;
        b[2..6].copy_from_slice(&col.to_le_bytes());
        self.buffer.write_bytes(&b);
        Ok(())
    }
    
    fn excel_date_serial(&self, dt: &DateTime<Utc>) -> f64 {
        let duration = dt.signed_duration_since(*EXCEL_EPOCH);
        let days = duration.num_days() as f64;
        let seconds = duration.num_seconds() % 86400;
        let fractional_day = seconds as f64 / 86400.0;
        days + fractional_day
    }
    
    fn timestamp_to_excel_serial(&self, timestamp: i64) -> Result<f64> {
        let dt = Utc.timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| XlsbError::InvalidFormat(format!("Invalid timestamp: {}", timestamp)))?;
        Ok(self.excel_date_serial(&dt))
    }
    
    pub fn serialize(&self) -> Bytes {
        self.buffer.clone().freeze()
    }

    /// 消费自身，取出序列化后的 sheet 字节（避免一次全量拷贝）。
    pub fn into_serialized(self) -> Bytes {
        self.buffer.into_bytes()
    }
}