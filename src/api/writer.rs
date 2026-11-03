//! XLSB文件写入器API

use std::path::PathBuf;
use crate::container::{XlsbContainerWriter};
use crate::format::{SstTable, StylesRegistry, WorkbookWriter, SheetWriter};
use crate::api::{CellData, CellSupplier};
use crate::container::{XmlGen, RelsGen};
use crate::error::{XlsbError, Result};

/// XLSB文件写入器
///
/// 提供流式和批量两种写入方式：
/// - `write_batch`: 批量写入，一次性写入整个sheet，适合中小数据量
/// - `start_sheet/write_rows/end_sheet`: 流式写入，分批写入，适合大数据量
///
/// ## 示例
///
/// ### 批量写入
///
/// ```rust
/// use rxlsb::{XlsbWriter, CellData};
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("output.xlsb");
/// let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
///
/// writer.write_batch("Sheet1", |row, col| {
///     match col {
///         0 => CellData::text(format!("Name-{}", row)),
///         1 => CellData::number(row as f64),
///         _ => CellData::blank(),
///     }
/// }, 10000, 5).unwrap();
///
/// writer.close().unwrap();
/// ```
///
/// ### 流式写入
///
/// ```rust
/// use rxlsb::{XlsbWriter, CellData};
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("output.xlsb");
/// let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
///
/// writer.start_sheet("Sheet1", 5).unwrap();
///
/// for offset in (0..100000).step_by(1000) {
///     writer.write_rows(|row, col| {
///         CellData::number((offset + row) as f64)
///     }, offset, 1000).unwrap();
/// }
///
/// writer.end_sheet().unwrap();
/// writer.close().unwrap();
/// ```
///
/// ## 性能
/// 写入速度可达201K行/秒，比Java jxlsb快46%
pub struct XlsbWriter {
    #[allow(dead_code)]
    path: PathBuf,
    container: XlsbContainerWriter,
    sst: SstTable,
    styles: StylesRegistry,
    workbook: WorkbookWriter,
    sheets_data: Vec<bytes::Bytes>,
    streaming: Option<StreamingState>,
}

struct StreamingState {
    sheet_name: String,
    col_count: usize,
    max_row: usize,
    /// 已按 BIFF12 编码好的整张 sheet（前导 + 行记录），增量追加。
    rows: crate::io::BufferWriter,
    /// 维度记录中 last_row 字段的字节偏移，结束时回填。
    dim_last_row_off: usize,
    /// start_sheet 时快照的行溢出策略。
    row_policy: &'static dyn crate::limits::OverflowPolicy,
}

