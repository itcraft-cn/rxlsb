# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-01

### Added

- **Number format support**: Complete implementation of cell number formats
  - Percentage (0.00%)
  - Currency (￥#,##0.00)
  - Thousand separator (#,##0.00)
  - Negative numbers with red color (#,##0.00;[Red]-#,##0.00)
  - Date format (m/d/yy h:mm)
  - Time format (h:mm:ss)
  - Custom format support (ifmt 164+)

- **CellData API extensions**:
  - `CellData::percentage(value)` - Percentage cells
  - `CellData::percentage_with_decimals(value, decimals)` - Custom decimal percentage
  - `CellData::number_with_comma(value)` - Thousand separator
  - `CellData::number_negative_red(value)` - Negative numbers in red
  - `CellData::currency(value)` - Currency format (￥ symbol)
  - `CellData::currency_with_symbol(value, symbol)` - Custom currency symbol
  - `CellData::number_with_format(value, format)` - Custom number format
  - `CellData::date_from_timestamp(timestamp)` - Date from Unix timestamp
  - `CellData::date_with_format(timestamp, format)` - Custom date format
  - `CellData::time(timestamp)` - Time format
  - `CellData::time_with_format(timestamp, format)` - Custom time format

- **NumberFormatRegistry**: Manages built-in (0-22) and custom (164+) format IDs
  - Automatic built-in format recognition
  - Sequential custom format ID allocation
  - Format string deduplication

- **StylesRegistry**: Dynamic style XF management
  - Automatic style registration
  - Format reuse optimization
  - Correct styleIndex mapping mechanism

- **Documentation**:
  - `docs/style_index_mapping.md`: Complete styleIndex mapping mechanism documentation
  - Built-in format ID reference (0-22)
  - Custom format ID allocation strategy

- **Tools**:
  - `tools/verify_style_index.py`: Python script to verify styleIndex mapping in XLSB files

### Fixed

- **Critical: styleIndex mapping mechanism**
  - Discovered Excel/WPS auto +1 mapping: styles[i] → XF[i+1]
  - `get_style_id_for_format` returns styles list index (not i+1)
  - Verified with format_test.xlsb and format_test2.xlsb
  - All format columns display correctly in WPS

- **Date format correction**:
  - `date_from_timestamp` uses "m/d/yy h:mm" (ifmt=22) instead of "mm-dd-yy" (ifmt=14)
  - Matches jxlsb default date format

- **NumberFormatRegistry built-in format recognition**:
  - "m/d/yy h:mm" → ifmt=22 (built-in, not 164)
  - "h:mm:ss" → ifmt=21 (built-in, not 164)
  - Prevents unnecessary custom format allocation

### Performance

- No performance regression
- Format registration: O(n) lookup, O(1) append
- Style reuse: Format deduplication reduces XF count

### Tests

- Added format_test.rs example (matches jxlsb NumberFormatTest)
- Added format_test2.rs example (verifies generalization)
- All formats verified in WPS:
  - Percentage, currency, thousand separator ✓
  - Negative red, date, time ✓
  - Different column order, different format combinations ✓

### Technical Details

- **XF Structure**:
  - XF[0]: BrtBeginXFs (cell XF, ixf=0xffff)
  - XF[1+]: BrtBeginStyles (style XF, ixf=0x0000)
  - styles[i] → BrtBeginStyles[i] → XF[i+1]

- **styleIndex Mapping** (Critical discovery):
  - cell.styleIndex = styles list index (0-based)
  - Excel/WPS auto +1 → global XF index
  - Implementation: return i, NOT i+1

- **Format ID Allocation**:
  - Built-in formats: 0-22 (Excel predefined)
  - Custom formats: 164+ (sequential allocation)
  - Built-in format strings return built-in IDs, not custom IDs

### Known Issues

- Formula cells not supported yet (planned for 0.3.0)
- Rich text cells not supported yet
- Charts not supported yet
- Cell font, fill, border customization not supported (basic styles only)

### Migration Guide (0.1.0 → 0.2.0)

**New API**:
```rust
// Number formats
CellData::percentage(0.123)              // 12.30%
CellData::number_with_comma(-1234.56)    // -1,234.56
CellData::number_negative_red(-500.0)    // -500.00 (red)
CellData::currency(1234.56)              // ￥1,234.56

// Date/Time formats
CellData::date_from_timestamp(1714560000)  // 5/1/2026 10:40
CellData::time(1714560000)                  // 10:40:00
```

**No breaking changes**: All 0.1.0 API still works

## [0.1.0] - 2026-04-30

### Added

- Initial release of rxlsb
- Core read/write functionality for XLSB files
- `XlsbReader` with stream and paginated read modes
- `XlsbWriter` with batch and stream write modes
- `TemplateFiller` for template-based data filling
- `CellData` types: Text, Number, Date, Bool, Blank, Error
- Zero-copy Bytes-based architecture
- ZIP container with Deflated compression
- BIFF12 format implementation
- SST (Shared String Table) support
- Performance benchmark (1M rows test)

### Performance

- Stream write: 201K rows/sec (+46% vs jxlsb)
- Batch write: 190K rows/sec (+71% vs jxlsb)
- Stream read: 2.3M rows/sec (+18% vs jxlsb)
- Paginated read: 31K rows/sec (+210% vs jxlsb)

### Fixed

- BrtCellSt inline string format: VarInt → u32_le format
- ZIP compression: Stored → Deflated (file size reduced from 220MB to 31MB)
- Sheet cache for paginated read (10x performance improvement)
- Cells clone elimination (zero-copy optimization)

### Documentation

- Module-level documentation with performance comparison
- API documentation for CellData, XlsbReader, XlsbWriter
- Code examples in documentation
- README.md and README_cn.md with quick start guides

### Tests

- 9 tests passing (features_test: 6, template_test: 3)
- 0 compilation warnings
- All dead_code warnings resolved with #[allow(dead_code)]

### Known Issues

- Formula cells not supported yet
- Rich text cells not supported yet
- Charts not supported yet

## Future Plans

### [0.3.0] - Planned

- Formula cell support
- Rich text cell support
- Advanced cell styles (font, color, alignment, fill, border)
- Multi-sheet read/write optimization

### [0.4.0] - Planned

- Chart support
- Pivot table support
- Auto-filter support
- Conditional formatting

## Version History Summary

| Version | Date | Key Features |
|---------|------|--------------|
| 0.1.0 | 2026-04-30 | Initial release, core read/write, template filling |
| 0.2.0 | 2026-05-01 | Number formats (percentage, currency, date, time), styleIndex mapping |
| 0.3.0 | TBD | Formula, rich text, advanced styles |
| 0.4.0 | TBD | Charts, pivot tables, advanced features |