//! XLSB文件读取器API

use std::path::PathBuf;
use std::collections::HashMap;
use bytes::Bytes;
use crate::container::XlsbContainerReader;
use crate::format::{SstTable, StylesRegistry, WorkbookReader, SheetReader};
use crate::data::SheetInfo;
use crate::api::{CellData, RowHandler};
use crate::error::{XlsbError, Result};

/// XLSB文件读取器
///
/// 提供流式和分页两种读取方式：
/// - `for_each_row`: 流式读取，逐行回调处理，适合大数据量
/// - `read_rows`: 分页读取，批量返回数据，适合分页展示
///
/// ## 示例
///
/// ```rust,ignore
/// use rxlsb::{XlsbReader, CellData};
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("data.xlsb");
/// let mut reader = XlsbReader::builder().path(&path).build().unwrap();
///
/// // 流式读取
/// reader.for_each_row(0, |row_idx, cells| {
///     println!("Row {}: {} cells", row_idx, cells.len());
/// }).unwrap();
///
/// // 分页读取
/// let rows = reader.read_rows(0, 0, 1000).unwrap();
/// println!("Read {} rows", rows.len());
/// ```
pub struct XlsbReader {
    container: XlsbContainerReader,
    sst: Option<SstTable>,
    #[allow(dead_code)]
    styles: StylesRegistry,
    workbook: WorkbookReader,
    sheet_cache: HashMap<usize, Bytes>,
}

impl XlsbReader {
    /// 创建Reader构建器
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
    
    /// 获取所有sheet信息
    pub fn get_sheet_infos(&self) -> &[SheetInfo] {
        self.workbook.get_sheet_infos()
    }
    
    /// 流式读取sheet数据
    ///
    /// 逐行回调处理，适合大数据量场景（百万行级别）
    ///
    /// ## 参数
    /// - `sheet_idx`: sheet索引（从0开始）
    /// - `handler`: 行处理回调函数
    ///
    /// ## 性能
    /// 读取速度可达2.3M行/秒
    pub fn for_each_row(&mut self, sheet_idx: usize, handler: impl RowHandler) -> Result<()> {
        if sheet_idx >= self.workbook.sheet_count() {
            return Err(XlsbError::InvalidSheetIndex(sheet_idx));
        }
        
        let sheet_data = self.container.get_sheet_data(sheet_idx)?;
        let mut sheet_reader = SheetReader::new(sheet_data, self.sst.as_ref());
        sheet_reader.for_each_row(handler)?;
        Ok(())
    }
    
    /// 分页读取sheet数据
    ///
    /// 批量返回指定范围的数据，适合分页展示场景
    ///
    /// ## 参数
    /// - `sheet_idx`: sheet索引（从0开始）
    /// - `start_row`: 起始行（从0开始）
    /// - `row_count`: 读取行数
    ///
    /// ## 性能
    /// 读取速度约31K行/秒（使用sheet缓存优化）
    ///
    /// ## 返回
    /// 返回二维数组，每个元素是一行的单元格数据
    pub fn read_rows(&mut self, sheet_idx: usize, start_row: usize, row_count: usize) -> Result<Vec<Vec<CellData>>> {
        if sheet_idx >= self.workbook.sheet_count() {
            return Err(XlsbError::InvalidSheetIndex(sheet_idx));
        }
        
        let sheet_data = self.get_cached_sheet(sheet_idx)?;
        let mut sheet_reader = SheetReader::new(sheet_data, self.sst.as_ref());
        sheet_reader.read_rows(start_row, row_count)
    }
}

/// XlsbReader构建器
pub struct XlsbReaderBuilder {
    path: Option<PathBuf>,
}

impl XlsbReaderBuilder {
    /// 设置文件路径
    pub fn path(mut self, p: impl Into<PathBuf>) -> Self {
        self.path = Some(p.into());
        self
    }
    
    /// 构建Reader实例
    pub fn build(self) -> Result<XlsbReader> {
        let path = self.path.ok_or(XlsbError::PathNotSet)?;
        XlsbReader::new(&path)
    }
}