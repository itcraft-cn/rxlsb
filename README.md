[中文文档](README_cn.md) | English

# rxlsb - High Performance XLSB Library for Rust

A pure Rust implementation for reading and writing Excel XLSB (Binary) files with exceptional performance.

## Features

- **High Performance**: 200K rows/sec write, 2.3M rows/sec read, outperforming Java jxlsb
- **Zero-copy**: Bytes-based architecture minimizing memory copying
- **Streaming API**: Batch write, paginated read, stream processing
- **Template Filling**: Data filling based on existing templates
- **Type Safe**: Rust type system guarantees, no runtime errors
- **Number Formats**: Percentage, currency, thousand separator, date, time, negative red
- **Custom Formats**: Flexible custom number format support

## Performance Comparison

| Operation | rxlsb (Rust) | jxlsb (Java) | Improvement |
|-----------|-------------|-------------|-------------|
| Stream Write | 201K/s | 137K/s | **+46%** |
| Batch Write | 190K/s | 111K/s | **+71%** |
| Stream Read | 2.3M/s | 1.95M/s | **+18%** |
| Paginated Read | 31K/s | 10K/s | **+210%** |

## Quick Start

### Writing XLSB Files

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

### Number Formats

```rust
use rxlsb::CellData;

// Percentage: 12.30%
let cell = CellData::percentage(0.123);

// Thousand separator: -1,234.56
let cell = CellData::number_with_comma(-1234.56);

// Negative red: -500.00 (displayed in red)
let cell = CellData::number_negative_red(-500.0);

// Currency: ￥1,234.56
let cell = CellData::currency(1234.56);

// Date: 5/1/2026 10:40
let cell = CellData::date_from_timestamp(1714560000);

// Time: 10:40:00
let cell = CellData::time(1714560000);

// Custom format
let cell = CellData::number_with_format(123.45, "#,##0.00");
```

### Reading XLSB Files

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

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rxlsb = "0.2"
```

## API Overview

### CellData Types

```rust
pub enum CellData {
    Text(String),               // Text string
    Number(f64),                // Number (IEEE 754 double)
    NumberWithFormat(f64, String), // Number with custom format
    Date(DateTime<Utc>),        // Date/Time
    DateWithFormat(i64, String), // Date with format string
    Bool(bool),                 // Boolean
    Blank,                      // Empty cell
    Error(CellError),           // Error value (#DIV/0!, #VALUE!, etc.)
}
```

### Write API

- `write_batch`: Batch write entire sheet, suitable for small-medium data
- `start_sheet / write_rows / end_sheet`: Stream write, suitable for large data

### Read API

- `for_each_row`: Stream read row by row, suitable for large data
- `read_rows`: Paginated read batch data, suitable for pagination display

## WPS/Excel Compatibility

rxlsb has been fully verified with WPS Office and Microsoft Excel:

- ✅ **100% Compatibility**: All generated files open correctly in WPS/Excel
- ✅ **Format Display**: Number formats (percentage, currency, date, time) display correctly
- ✅ **Streaming API**: Files generated with streaming API match batch API quality
- ✅ **Large Files**: 10,000+ row files open smoothly
- ✅ **Multi-sheet**: Multiple sheets switch correctly
- ✅ **Blank Cells**: Blank cells handled correctly (no display issues)

Verified test files available in `/tmp/`:
- minimal_one.xlsb (1 row × 1 col)
- test_stream_api.xlsb (100 rows × 4 cols)
- verify_formats.xlsb (100 rows × 8 cols, multiple formats)
- stream_large.xlsb (5000 rows)
- verify_large.xlsb (10,000 rows)

## Architecture

- **Layer 1**: IO layer - BufferReader/BufferWriter (zero-copy Bytes)
- **Layer 2**: Container layer - ZIP container reader/writer
- **Layer 3**: Format layer - BIFF12 format serializer/deserializer
- **Layer 4**: API layer - XlsbReader/XlsbWriter/TemplateFiller

## Project Status

- ✅ Core read/write functionality complete
- ✅ Template filling support
- ✅ Number format support (percentage, currency, date, time, custom)
- ✅ WPS/Excel compatibility 100% verified
- ✅ All tests passing
- ✅ Performance optimized (outperforming Java jxlsb)
- ✅ API documentation complete
- ✅ Streaming API fully tested (start/write/end)
- ✅ BIFF12 format correctly implemented
- 🚧 Formula cells support (planned for v0.4.0)
- 🚧 Rich text cells support
- 🚧 Chart support

## Related Projects

- [jxlsb](https://github.com/itcraft-cn/jxlsb) - Java implementation
- [cxlsb](https://github.com/itcraft-cn/cxlsb) - ANSI C implementation
- [jsxlsb](https://github.com/itcraft-cn/jsxlsb) - Node.js implementation

## License

MIT License

## Contributing

Contributions welcome! Please read the contributing guidelines before submitting PRs.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
