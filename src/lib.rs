//! # rxlsb - 高性能XLSB文件读写库
//!
//! rxlsb是一个纯Rust实现的Excel XLSB格式读写库，提供高性能的二进制Excel文件处理能力。
//!
//! ## 核心特性
//!
//! - **高性能**: 写入200K行/秒，读取2.3M行/秒，全面超过Java jxlsb
//! - **零拷贝**: 基于Bytes架构，最小化内存复制
//! - **流式API**: 支持批量写入、分页读取、流式处理
//! - **模板填充**: 支持基于模板的数据填充
//! - **类型安全**: Rust类型系统保证，无运行时错误
//!
//! ## 快速开始
//!
//! ### 写入XLSB文件
//!
//! ```rust
//! use rxlsb::{XlsbWriter, CellData};
//! use std::path::PathBuf;
//!
//! let path = PathBuf::from("output.xlsb");
//! let mut writer = XlsbWriter::builder().path(&path).build().unwrap();
//!
//! writer.write_batch("Sheet1", |row, col| {
//!     match col {
//!         0 => CellData::text(format!("Name-{}", row)),
//!         1 => CellData::number(row as f64),
//!         _ => CellData::blank(),
//!     }
//! }, 1000, 5).unwrap();
//!
//! writer.close().unwrap();
//! ```
//!
//! ### 读取XLSB文件
//!
//! ```rust,ignore
//! use rxlsb::{XlsbReader, CellData};
//! use std::path::PathBuf;
//!
//! let path = PathBuf::from("output.xlsb");
//! let mut reader = XlsbReader::builder().path(&path).build().unwrap();
//!
//! // 流式读取
//! reader.for_each_row(0, |row_idx, cells| {
//!     println!("Row {}: {} cells", row_idx, cells.len());
//! }).unwrap();
//!
//! // 分页读取
//! let rows = reader.read_rows(0, 0, 100).unwrap();
//! println!("Read {} rows", rows.len());
//! ```
//!
//! ## 性能对比
//!
//! | 操作 | rxlsb (Rust) | jxlsb (Java) | 提升 |
//! |------|-------------|-------------|------|
//! | 流式写 | 201K/s | 137K/s | **+46%** |
//! | 分页写 | 190K/s | 111K/s | **+71%** |
//! | 流式读 | 2.3M/s | 1.95M/s | **+18%** |
//! | 分页读 | 31K/s | 10K/s | **+210%** |

mod error;
mod io;
mod container;
mod format;
mod data;
mod api;

pub use error::{XlsbError, Result};
pub use api::{CellData, CellError, XlsbReader, XlsbWriter, TemplateFiller};
pub use data::SheetInfo;