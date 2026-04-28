# rxlsb 容器层详细计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 XLSB ZIP容器读写，包括 ZipReader、ZipWriter、XmlGen、RelsGen

**Architecture:** zip crate 处理 ZIP 容器 + XML生成

**Tech Stack:** zip 0.6 / Rust std

---

## Task 3.1: XlsbContainerWriter

**Files:**
- Create: `src/container/zip_writer.rs`
- Test: `tests/container/test_zip_writer.rs`

- [ ] **Step 1: 创建测试文件**

```rust
use rxlsb::container::XlsbContainerWriter;
use tempfile::NamedTempFile;

#[test]
fn test_add_entry() {
    let temp = NamedTempFile::new().unwrap();
    let mut writer = XlsbContainerWriter::create(temp.path()).unwrap();
    writer.add_entry("test.txt", "Hello".as_bytes()).unwrap();
    writer.finish().unwrap();
}
```

- [ ] **Step 2: 实现 src/container/zip_writer.rs**

```rust
use zip::{ZipWriter, CompressionMethod, write::FileOptions};
use std::fs::File;
use std::path::Path;
use crate::error::Result;
use bytes::Bytes;

pub struct XlsbContainerWriter {
    writer: ZipWriter<File>,
}

impl XlsbContainerWriter {
    pub fn create(path: &Path) -> Result<Self> {
        let file = File::create(path)?;
        Ok(Self { writer: ZipWriter::new(file) })
    }
    
    pub fn add_entry(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Stored);
        self.writer.start_file(name, options)?;
        self.writer.write_all(data)?;
        Ok(())
    }
    
    pub fn finish(&mut self) -> Result<()> {
        self.writer.finish()?;
        Ok(())
    }
}
```

- [ ] **Step 3: 运行测试**

- [ ] **Step 4: 提交**

---

## Task 3.2: XlsbContainerReader

**Files:**
- Create: `src/container/zip_reader.rs`

- [ ] **Step 1: 实现 src/container/zip_reader.rs**

```rust
use zip::ZipArchive;
use std::fs::File;
use std::path::Path;
use bytes::{Bytes, BytesMut};

pub struct XlsbContainerReader {
    archive: ZipArchive<File>,
}

impl XlsbContainerReader {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self { archive: ZipArchive::new(file)? })
    }
    
    pub fn has_entry(&self, name: &str) -> bool {
        self.archive.by_name(name).is_ok()
    }
    
    pub fn read_entry(&mut self, name: &str) -> Result<Bytes> {
        let mut file = self.archive.by_name(name)?;
        let mut buffer = BytesMut::with_capacity(file.size() as usize);
        std::io::copy(&mut file, &mut buffer)?;
        Ok(buffer.freeze())
    }
    
    pub fn get_sheet_data(&mut self, sheet_idx: usize) -> Result<Bytes> {
        self.read_entry(&format!("xl/worksheets/sheet{}.bin", sheet_idx + 1))
    }
}
```

- [ ] **Step 2: 提交**

---

## Task 3.3: XmlGen

**Files:**
- Create: `src/container/xml_gen.rs`

- [ ] **Step 1: 实现 src/container/xml_gen.rs**

```rust
use bytes::Bytes;

pub struct XmlGen;

impl XmlGen {
    pub fn content_types(sheet_count: usize, has_sst: bool) -> Bytes {
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
            <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">
            <Default Extension=\"bin\" ContentType=\"application/vnd.ms-excel.sheet.binary.macroEnabled.main\"/>
            {}
            </Types>",
            generate_sheet_overrides(sheet_count, has_sst)
        );
        Bytes::copy_from_slice(xml.as_bytes())
    }
    
    pub fn app_xml(sheet_count: usize) -> Bytes {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
            <Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\">
            <Application>rxlsb</Application>
            <SheetCount>{}</SheetCount>
            </Properties>",
            sheet_count
        ).into()
    }
    
    pub fn core_xml() -> Bytes { /* 实现 */ }
    pub fn theme_xml() -> Bytes { /* 实现 */ }
}
```

- [ ] **Step 2: 提交**

---

## Task 3.4: RelsGen

**Files:**
- Create: `src/container/rels_gen.rs`

- [ ] **Step 1: 实现 src/container/rels_gen.rs**

```rust
use bytes::Bytes;

pub struct RelsGen;

impl RelsGen {
    pub fn root_rels() -> Bytes {
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">
        <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.bin\"/>
        </Relationships>".into()
    }
    
    pub fn workbook_rels(sheet_count: usize, has_sst: bool) -> Bytes {
        // 生成 sheet 关系
        let mut xml = String::new();
        for i in 1..=sheet_count {
            xml.push_str(&format!(
                "<Relationship Id=\"rId{}\" Type=\"...worksheet\" Target=\"worksheets/sheet{}.bin\"/>",
                i, i
            ));
        }
        // ... 完整实现
        Bytes::copy_from_slice(xml.as_bytes())
    }
}
```

- [ ] **Step 2: 提交**

---

## Task 3.5: 更新模块导出

- [ ] **Step 1: 更新 src/container/mod.rs**

- [ ] **Step 2: 运行 cargo check**

- [ ] **Step 3: 提交**

---

**容器层计划完成。**