use thiserror::Error;

#[derive(Error, Debug)]
pub enum XlsbError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("ZIP错误: {0}")]
    Zip(#[from] zip::result::ZipError),
    
    #[error("缓冲区溢出: 位置 {position}, 长度 {length}")]
    BufferOverflow { position: usize, length: usize },
    
    #[error("无效的VarInt编码")]
    InvalidVarInt,
    
    #[error("无效的UTF-16编码")]
    InvalidUtf16,
    
    #[error("无效的BIFF12记录类型: {0}")]
    InvalidRecordType(u32),
    
    #[error("无效的单元格类型: {0}")]
    InvalidCellType(u32),
    
    #[error("无效的Sheet索引: {0}")]
    InvalidSheetIndex(usize),
    
    #[error("Sheet未开始写入")]
    SheetNotStarted,
    
    #[error("未找到标记: {0}")]
    MarkerNotFound(String),
    
    #[error("模板读取失败: {0}")]
    TemplateReadFailed(String),
    
    #[error("路径未设置")]
    PathNotSet,
    
    #[error("无效格式: {0}")]
    InvalidFormat(String),
    
    #[error("无效参数: {0}")]
    InvalidArgument(&'static str),
    
    #[error("无效状态: {0}")]
    InvalidState(String),
}

pub type Result<T> = std::result::Result<T, XlsbError>;