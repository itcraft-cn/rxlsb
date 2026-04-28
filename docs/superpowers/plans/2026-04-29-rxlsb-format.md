# rxlsb 格式层详细计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 BIFF12 格式层，包括 RecordTypes、Biff12 trait、SstTable、StylesRegistry、SheetWriter/Reader、WorkbookWriter/Reader

**Architecture:** Trait多态 + enum记录类型 + 零拷贝序列化

**Tech Stack:** Rust std / bytes crate

---

## Phase 4: RecordTypes与Biff12 trait

### Task 4.1: RecordTypes enum

**Files:**
- Create: `src/format/record_types.rs`

- [ ] **Step 1: 实现记录类型枚举**

```rust
pub enum RecordType {
    BrtRowHdr = 0,
    BrtCellBlank = 1,
    BrtCellRk = 2,
    BrtCellError = 3,
    BrtCellBool = 4,
    BrtCellReal = 5,
    BrtCellSt = 6,
    BrtCellIsst = 7,
    BrtBeginSheet = 129,
    BrtEndSheet = 130,
    BrtBeginBook = 131,
    BrtEndBook = 132,
    BrtBeginSheetData = 145,
    BrtEndSheetData = 146,
    BrtBeginSst = 159,
    BrtEndSst = 160,
    BrtBeginStyleSheet = 370,
    BrtEndStyleSheet = 371,
}

impl RecordType {
    pub fn from_u32(code: u32) -> Option<Self> { /* match实现 */ }
    pub fn to_u32(&self) -> u32 { *self as u32 }
}
```

- [ ] **Step 2: 提交**

---

### Task 4.2: Biff12 trait

**Files:**
- Create: `src/format/biff12.rs`

- [ ] **Step 1: 定义 trait**

```rust
use crate::io::{BufferReader, BufferWriter};

pub trait Biff12Writer {
    fn write_record_header(&mut self, record_type: RecordType, size: u32);
    fn write_empty_record(&mut self, record_type: RecordType);
}

pub trait Biff12Reader {
    fn read_record_header(&mut self) -> Result<(RecordType, u32)>;
}
```

- [ ] **Step 2: 为 BufferWriter 实现 Biff12Writer**

- [ ] **Step 3: 为 BufferReader 实现 Biff12Reader**

- [ ] **Step 4: 提交**

---

## Phase 5: SstTable与StylesRegistry

### Task 5.1: SstTable

**Files:**
- Create: `src/format/sst_table.rs`

- [ ] **Step 1: 实现共享字符串表**

```rust
use std::collections::HashMap;

pub struct SstTable {
    strings: Vec<String>,
    hash_map: HashMap<String, u32>,
}

impl SstTable {
    pub fn new() -> Self { Self { strings: vec![], hash_map: HashMap::new() } }
    
    pub fn add_string(&mut self, s: &str) -> u32 {
        if let Some(idx) = self.hash_map.get(s) { return *idx; }
        let idx = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.hash_map.insert(s.to_string(), idx);
        idx
    }
    
    pub fn get_string(&self, idx: u32) -> Option<&str> {
        self.strings.get(idx as usize).map(|s| s.as_str())
    }
    
    pub fn count(&self) -> usize { self.strings.len() }
}
```

- [ ] **Step 2: 提交**

---

### Task 5.2: StylesRegistry

**Files:**
- Create: `src/format/styles_registry.rs`

- [ ] **Step 1: 实现样式注册表**

```rust
pub struct Font { name: String, size: f64, bold: bool }
pub struct Fill { pattern_type: u32, fg_color: u32 }
pub struct Border { left_style: u32, right_style: u32 }
pub struct Xf { font_id: u32, fill_id: u32, num_format_id: u32 }

pub struct StylesRegistry {
    fonts: Vec<Font>,
    fills: Vec<Fill>,
    borders: Vec<Border>,
    xfs: Vec<Xf>,
    num_formats: Vec<String>,
}

impl StylesRegistry {
    pub fn new() -> Self { /* 默认样式 */ }
    pub fn add_font(&mut self, font: Font) -> u32 { /* 实现 */ }
    pub fn add_xf(&mut self, xf: Xf) -> u32 { /* 实现 */ }
}
```

