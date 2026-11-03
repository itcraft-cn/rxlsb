**中文** | [English](MANUAL_en.md)

# rxlsb 使用手册

> 本手册面向使用 rxlsb 的开发者，覆盖环境要求、依赖引入、构建、API 用法与示例。
> 特性概览见 [README.md](README.md)，版本历史见 [CHANGELOG.md](CHANGELOG.md)。

## 1. 简介

rxlsb 是纯 Rust 实现的 XLSB（Excel 二进制）读写库。设计出发点为高性能数据输出（写入）；读取最初用于读模板，读能力完备后自然延伸出高性能读。基于 `Bytes` 的零拷贝架构，性能全面超过 Java jxlsb。

## 2. 环境要求

| 项 | 要求 |
|----|------|
| Rust | 稳定版工具链（edition 2021） |
| 依赖 | zip、bytes、thiserror、chrono、once_cell |

## 3. 安装与构建

在 `Cargo.toml` 中加入：

```toml
[dependencies]
rxlsb = "0.2"
```

构建与测试：

```bash
cargo build --release
cargo test
cargo run --example <name>
```

## 4. API 用法

### 4.1 写入

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

流式写入：`start_sheet / write_rows / end_sheet`，适合大数据集。

### 4.2 读取

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

### 4.3 数字格式

```rust
use rxlsb::CellData;

let cell = CellData::percentage(0.123);            // 12.30%
let cell = CellData::number_with_comma(-1234.56);  // -1,234.56
let cell = CellData::number_negative_red(-500.0);  // -500.00（红）
let cell = CellData::currency(1234.56);            // ￥1,234.56
let cell = CellData::date_from_timestamp(1714560000); // 5/1/2026 10:40
let cell = CellData::time(1714560000);             // 10:40:00
let cell = CellData::number_with_format(123.45, "#,##0.00"); // 自定义
```

### 4.4 行列上限与压缩级别

```rust
use rxlsb::{set_overflow, OverflowBehavior, XlsbWriter};

// 列超限截断、行超限报错（全局配置，操作开始时快照）
set_overflow(OverflowBehavior::Cut, OverflowBehavior::Error);

let mut writer = XlsbWriter::builder()
    .path("output.xlsb")
    .compression_level(6)   // 0=store，1-9=deflate；默认 1
    .build()?;
```

- 上限常量：`MAX_COLUMNS = 16384`、`MAX_ROWS = 1048576`
- 超限行为：`OverflowBehavior::Error` 返回 `XlsbError::LimitExceeded`；`Cut` 截断到上限
- 配置是全局的，每次写入操作开始时快照，只影响之后的操作
- 批量写、流式写、模板填充均受同一策略约束

## 5. CellData 类型

```rust
pub enum CellData {
    Text(String),                  // 文本
    Number(f64),                   // 数字（IEEE 754 double）
    NumberWithFormat(f64, String), // 带自定义格式的数字
    Date(DateTime<Utc>),           // 日期/时间
    DateWithFormat(i64, String),   // 带格式字符串的日期
    Bool(bool),                    // 布尔
    Blank,                         // 空单元格
    Error(CellError),              // 错误值（#DIV/0!、#VALUE! 等）
}
```

## 6. 写入 / 读取 API

| API | 说明 |
|-----|------|
| `write_batch` | 批量写入整张工作表，适合中小数据量 |
| `start_sheet / write_rows / end_sheet` | 流式写入，适合大数据量 |
| `for_each_row` | 逐行流式读取，适合大数据量 |
| `read_rows` | 分页批量读取，适合分页展示 |

## 7. 性能对比

| 操作 | rxlsb (Rust) | jxlsb (Java) | 提升 |
|------|-------------|-------------|------|
| 流式写 | 201K/s | 137K/s | **+46%** |
| 批量写 | 190K/s | 111K/s | **+71%** |
| 流式读 | 2.3M/s | 1.95M/s | **+18%** |
| 分页读 | 31K/s | 10K/s | **+210%** |

## 8. 兼容性

- 生成文件在 WPS Office 与 Microsoft Excel 中 100% 正确打开
- 数字格式（百分比、货币、日期、时间）显示正确
- 流式 API 生成文件与批量 API 同等质量
- 支持 10,000+ 行、多工作表、空白单元格

## 9. 限制与计划

- 暂不支持公式、富文本、图表（v0.4.0 计划支持公式与高级样式）
- 当前仅基础样式

## 10. 更多文档

- [README.md](README.md) — 特性概览
- [CHANGELOG.md](CHANGELOG.md) — 版本历史
- `docs/style_index_mapping.md` — styleIndex 映射机制
- `examples/` — 可运行示例
