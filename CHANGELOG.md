**中文** | [English](CHANGELOG_en.md)

# 变更日志

本项目所有值得注意的变更都记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 变更

- 文档：新增「设计定位」章节
- 版本：进入 0.4.0-dev
- **默认 deflate 级别改为 1（速度优先）**：与主流表格库一致，同级别下文件体积相当；
  需要更小文件可用 `XlsbWriterBuilder::compression_level(6)` 或
  `XlsbWriter::set_compression_level(6)`
- 新增写入器压缩级别配置：`XlsbWriterBuilder::compression_level(i32)`、
  `XlsbWriter::set_compression_level(i32)`

### 修复

- 修正 `BrtCellBool` 读取：跳过 flags 字节（修复加入 flags 后反序列化错位）
- 读取测试数据第 4 列由空白改为真实数值（配合「跳过空白单元格」语义）

### 性能

- ZIP 输出使用 `zip` crate + `BufWriter`（256 KiB），减少小块文件写；**不做自研 ZIP**
- 新增可选 `zlib-ng` feature（C 后端）：默认纯 Rust（miniz_oxide）；
  `cargo build --features zlib-ng` 切换到更快的 C 版 deflate
- SST 序列化：单次 UTF-16 编码 + 复用 scratch 缓冲，消除每条字符串的 `Vec<u16>` 分配
- `write_batch` 预留 SST 容量，避免写入中反复 rehash/扩容
- SST 改为「移动存储 + 哈希索引（FNV-1a）」，每个唯一字符串仅一次分配，去掉 `Arc` 开销
- SST 支持分块流式压缩写入（累积 64 KiB 再写入），减少中间缓冲
- SST 使用 **ASCII 快速路径的 UTF-8→UTF-16 编码**，避免 `encode_utf16` 迭代器开销
- SST 去重索引使用**恒等哈希器**（键已是 FNV-1a 结果，省去二次 SipHash）
- 流式写入改为**增量编码行记录**：`write_rows` 直接写入行记录缓冲，不再缓存整张
  `Vec<Vec<CellData>>`
- 流式 `start_sheet` 即写入前导并记录维度偏移，`end_sheet` 仅回填 `last_row` 并追加
  收尾，**去掉结束时的整块 650KB 拷贝**（峰值堆与分配进一步下降）
- 单元格与行头改为**固定长度一次性编码 + 单次 `put_slice`**，去掉大量逐字段写入的边界检查
- sheet 序列化改为移动缓冲区，去掉一次全量拷贝
- 实测（5000 行 × 7 列，同轮对手基准）：
  - 默认纯 Rust、deflate 1：批写 3.9ms（对手 5.5ms）、流式 4.1ms（对手 4.5ms）
  - 开启 `zlib-ng`、deflate 6：批写 9.1ms（对手 12.9ms）、流式 9.5ms（对手 9.3ms）
  - 产物体积略小于对手；总分配低于对手（6.2/6.6MiB vs 6.8/7.3MiB）


## [0.3.0] - 2026-05-07

### 新增

- **流式 API 完整测试套件**：
  - `start_sheet / write_rows / end_sheet` 全面测试
  - 验证与批量 API 100% 兼容
  - 多工作表流式支持测试
  - 批量追加模式测试（无缝拼接）
  - 大数据流式测试（5000 行）

- **综合测试示例**：
  - `minimal_one.rs`：最小单单元格测试（1×1）
  - `test_stream_api.rs`：流式 API 与批量 API 对比
  - `streaming_full_test.rs`：完整流式测试套件
  - `gen_verify.rs`：完整验证文件生成器

### 修复

- **严重：WPS/Excel 兼容性问题**
  - 修正 BrtWsFmtInfo RecordType 值（229 → 485）
  - 修正单元格结构：iStyleRef（3 字节）+ flags（1 字节）
  - 修正 BrtCellRk 整数编码优化
  - 修正空白单元格处理：跳过空白单元格（与 jxlsb 行为一致）

- **BIFF12 格式修正**：
  - BrtWsFmtInfo 记录类型正确设为 485
  - 单元格记录使用正确结构：column(4) + iStyleRef(3) + flags(1)
  - 整数在 [-536870912, 536870911] 范围内使用 RK 编码
  - 空白单元格不再生成 BrtCellBlank 记录

### 性能

- **文件体积优化**：跳过空白单元格，减小文件体积
- **整数优化**：使用 BrtCellRk（12 字节）替代 BrtCellReal（16 字节）
- **无性能回退**：所有操作保持高吞吐

### 测试

