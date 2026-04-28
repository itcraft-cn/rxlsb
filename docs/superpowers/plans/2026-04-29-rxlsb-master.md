# rxlsb 实施计划（总）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建纯 Rust XLSB 读写库，功能与 jxlsb/cxlsb 完全对齐

**Architecture:** 分层架构 + BytesMut 零拷贝 + Trait 多态 + Result 错误处理

**Tech Stack:** Rust 1.70+ / zip crate / bytes crate / thiserror / chrono

---

## 文件结构总览

```
rxlsb/
├── Cargo.toml                      # 项目配置
├── src/
│   ├── lib.rs                      # 库入口
│   ├── error.rs                    # 错误定义
│   ├── io/                         # IO层（Phase 2）
│   ├── container/                  # 容器层（Phase 3）
│   ├── format/                     # 格式层（Phase 4-7）
│   ├── data/                       # 数据结构层（Phase 8）
│   └── api/                        # API层（Phase 8-9）
└── tests/                          # 测试（Phase 10）
```

---

## 详细计划索引

由于项目规模较大（预估12小时，10个Phase），拆分为以下详细计划：

1. **IO层详细计划**：`2026-04-29-rxlsb-io.md`
   - buffer_reader.rs
   - buffer_writer.rs
   - varint.rs
   - utf16.rs

2. **容器层详细计划**：`2026-04-29-rxlsb-container.md`
   - zip_reader.rs
   - zip_writer.rs
   - xml_gen.rs
   - rels_gen.rs

3. **格式层详细计划**：`2026-04-29-rxlsb-format.md`
   - record_types.rs
   - biff12.rs
   - sst_table.rs
   - styles_registry.rs
   - sheet_writer.rs
   - sheet_reader.rs
   - workbook_writer.rs
   - workbook_reader.rs

4. **API层详细计划**：`2026-04-29-rxlsb-api.md`
   - cell_data.rs
   - reader.rs
   - writer.rs
   - template.rs

5. **测试与示例详细计划**：`2026-04-29-rxlsb-tests.md`
   - 单元测试
   - 集成测试
   - 使用示例

---

## Phase 1: 项目骨架

### Task 1.1: 初始化项目

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/error.rs`

- [ ] **Step 1: 创建 Cargo.toml**

```toml
[package]
name = "rxlsb"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
description = "Pure Rust XLSB (Excel Binary Workbook) reader/writer library"
repository = "https://github.com/itcraft-cn/rxlsb"

[dependencies]
zip = "0.6"
bytes = "1.5"
thiserror = "1.0"
chrono = "0.4"

[dev-dependencies]
tempfile = "3.8"
```

- [ ] **Step 2: 创建 src/lib.rs**

```rust
mod error;
pub use error::{XlsbError, Result};
```

- [ ] **Step 3: 创建 src/error.rs**

```rust
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
}

pub type Result<T> = std::result::Result<T, XlsbError>;
```

- [ ] **Step 4: 运行 cargo check 验证编译**

Run: `cargo check`
Expected: PASS (无错误)

- [ ] **Step 5: 提交**

```bash
git add Cargo.toml src/lib.rs src/error.rs
git commit -m "feat: initialize project skeleton with error handling"
```

---

### Task 1.2: 创建模块目录结构

**Files:**
- Create: `src/io/mod.rs`
- Create: `src/container/mod.rs`
- Create: `src/format/mod.rs`
- Create: `src/data/mod.rs`
- Create: `src/api/mod.rs`

- [ ] **Step 1: 创建 src/io/mod.rs**

```rust
pub mod buffer_reader;
pub mod buffer_writer;
pub mod varint;
pub mod utf16;

pub use buffer_reader::BufferReader;
pub use buffer_writer::BufferWriter;
```

- [ ] **Step 2: 创建 src/container/mod.rs**

```rust
pub mod zip_reader;
pub mod zip_writer;
pub mod xml_gen;
pub mod rels_gen;

pub use zip_reader::XlsbContainerReader;
pub use zip_writer::XlsbContainerWriter;
```

- [ ] **Step 3: 创建 src/format/mod.rs**

```rust
pub mod record_types;
pub mod biff12;
pub mod sst_table;
pub mod styles_registry;
pub mod sheet_writer;
pub mod sheet_reader;
pub mod workbook_writer;
pub mod workbook_reader;

