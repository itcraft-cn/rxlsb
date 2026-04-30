# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

### [0.2.0] - Planned

- Formula cell support
- Rich text cell support
- Cell style support (font, color, alignment)
- Multi-sheet read/write optimization

### [0.3.0] - Planned

- Chart support
- Pivot table support
- Auto-filter support
- Conditional formatting

## Version History Summary

| Version | Date | Key Features |
|---------|------|--------------|
| 0.1.0 | 2026-04-30 | Initial release, core read/write, template filling |
| 0.2.0 | TBD | Formula, rich text, cell styles |
| 0.3.0 | TBD | Charts, pivot tables, advanced features |