- **WPS 验证**：所有生成文件在 WPS Office 中验证通过
  - minimal_one.xlsb (1×1) ✓
  - minimal.xlsb (10×4) ✓
  - test_stream_api.xlsb (100×4) ✓
  - verify_basic.xlsb (50×5) ✓
  - verify_formats.xlsb (100×8) ✓
  - verify_streaming.xlsb (200×4) ✓
  - verify_large.xlsb (10,000×4) ✓
  - stream_single.xlsb (100×4) ✓
  - stream_multi.xlsb (3 个工作表) ✓
  - stream_batch.xlsb (300 行) ✓
  - stream_large.xlsb (5000×4) ✓

- **API 一致性验证**：
  - stream_api_test.xlsb == batch_api_test.xlsb（十六进制完全相同）
  - 流式 API 生成与批量 API 同等质量的文件

### 技术细节

- **BrtWsFmtInfo 修正**：
  - 旧：RecordType = 229（错误）
  - 新：RecordType = 485（符合 MS-XLSB 规范 2.4.823）

- **单元格结构修正**：
  - 旧：column(4) + style(4) = 8 字节（错误）
  - 新：column(4) + iStyleRef(3) + flags(1) = 8 字节（正确）

- **RK 编码支持**：
  - 范围：[-536870912, 536870911]（±2^29）
  - 编码：IEEE 754 double 高 32 位
  - 记录：BrtCellRk（12 字节） vs BrtCellReal（16 字节）

- **空白单元格优化**：
  - jxlsb 行为：跳过空白单元格，不生成记录
  - rxlsb 现在一致：不写入空白单元格
  - 收益：文件体积减小、WPS 兼容

### 迁移指南（0.2.0 → 0.3.0）

**无破坏性变更**：所有 0.2.0 API 仍然可用

**新增测试示例**：
```bash
cargo run --example minimal_one          # 最小测试
cargo run --example test_stream_api      # 流式 API 测试
cargo run --example gen_verify           # 完整验证
cargo run --example streaming_full_test
```

**已验证文件**：
- 所有生成文件现可在 WPS/Excel 中正确打开
- 格式显示验证：百分比、货币、日期、时间
- 大文件测试：5,000 与 10,000 行

### 文档

- 更新 README 的 WPS 兼容性章节
- 更新 README 的验证细节
- 新增测试示例文档

## [0.2.0] - 2026-05-01

### 新增

- **数字格式支持**：单元格数字格式完整实现
  - 百分比（0.00%）
  - 货币（￥#,##0.00）
  - 千分位（#,##0.00）
  - 负数标红（#,##0.00;[Red]-#,##0.00）
  - 日期格式（m/d/yy h:mm）
  - 时间格式（h:mm:ss）
  - 自定义格式支持（ifmt 164+）

- **CellData API 扩展**：
  - `CellData::percentage(value)` — 百分比单元格
  - `CellData::percentage_with_decimals(value, decimals)` — 自定义小数百分比
  - `CellData::number_with_comma(value)` — 千分位
  - `CellData::number_negative_red(value)` — 负数标红
  - `CellData::currency(value)` — 货币格式（￥ 符号）
  - `CellData::currency_with_symbol(value, symbol)` — 自定义货币符号
  - `CellData::number_with_format(value, format)` — 自定义数字格式
  - `CellData::date_from_timestamp(timestamp)` — 由 Unix 时间戳生成日期
  - `CellData::date_with_format(timestamp, format)` — 自定义日期格式
  - `CellData::time(timestamp)` — 时间格式
  - `CellData::time_with_format(timestamp, format)` — 自定义时间格式

- **NumberFormatRegistry**：管理内置（0-22）与自定义（164+）格式 ID
  - 自动识别内置格式
  - 顺序分配自定义格式 ID
  - 格式字符串去重

- **StylesRegistry**：动态样式 XF 管理
  - 自动注册样式
  - 格式复用优化
  - 正确的 styleIndex 映射机制

- **文档**：
  - `docs/style_index_mapping.md`：完整的 styleIndex 映射机制文档
  - 内置格式 ID 参考（0-22）
  - 自定义格式 ID 分配策略

- **工具**：
  - `tools/verify_style_index.py`：验证 XLSB 文件中 styleIndex 映射的 Python 脚本

### 修复

- **严重：styleIndex 映射机制**
  - 发现 Excel/WPS 自动 +1 映射：styles[i] → XF[i+1]
  - `get_style_id_for_format` 返回 styles 列表索引（而非 i+1）
  - 使用 format_test.xlsb 与 format_test2.xlsb 验证
  - 所有格式列在 WPS 中正确显示

- **日期格式修正**：
  - `date_from_timestamp` 使用 "m/d/yy h:mm"（ifmt=22）而非 "mm-dd-yy"（ifmt=14）
  - 与 jxlsb 默认日期格式一致

