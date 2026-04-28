# rxlsb 测试与示例详细计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-step. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 完整测试覆盖和使用示例，确保生产级质量

**Architecture:** 单元测试 + 集成测试 + 示例代码

**Tech Stack:** Rust test / tempfile

---

## Task 10.1: 写入集成测试

**Files:**
- Create: `tests/integration/test_write.rs`

- [ ] **Step 1: 创建简单写入测试**

```rust
use rxlsb::{XlsbWriter, CellData};
use tempfile::NamedTempFile;

#[test]
fn test_write_simple() {
    let temp = NamedTempFile::new().unwrap();
    
    let mut writer = XlsbWriter::builder()
        .path(temp.path())
        .build()
        .unwrap();
    
    writer.write_batch("Sheet1", |row, col| {
        match col % 3 {
            0 => CellData::text(format!("Item-{}", row)),
            1 => CellData::number(row as f64 * 10.0),
            2 => CellData::bool(row % 2 == 0),
            _ => CellData::blank(),
        }
    }, 100, 3).unwrap();
    
    writer.close().unwrap();
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test test_write_simple`
Expected: PASS

- [ ] **Step 3: 提交**

---

## Task 10.2: 读取集成测试

**Files:**
- Create: `tests/integration/test_read.rs`

- [ ] **Step 1: 创建读取测试（基于写入测试生成的文件）**

```rust
#[test]
fn test_read_simple() {
    // 先写入文件
    let temp = NamedTempFile::new().unwrap();
    write_test_file(temp.path());
    
    // 再读取
    let mut reader = XlsbReader::builder()
        .path(temp.path())
        .build()
        .unwrap();
    
    let infos = reader.get_sheet_infos();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].name, "Sheet1");
    
    let mut row_count = 0;
    reader.for_each_row(0, |idx, cells| {
        row_count += 1;
        assert_eq!(cells.len(), 3);
    }).unwrap();
    
    assert_eq!(row_count, 100);
}
```

- [ ] **Step 2: 运行测试**

- [ ] **Step 3: 提交**

---

## Task 10.3: 读写往返测试

**Files:**
- Create: `tests/integration/test_roundtrip.rs`

- [ ] **Step 1: 创建往返测试**

```rust
#[test]
fn test_roundtrip() {
    let data = generate_test_data(1000, 10);
    
    let temp1 = NamedTempFile::new().unwrap();
    write_data(temp1.path(), &data);
    
    let temp2 = NamedTempFile::new().unwrap();
    let read_data = read_data(temp1.path());
    write_data(temp2.path(), &read_data);
    
    let verify_data = read_data(temp2.path());
    
    assert_data_equal(&data, &verify_data);
}
```

- [ ] **Step 2: 运行测试**

- [ ] **Step 3: 提交**

---

## Task 10.4: 模板填充测试

**Files:**
- Create: `tests/integration/test_template.rs`

- [ ] **Step 1: 创建模板填充测试（使用 demo_template.xlsb）**

```rust
#[test]
fn test_template_fill() {
    let template = Path::new("tests/resources/template/demo_template.xlsb");
    let output = NamedTempFile::new().unwrap();
    
    let mut filler = TemplateFiller::builder()
        .template(template)
        .output(output.path())
        .build()
        .unwrap();
    
    filler.fill_at_marker(0, "${data}", |row, col| {
        CellData::text(format!("Row-{} Col-{}", row, col))
    }, 10, 5).unwrap();
    
    filler.save().unwrap();
    
    // 验证输出文件
    let mut reader = XlsbReader::builder()
        .path(output.path())
        .build()
        .unwrap();
    
    // ... 验证填充的数据
}
```

- [ ] **Step 2: 运行测试**

- [ ] **Step 3: 提交**

---

## Task 10.5: 流式写入测试

**Files:**
- Create: `tests/integration/test_streaming.rs`

- [ ] **Step 1: 创建流式写入测试**

```rust
#[test]
fn test_streaming_write() {
    let temp = NamedTempFile::new().unwrap();
    
    let mut writer = XlsbWriter::builder()
        .path(temp.path())
        .build()
        .unwrap();
    
    writer.start_sheet("Data", 4).unwrap();
    
    for batch in 0..10 {
        writer.write_rows(batch * 1000, |row, col| {
            CellData::number(row as f64 * col as f64)
        }, 1000).unwrap();
    }
    
    writer.end_sheet().unwrap();
    writer.close().unwrap();
    
    // 验证文件包含 10000 行
}
```

- [ ] **Step 2: 运行测试**

- [ ] **Step 3: 提交**

---

## Task 10.6: 使用示例

**Files:**
- Create: `examples/example_write.rs`
- Create: `examples/example_read.rs`
- Create: `examples/example_streaming.rs`

- [ ] **Step 1: 创建写入示例**

```rust
use rxlsb::{XlsbWriter, CellData};

fn main() {
    let mut writer = XlsbWriter::builder()
        .path("output.xlsb")
        .build()
        .unwrap();
    
    writer.write_batch("Sheet1", |row, col| {
        CellData::text(format!("Cell-{}-{}", row, col))
    }, 100, 5).unwrap();
    
    writer.close().unwrap();
    
    println!("XLSB file created: output.xlsb");
}
```

- [ ] **Step 2: 创建读取示例**

- [ ] **Step 3: 创建流式示例**

- [ ] **Step 4: 运行示例验证**

Run: `cargo run --example example_write`
Expected: 成功生成 output.xlsb

- [ ] **Step 5: 提交**

---

## Task 10.7: 最终集成测试

- [ ] **Step 1: 运行所有测试**

Run: `cargo test`
Expected: 全部 PASS

- [ ] **Step 2: 运行 cargo doc**

Run: `cargo doc --no-deps`
Expected: 成功生成文档

- [ ] **Step 3: 最终提交**

```bash
git add tests/ examples/
git commit -m "test: complete integration tests and examples"
```

---

**测试与示例计划完成。**