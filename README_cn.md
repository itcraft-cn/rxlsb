[English](README.md) | 中文文档

# rxlsb - Rust高性能XLSB文件读写库

纯Rust实现的Excel XLSB（二进制）格式读写库，提供卓越的性能。

## 核心特性

- **高性能**: 写入200K行/秒，读取2.3M行/秒，全面超过Java jxlsb
- **零拷贝**: 基于Bytes架构，最小化内存复制
- **流式API**: 支持批量写入、分页读取、流式处理
- **模板填充**: 支持基于模板的数据填充
- **类型安全**: Rust类型系统保证，无运行时错误
- **数字格式**: 百分比、货币、千分位、日期、时间、负数标红
- **自定义格式**: 灵活的自定义数字格式支持

## 性能对比

| 操作 | rxlsb (Rust) | jxlsb (Java) | 提升 |
|------|-------------|-------------|------|
| 流式写 | 201K/s | 137K/s | **+46%** |
| 分页写 | 190K/s | 111K/s | **+71%** |
| 流式读 | 2.3M/s | 1.95M/s | **+18%** |
| 分页读 | 31K/s | 10K/s | **+210%** |

## 快速开始

### 写入XLSB文件

```rust
use rxlsb::{XlsbWriter, CellData};
use std::path::PathBuf;

let path = PathBuf::from("output.xlsb");
let mut writer = XlsbWriter::builder().path(&path).build().unwrap();

writer.write_batch("Sheet1", |row, col| {
    match col {
        0 => CellData::text(format!("Name-{}", row)),
        1 => CellData::number(row as f64),
        _ => CellData::blank(),
    }
}, 10000, 5).unwrap();

writer.close().unwrap();
```

### 数字格式

```rust
use rxlsb::CellData;

// 百分比：12.30%
let cell = CellData::percentage(0.123);

// 千分位：-1,234.56
let cell = CellData::number_with_comma(-1234.56);

// 负数标红：-500.00（显示为红色）
let cell = CellData::number_negative_red(-500.0);

// 货币：￥1,234.56
let cell = CellData::currency(1234.56);

// 日期：5/1/2026 10:40
let cell = CellData::date_from_timestamp(1714560000);

// 时间：10:40:00
let cell = CellData::time(1714560000);

// 自定义格式
let cell = CellData::number_with_format(123.45, "#,##0.00");
```

### 读取XLSB文件

```rust
use rxlsb::{XlsbReader, CellData};
use std::path::PathBuf;

let path = PathBuf::from("output.xlsb");
let mut reader = XlsbReader::builder().path(&path).build().unwrap();

// 流式读取
reader.for_each_row(0, |row_idx, cells| {
    println!("Row {}: {} cells", row_idx, cells.len());
}).unwrap();

// 分页读取
let rows = reader.read_rows(0, 0, 1000).unwrap();
println!("Read {} rows", rows.len());
```

## 安装

添加到 `Cargo.toml`:

```toml
[dependencies]
rxlsb = "0.2"
```

## API概览

### CellData数据类型

```rust
pub enum CellData {
    Text(String),               // 文本字符串
    Number(f64),                // 数值（IEEE 754双精度）
    NumberWithFormat(f64, String), // 带格式的数值
    Date(DateTime<Utc>),        // 日期时间
    DateWithFormat(i64, String), // 带格式的日期
    Bool(bool),                 // 布尔值
    Blank,                      // 空单元格
    Error(CellError),           // 错误值（#DIV/0!、#VALUE!等）
}
```

### 写入API

- `write_batch`: 批量写入整个sheet，适合中小数据量
- `start_sheet / write_rows / end_sheet`: 流式写入，适合大数据量

### 读取API

- `for_each_row`: 流式读取逐行处理，适合大数据量
- `read_rows`: 分页读取批量数据，适合分页展示

## 架构设计

- **第1层**: IO层 - BufferReader/BufferWriter（零拷贝Bytes）
- **第2层**: 容器层 - ZIP容器读写器
- **第3层**: 格式层 - BIFF12格式序列化/反序列化
- **第4层**: API层 - XlsbReader/XlsbWriter/TemplateFiller

## 项目状态

- ✅ 核心读写功能完整
- ✅ 模板填充支持
- ✅ 数字格式支持（百分比、货币、日期、时间、自定义）
- ✅ 所有测试通过
- ✅ 性能优化（全面超过Java jxlsb）
- ✅ API文档完整
- 🚧 公式单元格支持（计划0.3.0版本）
- 🚧 富文本单元格支持
- 🚧 图表支持

## 相关项目

- [jxlsb](https://github.com/itcraft-cn/jxlsb) - Java实现
- [cxlsb](https://github.com/itcraft-cn/cxlsb) - ANSI C实现
- [jsxlsb](https://github.com/itcraft-cn/jsxlsb) - Node.js实现

## 许可证

MIT License

## 贡献指南

欢迎贡献代码！提交PR前请阅读贡献指南。

## 变更日志

查看 [CHANGELOG.md](CHANGELOG.md) 了解版本历史。