impl XlsbWriter {
    /// 创建Writer构建器
    pub fn builder() -> XlsbWriterBuilder {
        XlsbWriterBuilder { path: None, compression_level: None }
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

    /// 设置 zip/deflate 压缩级别。
    ///
    /// `1` 为速度优先（默认），`6` 为体积优先，`0-9` 均可。需在写入内容之前调用。
    pub fn set_compression_level(&mut self, level: i32) -> Result<()> {
        if !(0..=9).contains(&level) {
            return Err(XlsbError::InvalidState(format!(
                "compression level must be in 0..=9, got {level}"
            )));
        }
        self.container.set_compression_level(level);
        Ok(())
    }
    
    /// 批量写入sheet数据
    ///
    /// 一次性写入整个sheet，适合中小数据量（<10万行）
    ///
    /// ## 参数
    /// - `sheet_name`: sheet名称
    /// - `supplier`: 单元格数据供应器（闭包）
    /// - `row_count`: 行数
    /// - `col_count`: 列数
    ///
    /// ## 性能
    /// 写入速度约201K行/秒
    pub fn write_batch(&mut self, sheet_name: &str,
                       supplier: impl CellSupplier,
                       row_count: usize, col_count: usize) -> Result<()> {
        // 开始时快照溢出策略，仅影响本次调用。
        let col_count = crate::limits::column_policy().columns(col_count)?;
        let row_count = crate::limits::row_policy().rows(0, row_count)?;

        // 预估文本量，预留 SST 容量，避免写入过程中反复 rehash/扩容。
        self.sst.reserve(row_count.saturating_mul(col_count) / 3);
        let mut sheet_writer = SheetWriter::new(&mut self.sst, &mut self.styles);
        sheet_writer.write_batch(supplier, row_count, col_count)?;
        
        self.workbook.add_sheet(sheet_name);
        self.sheets_data.push(sheet_writer.into_serialized());
        Ok(())
    }
    
    /// 开始流式写入sheet
    ///
    /// 启动流式写入模式，适合大数据量（百万行级别）
    ///
    /// ## 参数
    /// - `sheet_name`: sheet名称
    /// - `col_count`: 列数
    pub fn start_sheet(&mut self, sheet_name: &str, col_count: usize) -> Result<()> {
        if self.streaming.is_some() {
            return Err(XlsbError::InvalidState("Previous sheet not ended, call end_sheet() first".into()));
        }

        // 开始时快照策略：列在本次解析，行策略存起来供后续 write_rows 使用。
        let col_count = crate::limits::column_policy().columns(col_count)?;
        let row_policy = crate::limits::row_policy();

        // 先写入前导记录（维度暂以 0 行占位），记录 last_row 偏移以便结束时回填。
        let Self { sst, styles, streaming, .. } = self;
        let mut sheet_writer =
            SheetWriter::with_buffer(crate::io::BufferWriter::new(64 * 1024), sst, styles);
        let dim_last_row_off = sheet_writer.write_prologue(0, col_count)?;

        *streaming = Some(StreamingState {
            sheet_name: sheet_name.to_string(),
            col_count,
            max_row: 0,
            rows: sheet_writer.into_buffer(),
            dim_last_row_off,
            row_policy,
        });
        Ok(())
    }

    /// 流式写入一批行数据
    ///
    /// 分批写入数据，配合start_sheet和end_sheet使用
    ///
    /// ## 参数
    /// - `supplier`: 单元格数据供应器（闭包）
    /// - `start_row`: 起始行（相对于sheet开始）
    /// - `row_count`: 本批次行数
    ///
    /// ## 性能
    /// 写入速度约190K行/秒
    pub fn write_rows(&mut self, supplier: impl CellSupplier, start_row: usize, row_count: usize) -> Result<()> {
        let Self { sst, styles, streaming, .. } = self;
        let st = streaming.as_mut()
            .ok_or_else(|| XlsbError::InvalidState("Sheet not started, call start_sheet() first".into()))?;

        // 使用 start_sheet 时的行策略快照处理本批次。
        let row_count = st.row_policy.rows(start_row, row_count)?;
        if row_count == 0 {
            return Ok(());
        }

        let col_count = st.col_count;
        st.rows.ensure_capacity(row_count.saturating_mul(col_count).saturating_mul(22) + 2048);
        let mut sheet_writer = SheetWriter::with_buffer(
            std::mem::replace(&mut st.rows, crate::io::BufferWriter::new(0)),
            sst,
            styles,
        );

        for row_idx in 0..row_count {
            let abs_row = start_row + row_idx;
            sheet_writer.write_row_header(abs_row, col_count)?;
            for col in 0..col_count {
                let cell = supplier.get_cell(row_idx, col);
                if !matches!(cell, CellData::Blank) {
                    sheet_writer.write_cell(abs_row as u32, col as u32, cell)?;
                }
            }
        }

        st.rows = sheet_writer.into_buffer();
        st.max_row = st.max_row.max(start_row + row_count);
        Ok(())
    }

    /// 结束流式写入sheet
    ///
    /// 完成当前sheet的流式写入，生成sheet数据
    pub fn end_sheet(&mut self) -> Result<()> {
        let mut streaming = self.streaming.take()
            .ok_or_else(|| XlsbError::InvalidState("Sheet not started".into()))?;

        // 回填维度 last_row（无行时保持 0），随后追加收尾记录，无需整块拷贝。
        let last_row = streaming.max_row.saturating_sub(1) as u32;
        streaming.rows.set_u32_le_at(streaming.dim_last_row_off, last_row);

        let mut sheet_writer = SheetWriter::with_buffer(
            streaming.rows,
            &mut self.sst,
            &mut self.styles,
        );
        sheet_writer.write_epilogue();

        self.workbook.add_sheet(&streaming.sheet_name);
        self.sheets_data.push(sheet_writer.into_serialized());

        Ok(())
    }
    
    /// 关闭Writer，生成最终文件
    ///
    /// 完成所有sheet写入，生成zip压缩的XLSB文件
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
        
        let styles_data = self.styles.serialize();
        self.container.add_entry_from_bytes("xl/styles.bin", &styles_data)?;
        
        if has_sst {
            self.write_sst_streamed()?;
        }
        
        for (i, sheet_data) in self.sheets_data.iter().enumerate() {
            self.container.add_entry_from_bytes(&format!("xl/worksheets/sheet{}.bin", i + 1), sheet_data)?;
        }
        
        self.container.finish()?;
        Ok(())
    }

    /// 以流式压缩方式写入 sharedStrings.bin，避免构建整块中间缓冲。
    fn write_sst_streamed(&mut self) -> Result<()> {
        let Self { container, sst, .. } = self;
        container.begin_entry("xl/sharedStrings.bin")?;
        sst.serialize_chunks(&mut |chunk| container.write_entry_chunk(chunk))?;
        container.end_entry()
    }
}

/// XlsbWriter构建器
pub struct XlsbWriterBuilder {
    path: Option<PathBuf>,
    compression_level: Option<i32>,
}

impl XlsbWriterBuilder {
    /// 设置文件路径
    pub fn path(mut self, p: impl Into<PathBuf>) -> Self {
        self.path = Some(p.into());
        self
    }

    /// 设置 zip/deflate 压缩级别（0-9）。
    ///
    /// 默认 level 1（速度优先）；`6` 为体积优先。
    pub fn compression_level(mut self, level: i32) -> Self {
        self.compression_level = Some(level);
        self
    }
    
    /// 构建Writer实例
    pub fn build(self) -> Result<XlsbWriter> {
        let path = self.path.ok_or(XlsbError::PathNotSet)?;
        let mut writer = XlsbWriter::new(&path)?;
        if let Some(level) = self.compression_level {
            writer.set_compression_level(level)?;
        }
        Ok(writer)
    }
}