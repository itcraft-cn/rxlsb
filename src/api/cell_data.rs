//! Excel单元格数据类型定义

use chrono::{DateTime, Utc};

/// Excel单元格数据类型
///
/// 代表一个Excel单元格可能包含的各种数据类型：
/// - 文本字符串
/// - 数值（浮点数）
/// - 日期时间
/// - 布尔值
/// - 空单元格
/// - 错误值（如#DIV/0!、#VALUE!等）
#[derive(Clone, Debug)]
pub enum CellData {
    /// 文本字符串
    Text(String),
    /// 数值（IEEE 754双精度浮点数）
    Number(f64),
    /// 数值带格式（数值 + 格式代码）
    NumberWithFormat(f64, String),
    /// 日期时间（UTC时区）
    Date(DateTime<Utc>),
    /// 日期带格式（时间戳 + 格式代码）
    DateWithFormat(i64, String),
    /// 布尔值
    Bool(bool),
    /// 空单元格
    Blank,
    /// 错误值
    Error(CellError),
}

/// Excel错误值类型
///
/// 对应Excel中的各种错误值：
/// - `#DIV/0!` - 除零错误
/// - `#VALUE!` - 值错误
/// - `#REF!` - 引用错误
/// - `#NAME?` - 名称错误
/// - `#NUM!` - 数字错误
/// - `#N/A` - 不可用
#[derive(Clone, Debug)]
pub enum CellError {
    Div0, Value, Ref, Name, Num, Na,
}

impl CellData {
    /// 创建文本单元格
    pub fn text(s: impl Into<String>) -> Self { CellData::Text(s.into()) }
    
    /// 创建数值单元格
    pub fn number(n: f64) -> Self { CellData::Number(n) }
    
    /// 创建带格式的数值单元格
    pub fn number_with_format(n: f64, format_code: impl Into<String>) -> Self {
        CellData::NumberWithFormat(n, format_code.into())
    }
    
    /// 创建日期单元格
    pub fn date(d: DateTime<Utc>) -> Self { CellData::Date(d) }
    
    /// 创建日期单元格（时间戳）
    pub fn date_from_timestamp(timestamp: i64) -> Self {
        CellData::DateWithFormat(timestamp, "mm-dd-yy".to_string())
    }
    
    /// 创建日期单元格带格式
    pub fn date_with_format(timestamp: i64, format_code: impl Into<String>) -> Self {
        CellData::DateWithFormat(timestamp, format_code.into())
    }
    
    /// 创建布尔单元格
    pub fn bool(b: bool) -> Self { CellData::Bool(b) }
    
    /// 创建空单元格
    pub fn blank() -> Self { CellData::Blank }
    
    /// 创建百分比单元格（默认2位小数）
    pub fn percentage(value: f64) -> Self {
        CellData::NumberWithFormat(value, "0.00%".to_string())
    }
    
    /// 创建百分比单元格（指定小数位数）
    pub fn percentage_with_decimals(value: f64, decimals: usize) -> Self {
        let format = if decimals == 0 {
            "0%".to_string()
        } else {
            let zeros = "0".repeat(decimals);
            format!("0.{}%", zeros)
        };
        CellData::NumberWithFormat(value, format)
    }
    
    /// 创建时间单元格
    pub fn time(timestamp: i64) -> Self {
        CellData::DateWithFormat(timestamp, "h:mm:ss".to_string())
    }
    
    /// 创建时间单元格带格式
    pub fn time_with_format(timestamp: i64, format_code: impl Into<String>) -> Self {
        CellData::DateWithFormat(timestamp, format_code.into())
    }
    
    /// 创建负数红色显示的数值单元格
    pub fn number_negative_red(value: f64) -> Self {
        CellData::NumberWithFormat(value, "#,##0.00;[Red]-#,##0.00".to_string())
    }
    
    /// 创建千分位分隔的数值单元格（默认2位小数）
    pub fn number_with_comma(value: f64) -> Self {
        CellData::NumberWithFormat(value, "#,##0.00".to_string())
    }
    
    /// 创建千分位分隔的数值单元格（指定小数位数）
    pub fn number_with_comma_and_decimals(value: f64, decimals: usize) -> Self {
        let format = if decimals == 0 {
            "#,##0".to_string()
        } else {
            let zeros = "0".repeat(decimals);
            format!("#,##0.{}", zeros)
        };
        CellData::NumberWithFormat(value, format)
    }
    
    /// 创建货币单元格（默认人民币符号）
    pub fn currency(value: f64) -> Self {
        CellData::NumberWithFormat(value, "￥#,##0.00".to_string())
    }
    
    /// 创建货币单元格（指定货币符号）
    pub fn currency_with_symbol(value: f64, symbol: impl Into<String>) -> Self {
        let format = format!("{}#,##0.00", symbol.into());
        CellData::NumberWithFormat(value, format)
    }
    
    /// 获取格式代码（如果有）
    pub fn get_format_code(&self) -> Option<&str> {
        match self {
            CellData::NumberWithFormat(_, format) => Some(format),
            CellData::DateWithFormat(_, format) => Some(format),
            _ => None,
        }
    }
    
    /// 是否有格式代码
    pub fn has_format(&self) -> bool {
        matches!(self, CellData::NumberWithFormat(_, _) | CellData::DateWithFormat(_, _))
    }
}

/// 单元格数据供应器trait
///
/// 用于批量写入时提供单元格数据。通常通过闭包实现：
///
/// ```rust,ignore
/// use rxlsb::{XlsbWriter, CellData};
///
/// let supplier = |row: usize, col: usize| {
///     CellData::number(row as f64 * 10.0 + col as f64)
/// };
/// ```
pub trait CellSupplier {
    /// 获取指定行列的单元格数据
    fn get_cell(&self, row: usize, col: usize) -> CellData;
}

impl<F: Fn(usize, usize) -> CellData> CellSupplier for F {
    fn get_cell(&self, row: usize, col: usize) -> CellData { self(row, col) }
}

/// 行处理handler trait
///
/// 用于流式读取时处理每一行数据。通常通过闭包实现：
///
/// ```rust,ignore
/// use rxlsb::{XlsbReader, CellData};
///
/// reader.for_each_row(0, |row_idx, cells| {
///     println!("Row {}: {} cells", row_idx, cells.len());
/// }).unwrap();
/// ```
pub trait RowHandler {
    /// 处理一行数据
    fn on_row(&mut self, row_idx: usize, cells: &[CellData]);
}

impl<F: FnMut(usize, &[CellData])> RowHandler for F {
    fn on_row(&mut self, row_idx: usize, cells: &[CellData]) { self(row_idx, cells) }
}