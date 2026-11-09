[中文](MANUAL.md) | **English**

# rxlsb Manual

> This manual is for developers using rxlsb. It covers requirements, dependency setup, build, API usage and examples.
> For a feature overview see [README_en.md](README_en.md); for version history see [CHANGELOG_en.md](CHANGELOG_en.md).

## 1. Introduction

rxlsb is a pure Rust XLSB (Excel binary) reader/writer library. Its fixed starting point is high-performance data output (writing); reading was initially meant only for consuming templates, and high-performance reading followed naturally once reading was complete. Built on a zero-copy `Bytes` architecture, it outperforms the Java jxlsb across the board.

## 2. Requirements

| Item | Requirement |
|------|-------------|
| Rust | Stable toolchain (edition 2021) |
| Dependencies | zip, bytes, thiserror, chrono, once_cell |

## 3. Installation & Build

Add to `Cargo.toml`:

```toml
[dependencies]
rxlsb = "0.2"
```

Build and test:

```bash
cargo build --release
cargo test
cargo run --example <name>
```

## 4. API Usage

### 4.1 Writing

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

Streaming write: `start_sheet / write_rows / end_sheet`, suitable for large datasets.

### 4.2 Reading

```rust
use rxlsb::{XlsbReader, CellData};
use std::path::PathBuf;

let path = PathBuf::from("output.xlsb");
let mut reader = XlsbReader::builder().path(&path).build().unwrap();

// Stream read
reader.for_each_row(0, |row_idx, cells| {
    println!("Row {}: {} cells", row_idx, cells.len());
}).unwrap();

// Paginated read
let rows = reader.read_rows(0, 0, 1000).unwrap();
println!("Read {} rows", rows.len());
```

### 4.3 Number Formats

```rust
use rxlsb::CellData;

let cell = CellData::percentage(0.123);            // 12.30%
let cell = CellData::number_with_comma(-1234.56);  // -1,234.56
let cell = CellData::number_negative_red(-500.0);  // -500.00 (red)
let cell = CellData::currency(1234.56);            // ￥1,234.56
let cell = CellData::date_from_timestamp(1714560000); // 5/1/2026 10:40
let cell = CellData::time(1714560000);             // 10:40:00
let cell = CellData::number_with_format(123.45, "#,##0.00"); // custom
```

### 4.4 Row/Column Limits and Compression

```rust
use rxlsb::{set_overflow, OverflowBehavior, XlsbWriter};

// Cut on column overflow, error on row overflow (global, snapshotted per write)
set_overflow(OverflowBehavior::Cut, OverflowBehavior::Error);

let mut writer = XlsbWriter::builder()
    .path("output.xlsb")
    .compression_level(6)   // 0=store, 1-9=deflate; default 1
    .build()?;
```

- Limits: `MAX_COLUMNS = 16384`, `MAX_ROWS = 1048576`
- `OverflowBehavior::Error` returns `XlsbError::LimitExceeded`; `Cut` truncates to the limit
- Configuration is global and snapshotted when each write operation starts
- Batch writes, streaming writes and template filling share the same policy

## 5. CellData Types

```rust
pub enum CellData {
    Text(String),                  // Text string
    Number(f64),                   // Number (IEEE 754 double)
    NumberWithFormat(f64, String), // Number with custom format
    Date(DateTime<Utc>),           // Date/Time
    DateWithFormat(i64, String),   // Date with format string
    Bool(bool),                    // Boolean
    Blank,                         // Empty cell
    Error(CellError),              // Error value (#DIV/0!, #VALUE!, etc.)
}
```

## 6. Write / Read API

| API | Description |
|-----|-------------|
| `write_batch` | Batch write an entire sheet, for small-medium data |
| `start_sheet / write_rows / end_sheet` | Streaming write, for large data |
| `for_each_row` | Row-by-row streaming read, for large data |
| `read_rows` | Paginated batch read, for pagination display |

## 7. Performance

| Operation | rxlsb (Rust) | jxlsb (Java) | Improvement |
|-----------|-------------|-------------|-------------|
| Stream write | 201K/s | 137K/s | **+46%** |
| Batch write | 190K/s | 111K/s | **+71%** |
| Stream read | 2.3M/s | 1.95M/s | **+18%** |
| Paginated read | 31K/s | 10K/s | **+210%** |

## 8. Compatibility

- Generated files open 100% correctly in WPS Office and Microsoft Excel
- Number formats (percentage, currency, date, time) display correctly
- Files from the streaming API match the batch API in quality
- Supports 10,000+ rows, multiple sheets, blank cells

## 9. Limitations & Roadmap

- No formula, rich text or chart support yet (v0.5.0 plans formula and advanced styles)
- Only basic styles for now

## 10. Further Docs

- [README_en.md](README_en.md) — feature overview
- [CHANGELOG_en.md](CHANGELOG_en.md) — version history
- `docs/style_index_mapping.md` — styleIndex mapping mechanism
- `examples/` — runnable examples