pub use record_types::RecordType;
pub use sst_table::SstTable;
pub use styles_registry::StylesRegistry;
```

- [ ] **Step 4: 创建 src/data/mod.rs**

```rust
pub mod sheet_info;
pub mod row_data;

pub use sheet_info::SheetInfo;
```

- [ ] **Step 5: 创建 src/api/mod.rs**

```rust
pub mod cell_data;
pub mod reader;
pub mod writer;
pub mod template;

pub use cell_data::{CellData, CellError};
pub use reader::XlsbReader;
pub use writer::XlsbWriter;
pub use template::TemplateFiller;
```

- [ ] **Step 6: 更新 src/lib.rs 导出模块**

```rust
mod error;
mod io;
mod container;
mod format;
mod data;
mod api;

pub use error::{XlsbError, Result};
pub use api::{CellData, CellError, XlsbReader, XlsbWriter, TemplateFiller};
pub use data::SheetInfo;
```

- [ ] **Step 7: 创建空模块文件占位**

创建每个模块下的空文件（暂时只声明模块）：
- `src/io/buffer_reader.rs`: 空文件
- `src/io/buffer_writer.rs`: 空文件
- `src/io/varint.rs`: 空文件
- `src/io/utf16.rs`: 空文件
- `src/container/zip_reader.rs`: 空文件
- `src/container/zip_writer.rs`: 空文件
- `src/container/xml_gen.rs`: 空文件
- `src/container/rels_gen.rs`: 空文件
- `src/format/record_types.rs`: 文件
- `src/format/biff12.rs`: 空文件
- `src/format/sst_table.rs`: 空文件
- `src/format/styles_registry.rs`: 空文件
- `src/format/sheet_writer.rs`: 空文件
- `src/format/sheet_reader.rs`: 空文件
- `src/format/workbook_writer.rs`: 空文件
- `src/format/workbook_reader.rs`: 空文件
- `src/data/sheet_info.rs`: 空文件
- `src/data/row_data.rs`: 空文件
- `src/api/cell_data.rs`: 空文件
- `src/api/reader.rs`: 空文件
- `src/api/writer.rs`: 空文件
- `src/api/template.rs`: 空文件

- [ ] **Step 8: 运行 cargo check 验证模块结构**

Run: `cargo check`
Expected: PASS

- [ ] **Step 9: 提交**

```bash
git add src/io/ src/container/ src/format/ src/data/ src/api/ src/lib.rs
git commit -m "feat: create module directory structure"
```

---

## Phase 依赖顺序

各 Phase 之间的依赖关系：

```
Phase 1 (骨架) ──┬──> Phase 2 (IO层)
                │
                ├──> Phase 3 (容器层)
                │
Phase 2 ────────┴──> Phase 4-7 (格式层)
                      │
Phase 3 ─────────────┘
                      │
                      ├──> Phase 8-9 (API层)
                      │
                      └──> Phase 10 (测试)
```

---

## 开发时间预估

| Phase | 内容 | 预估时间 | 详细计划 |
|-------|------|---------|---------|
| Phase 1 | 项目骨架 | 0.5h | 本文档 |
| Phase 2 | IO层 | 1h | 2026-04-29-rxlsb-io.md |
| Phase 3 | 容器层 | 1h | 2026-04-29-rxlsb-container.md |
| Phase 4-7 | 格式层 | 5h | 2026-04-29-rxlsb-format.md |
| Phase 8-9 | API层 | 3h | 2026-04-29-rxlsb-api.md |
| Phase 10 | 测试 | 2h | 2026-04-29-rxlsb-tests.md |

**总计：约 12 小时**

---

## 执行建议

每个详细计划文档可独立执行，建议顺序：

1. 先完成 **Phase 1**（项目骨架）
2. 并行执行 **Phase 2** 和 **Phase 3**（IO层和容器层无依赖）
3. 执行 **Phase 4-7**（格式层依赖IO层）
4. 执行 **Phase 8-9**（API层依赖格式层）
5. 执行 **Phase 10**（测试依赖全部）

---

**总计划完成，待详细计划编写后开始执行。**