- [ ] **Step 2: 提交**

---

## Phase 6: SheetWriter与SheetReader

### Task 6.1: SheetWriter

**Files:**
- Create: `src/format/sheet_writer.rs`

- [ ] **Step 1: 实现 Sheet写入器**

```rust
use crate::io::BufferWriter;

pub struct SheetWriter<'a> {
    buffer: BufferWriter,
    sst: &'a mut SstTable,
    styles: &'a mut StylesRegistry,
}

impl<'a> SheetWriter<'a> {
    pub fn write_batch(&mut self, supplier: impl CellSupplier,
                       row_count: usize, col_count: usize) -> Result<()> {
        // 写入 BrtBeginSheet
        // 写入 BrtWsDim
        // 写入 BrtBeginSheetData
        // 写入每一行 BrtRowHdr + 单元格记录
        // 写入 BrtEndSheetData
        // 写入 BrtEndSheet
    }
    
    fn write_cell(&mut self, row: u32, col: u32, data: CellData) {
        match data {
            CellData::Text(s) => {
                if s.len() <= 3 {
                    self.write_cell_st(row, col, &s);
                } else {
                    let sst_idx = self.sst.add_string(&s);
                    self.write_cell_isst(row, col, sst_idx);
                }
            }
            CellData::Number(n) => {
                if can_encode_rk(n) {
                    self.write_cell_rk(row, col, n);
                } else {
                    self.write_cell_real(row, col, n);
                }
            }
            // ... 其他类型
        }
    }
}
```

- [ ] **Step 2: 提交**

---

### Task 6.2: SheetReader

**Files:**
- Create: `src/format/sheet_reader.rs`

- [ ] **Step 1: 实现 Sheet读取器**

```rust
pub struct SheetReader<'a> {
    buffer: BufferReader,
    sst: Option<&'a SstTable>,
}

impl<'a> SheetReader<'a> {
    pub fn for_each_row(&mut self, handler: impl RowHandler) -> Result<()> {
        // 解析记录
        // 当遇到 BrtRowHdr 时，处理上一行并开始新行
        // 当遇到单元格记录时，转换为 CellData
        // 当遇到 BrtEndSheetData 时，结束处理
    }
}
```

- [ ] **Step 2: 提交**

---

## Phase 7: WorkbookWriter与WorkbookReader

### Task 7.1: WorkbookWriter

**Files:**
- Create: `src/format/workbook_writer.rs`

- [ ] **Step 1: 实现 Workbook写入器**

```rust
pub struct WorkbookWriter {
    sheets: Vec<SheetEntry>,
}

impl WorkbookWriter {
    pub fn add_sheet(&mut self, name: &str) {
        self.sheets.push(SheetEntry { name: name.to_string() });
    }
    
    pub fn serialize(&self) -> Result<Bytes> {
        // 写入 BrtBeginBook
        // 写入每个 sheet 的 BrtBundleSh
        // 写入 BrtEndBook
    }
}
```

- [ ] **Step 2: 提交**

---

### Task 7.2: WorkbookReader

**Files:**
- Create: `src/format/workbook_reader.rs`

- [ ] **Step 1: 实现 Workbook读取器**

```rust
pub struct WorkbookReader {
    sheet_infos: Vec<SheetInfo>,
}

impl WorkbookReader {
    pub fn deserialize(data: &Bytes) -> Result<Self> {
        // 解析 BrtBeginBook
        // 解析每个 BrtBundleSh
        // 解析 BrtEndBook
    }
    
    pub fn get_sheet_infos(&self) -> &[SheetInfo] {
        &self.sheet_infos
    }
}
```

- [ ] **Step 2: 提交**

---

### Task 7.3: 更新模块导出

- [ ] **Step 1: 更新 src/format/mod.rs**

- [ ] **Step 2: 运行 cargo check**

- [ ] **Step 3: 提交**

---

**格式层计划完成。**