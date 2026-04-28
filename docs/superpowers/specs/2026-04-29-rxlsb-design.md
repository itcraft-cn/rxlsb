# rxlsb 设计文档（总）

## 一、项目概述

### 1.1 项目目标

构建一个纯 Rust 实现的 XLSB（Excel Binary Workbook）格式读写库，具备以下特性：

- **零拷贝架构**：使用 BytesMut（bytes crate）零拷贝缓冲区
- **极致性能**：读取 >100MB/s，写入 >80MB/s
- **全平台支持**：Linux + Windows + macOS
- **功能完整**：与 Java/C 版本（jxlsb/cxlsb）完全对齐
- **Rust 原生设计**：Result 错误处理、RAII 资源管理、Trait 多态

### 1.2 功能范围

**基础读写功能：**
- 单元格数据类型：文本、数值、日期、布尔、错误、空白
- 行/列操作：读取、写入、批量操作
- 多 Sheet 支持：Workbook 包含多个 Sheet
- 流式 API：write_batch / write_rows / for_each_row / read_rows

**高级功能：**
- 模板填充：fill_batch / fill_at_marker / start_fill / fill_rows / end_fill
- 样式支持：字体、边框、填充、对齐、数字格式
- 合并单元格：读取和写入合并单元格
- 共享字符串表（SST）：文本优化存储

### 1.3 技术约束

- 包名：`rxlsb`
- Rust 版本：1.70+（使用 bytes 1.x）
- 依赖：zip + bytes + thiserror + chrono
- 构建：Cargo
- 错误处理：thiserror + Result<T, XlsbError>
- 测试：cargo test + 集成测试

---

## 二、整体架构

### 2.1 架构分层

```
┌──────────────────────────────────────────────────────────────┐
│                      API Layer                                │
│  XlsbReader::builder().path(...).build()                     │
│  XlsbWriter::builder().path(...).build()                     │
│  write_batch / start_sheet / write_rows / end_sheet          │
│  for_each_row / read_rows                                     │
│  TemplateFiller: fill_batch / fill_at_marker                  │
├──────────────────────────────────────────────────────────────┤
│                    Data Structure Layer                       │
│  CellData enum: Text/Number/Date/Bool/Blank/Error            │
│  RowData: Vec<CellData> 或 CellSupplier trait             │
│  SheetInfo: sheet_index, name, row_count, col_count          │
├──────────────────────────────────────────────────────────────┤
│                     Format Layer                              │
│  Biff12Reader trait: parse_record / parse_cell               │
│  Biff12Writer trait: write_record / write_cell               │
│  RecordTypes enum: BrtRowHdr, BrtCellReal, BrtCellIsst...    │
│  SstTable: HashMap<String, u32> 共享字符串表                  │
│  StylesRegistry: fonts, fills, borders, xfs                  │
├──────────────────────────────────────────────────────────────┤
│                       IO Layer                                │
│  BufferReader: Bytes零拷贝读取                                │
│  BufferWriter: BytesMut零拷贝写入                             │
│  VarInt: encode/decode BIFF12变长整数                        │
│  Utf16Le: UTF-8 <-> UTF-16LE转换                             │
├──────────────────────────────────────────────────────────────┤
│                    Container Layer                            │
│  ZipReader: zip crate读取XLSB ZIP容器                         │
│  ZipWriter: zip crate写入XLSB ZIP容器                         │
│  XmlGen: [Content_Types].xml, _rels/.rels生成                │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 核心设计原则

1. **BytesMut零拷贝**：使用 bytes crate 实现内存池化零拷贝
2. **Result错误处理**：所有 API 返回 Result<T, XlsbError>
3. **RAII资源管理**：资源在 Drop 时自动释放
4. **Trait多态**：Biff12Reader/Writer trait，支持扩展
5. **流式处理**：避免全量加载到内存，支持 GB 级文件
6. **性能优化**：SST去重、RK编码、批量写入

---

## 三、目录结构

```
rxlsb/
├── Cargo.toml                      # 项目配置
├── README.md                       # 项目说明
├── LICENSE                         # Apache 2.0
├── src/
│   ├── lib.rs                      # 库入口，导出公共API
│   ├── api/                        # API层
│   │   ├── mod.rs
│   │   ├── reader.rs               # XlsbReader + Builder
│   │   ├── writer.rs               # XlsbWriter + Builder
│   │   ├── template.rs             # TemplateFiller
│   │   └── cell_data.rs            # CellData enum + CellSupplier trait
│   ├── data/                       # 数据结构层
│   │   ├── mod.rs
│   │   ├── sheet_info.rs           # SheetInfo结构
│   │   └── row_data.rs             # RowData结构
│   ├── format/                     # 格式层
│   │   ├── mod.rs
│   │   ├── biff12.rs               # Biff12Reader/Writer trait
│   │   ├── record_types.rs         # RecordTypes enum
│   │   ├── sheet_writer.rs         # Sheet写入器
│   │   ├── sheet_reader.rs         # Sheet读取器
│   │   ├── workbook_writer.rs      # Workbook写入器
│   │   ├── workbook_reader.rs      # Workbook读取器
│   │   ├── sst_table.rs            # 共享字符串表
│   │   └── styles_registry.rs      # 样式注册表
│   ├── io/                         # IO层
│   │   ├── mod.rs
│   │   ├── buffer_reader.rs        # Bytes零拷贝读取
│   │   ├── buffer_writer.rs        # BytesMut零拷贝写入
│   │   ├── varint.rs               # VarInt编解码
│   │   └── utf16.rs                # UTF-16LE处理
│   ├── container/                  # 容器层
│   │   ├── mod.rs
│   │   ├── zip_reader.rs           # ZIP容器读取
│   │   ├── zip_writer.rs           # ZIP容器写入
│   │   ├── xml_gen.rs              # XML生成
│   │   └── rels_gen.rs             # 关系文件生成
│   └── error.rs                    # XlsbError enum
├── tests/                          # 集成测试
└── examples/                       # 使用示例
```

---

## 四、详细设计文档索引

由于设计内容较多，拆分为以下详细设计文档：

1. **API层详细设计**：`2026-04-29-rxlsb-api.md`
   - CellData enum
   - CellSupplier trait
   - RowHandler trait
   - XlsbWriter API
   - XlsbReader API
   - TemplateFiller API

2. **格式层详细设计**：`2026-04-29-rxlsb-format.md`
   - RecordTypes enum
   - Biff12Writer trait
   - Biff12Reader trait
   - CellRecord enum
   - SstTable
   - StylesRegistry

3. **IO层详细设计**：`2026-04-29-rxlsb-io.md`
   - BufferReader
   - BufferWriter
   - VarInt编解码
   - Utf16Le处理

4. **容器层详细设计**：`2026-04-29-rxlsb-container.md`
   - ZipWriter
   - ZipReader
   - XmlGen
   - RelsGen

---

## 五、错误处理

### 5.1 XlsbError定义

使用 thiserror crate 定义统一错误类型：

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

---

## 六、性能目标

| 指标 | 目标值 | 实现策略 |
|------|--------|---------|
| 读取速度 | >100MB/s | Bytes零拷贝 + 逐行解析 |
| 写入速度 | >80MB/s | BytesMut池化 + SST优化 |
| 堆内存占用 | <10MB | Bytes池化 + RAII |
| SST去重率 | >50% | HashMap查找 + 短字符串inline |
| 大文件支持 | GB级 | 流式处理，不全量加载 |

---

## 七、依赖配置

### Cargo.toml

```toml
[package]
name = "rxlsb"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"