- **NumberFormatRegistry 内置格式识别**：
  - "m/d/yy h:mm" → ifmt=22（内置，非 164）
  - "h:mm:ss" → ifmt=21（内置，非 164）
  - 避免不必要的自定义格式分配

### 性能

- 无性能回退
- 格式注册：O(n) 查找，O(1) 追加
- 样式复用：格式去重减少 XF 数量

### 测试

- 新增 format_test.rs 示例（对应 jxlsb NumberFormatTest）
- 新增 format_test2.rs 示例（验证泛化）
- 所有格式在 WPS 中验证：
  - 百分比、货币、千分位 ✓
  - 负数标红、日期、时间 ✓
  - 不同列顺序、不同格式组合 ✓

### 技术细节

- **XF 结构**：
  - XF[0]：BrtBeginXFs（单元格 XF，ixf=0xffff）
  - XF[1+]：BrtBeginStyles（样式 XF，ixf=0x0000）
  - styles[i] → BrtBeginStyles[i] → XF[i+1]

- **styleIndex 映射**（关键发现）：
  - cell.styleIndex = styles 列表索引（从 0 开始）
  - Excel/WPS 自动 +1 → 全局 XF 索引
  - 实现：返回 i，而非 i+1

- **格式 ID 分配**：
  - 内置格式：0-22（Excel 预定义）
  - 自定义格式：164+（顺序分配）
  - 内置格式字符串返回内置 ID，而非自定义 ID

### 已知问题

- 暂不支持公式单元格（计划于 0.3.0）
- 暂不支持富文本单元格
- 暂不支持图表
- 暂不支持字体、填充、边框自定义（仅基础样式）

### 迁移指南（0.1.0 → 0.2.0）

**新增 API**：
```rust
// 数字格式
CellData::percentage(0.123)              // 12.30%
CellData::number_with_comma(-1234.56)    // -1,234.56
CellData::number_negative_red(-500.0)    // -500.00（红）
CellData::currency(1234.56)              // ￥1,234.56

// 日期/时间格式
CellData::date_from_timestamp(1714560000)  // 5/1/2026 10:40
CellData::time(1714560000)                  // 10:40:00
```

**无破坏性变更**：所有 0.1.0 API 仍然可用

## [0.1.0] - 2026-04-30

### 新增

- rxlsb 首次发布
- XLSB 文件核心读写功能
- `XlsbReader`：流式与分页读取模式
- `XlsbWriter`：批量与流式写入模式
- `TemplateFiller`：基于模板的数据填充
- `CellData` 类型：Text、Number、Date、Bool、Blank、Error
- 零拷贝 Bytes 架构
- Deflated 压缩的 ZIP 容器
- BIFF12 格式实现
- SST（共享字符串表）支持
- 性能基准（1M 行测试）

### 性能

- 流式写：201K 行/秒（相较 jxlsb +46%）
- 批量写：190K 行/秒（+71%）
- 流式读：2.3M 行/秒（+18%）
- 分页读：31K 行/秒（+210%）

### 修复

- BrtCellSt 内联字符串格式：VarInt → u32_le
- ZIP 压缩：Stored → Deflated（文件从 220MB 降至 31MB）
- 分页读取的工作表缓存（10 倍性能提升）
- 消除 Cells clone（零拷贝优化）

### 文档

- 带性能对比的模块级文档
- CellData、XlsbReader、XlsbWriter 的 API 文档
- 文档中的代码示例
- 含快速上手指南的 README

### 测试

- 9 个测试通过（features_test：6，template_test：3）
- 0 编译告警
- 所有 dead_code 告警通过 #[allow(dead_code)] 解决

### 已知问题

- 暂不支持公式单元格
- 暂不支持富文本单元格
- 暂不支持图表

## 未来计划

### [0.4.0] - 计划中

- 公式单元格支持
- 富文本单元格支持
- 高级单元格样式（字体、颜色、对齐、填充、边框）
- 图表支持
- 数据透视表支持
- 自动筛选支持
- 条件格式

## 版本历史汇总

| 版本 | 日期 | 关键特性 |
|------|------|----------|
| 0.1.0 | 2026-04-30 | 首次发布，核心读写，模板填充 |
| 0.2.0 | 2026-05-01 | 数字格式（百分比、货币、日期、时间），styleIndex 映射 |
| 0.3.0 | 2026-05-07 | WPS/Excel 100% 兼容，BIFF12 格式修正，流式 API 完成 |
| 0.4.0 | 待定 | 公式、富文本、高级样式、图表、数据透视表 |

## 相关文档

- [README.md](README.md)（中文） / [README_en.md](README_en.md)（英文）
- [MANUAL.md](MANUAL.md)（中文） / [MANUAL_en.md](MANUAL_en.md)（英文）