[dependencies]
zip = "0.6"
bytes = "1.5"
thiserror = "1.0"
chrono = "0.4"

[dev-dependencies]
tempfile = "3.8"
```

---

## 八、开发计划

| Phase | 内容 | 预估工作量 |
|-------|------|-----------|
| Phase 1 | 项目骨架 + Cargo配置 + 目录结构 | 0.5h |
| Phase 2 | IO层：buffer_reader, buffer_writer, varint, utf16 | 1h |
| Phase 3 | 容器层：zip_reader, zip_writer, xml_gen, rels_gen | 1h |
| Phase 4 | 格式层：record_types, biff12 trait | 1.5h |
| Phase 5 | 格式层：sst_table, styles_registry | 1h |
| Phase 6 | 格式层：sheet_writer, sheet_reader | 1.5h |
| Phase 7 | 格式层：workbook_writer, workbook_reader | 1h |
| Phase 8 | API层：cell_data, reader, writer | 2h |
| Phase 9 | API层：template_filler | 1h |
| Phase 10 | 测试 + 示例 | 2h |

**总计预估：约 12 小时**

---

## 九、与Java/C版本对比

| 特性 | jxlsb (Java) | cxlsb (C) | rxlsb (Rust) |
|------|-------------|----------|-------------|
| 内存架构 | 堆外内存 (ByteBuffer/MemorySegment) | 动态缓冲区 | BytesMut零拷贝 |
| 错误处理 | 异常机制 | 错误码返回值 | Result<T, E> |
| API风格 | Builder + 函数式接口 | 函数指针回调 | Builder + 闭包 |
| 资源管理 | Cleaner/AutoCloseable | 手动destroy | RAII Drop |
| 多态实现 | 接口抽象 | 函数指针 | Trait |
| 平台支持 | Java虚拟机 | 跨平台C | 全平台Rust |

---

## 十、参考资料

- **[MS-XLSB]**: Excel Binary Workbook (.xlsb) File Format - Microsoft Open Specifications
- **jxlsb**: Java XLSB库实现
- **cxlsb**: ANSI C XLSB库实现
- **bytes crate**: https://docs.rs/bytes
- **zip crate**: https://docs.rs/zip

---

**设计完成，待用户审批后进入 writing-plans 阶段生成详细实施计划。**