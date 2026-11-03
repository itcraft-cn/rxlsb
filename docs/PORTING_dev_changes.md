# dev 相对 origin/dev 的变更抽取（供其他 XLSB 实现同步）

范围：`git diff origin/dev..dev`（12 个提交）。
目的：把 **必须同步的格式/行为修正** 与 **可移植的性能优化** 分离，便于 cxlsb(C) / jxlsb(Java) / jsxlsb(Node) 对齐。

---

## 一、必须同步：BIFF12 写入格式修正（WPS/Excel 兼容）

### 1. BrtWsFmtInfo 记录类型
- 正确值：`485`（varint 编码为 `E5 03`）
- 旧错误值：`229`（`E5 01`）——会导致 WPS 只显示第一行

### 2. 单元格记录布局（所有 cell 记录通用）
```
column        u32   (4)
iStyleRef     u24   (3)   // 低 24 位样式索引
flags         u8    (1)   // 固定 0
payload...                // 按类型
```
共 8 字节头。**旧实现常见错误**：把 style 写成 4 字节（缺少 flags），会导致 WPS 只显示部分行。

### 3. 数值单元格：RK / Real 二选一
- 若 `n` 为整数且 `-536870912 <= n <= 536870911`（±2^29）：
  - `BrtCellRk`(type=2)，size=12，payload = col(4)+style(4)+RK(4)
  - **RK 采用标准整数形式**：`((n as i32 as u32) << 2) | 0x02`
    - `bit1 = fInt = 1`，`bit0 = fX100 = 0`，高 30 位为有符号整数
  - 解码：`fInt` 置位时取 `(rk as i32) >> 2`；否则 `f64::from_bits((rk & 0xFFFFFFFC) as u64 << 32)`
  - **注意**：不要直接把 f64 高 32 位当作 RK。f64 高字的低 2 位对应尾数中段，
    在大整数（如 `536870911`）上可能非 0，会被误判为 fInt/fX100，导致读回错误
- 否则 `BrtCellReal`(type=5)，size=16，payload = col(4)+style(4)+f64(8)

### 4. 空白单元格：跳过
- **不要**为 Blank 写 `BrtCellBlank`，直接不生成记录（与 jxlsb 一致）
- 否则文件膨胀且 WPS 可能解析异常（历史 bug）

### 5. 记录类型常量（本实现使用）
| 名称 | 值 |
|---|---|
| BrtRowHdr | 0 |
| BrtCellBlank | 1 |
| BrtCellRk | 2 |
| BrtCellError | 3 |
| BrtCellBool | 4 |
| BrtCellReal | 5 |
| BrtCellSt | 6 |
| BrtCellIsst | 7 |
| BrtWsFmtInfo | **485** |
| BrtBeginSheet / EndSheet | 129 / 130 |
| BrtBeginSheetData / EndSheetData | 145 / 146 |
| BrtWsProp / BrtWsDim | 147 / 148 |
| BrtBeginSst / EndSst / SstItem | 159 / 160 / 19 |

### 6. 行头（BrtRowHdr，type=0）
```
rw        u32   // 行号
ixfe      u32   // 0
miyRw     u16   // 270
flags     3 x u8 // 0
ccolspan  u32   // span 段数
spans: (colMic u32, colLast u32) * ccolspan
```
- 每 1024 列一个 span：`num_spans = (col_count-1)/1024 + 1`
- 常用快路径（≤1024 列）：record size=25，固定 27 字节整块写出

### 7. 维度（BrtWsDim，type=148）
`first_row, last_row, first_col, last_col`（last 均为 count-1）。
- 流式写入：**前导先写**，`last_row` 处占位；结束时回填。
  - 回填偏移 = `dim_record_offset + 7`（type varint 2 + varsize 1 + first_row 4）
  - 这样避免结束阶段整块拷贝行数据

### 8. varint / varsize
- varint（记录类型）：`<128` 单字节；否则 `(v&0x7F)|0x80, (v>>7)&0x7F`
- varsize（记录长度）：标准 base-128 小端变长

---

## 二、必须同步：读取修正

### 1. BrtCellBool 读取
- 布局：col(4) + iStyleRef(3) + flags(1) + value(1) = 9
- 读取时必须先 **skip 4**（style3+flags1）再读 value；旧代码 `skip(3)` 会错位

### 2. SST 反序列化（性能 + 正确性）
- `BrtSstItem`：flags(1) + cch(u32) + UTF-16LE(cch*2)
- **批量解码**：一次边界检查后直接在字节切片上按 u16 顺序填充，再 `from_utf16`
- 不要逐 u16 调用带边界检查的 read（实测占读取 24%）
- 注意：使用 `Bytes::slice + iterator.collect` 的写法在 release 反而更慢，应直接切片手动填充
- 闭环 `decode_utf16().collect::<Result<String,_>>()` 也明显更慢，勿用

---

## 三、需要对齐的新增能力（API/行为）

### 1. 压缩级别可配置
- 写入器支持 `compression_level`（0-9），默认 **1（速度优先）**
- C 后端（zlib-ng）作为**可选 feature**，默认纯 Rust
- ZIP 容器使用标准库（support zip64），不要自研

### 2. 行列上限溢出策略（全局、底层统一）
- 常量：`MAX_COLUMNS = 16384`，`MAX_ROWS = 1048576`
- 行为：`OverflowBehavior = { Error, Cut }`
  - Error：返回 `LimitExceeded { dimension, requested, limit }`
  - Cut：裁剪到上限
- 配置是**全局**的，可随时修改；**每次写入操作开始时快照**，只影响之后的操作
- 列/行独立配置：`set_column_overflow` / `set_row_overflow`
- 组合配置：`set_overflow(columns, rows)`；按维度“后设置覆盖先设置”
- 内部用 **trait 策略模式**：`OverflowPolicy { columns(), rows() }`，`ErrorPolicy` / `CutPolicy`
- 底层 `SheetLimits`（一次操作内快照），统一覆盖：
  - 批量写、流式写、**模板填充**（维度裁剪 + 超限单元格/填充列跳过）

### 3. 模板填充
- 目标维度先经 `SheetLimits`：`max_col = columns-1`、`max_row = rows(0, requested)-1`
- 模板单元格 `col > max_col` 跳过；填充列循环以 `max_col` 截断
- 修正示例坐标：模板数据应从 **I13（row 12, col 8）** 开始，勿写成 A 列

---

## 四、可移植的性能优化（不改格式）

### 写入
1. **SST 存储**：`Vec<String>` 移动存储 + `HashMap<u64,u32>`，键为 FNV-1a 哈希；用**恒等哈希器**（键已是哈希，免二次哈希）
2. **UTF-8→UTF-16**：
   - 序列化时单次编码；小端平台直接 memcpy `[u16]` 字节
   - 编码用 **u64 批量 ASCII 快路径**（每次判 8 字节），非 ASCII 才走字符解码
3. **SST 分块序列化**：累积 64 KiB 即写入压缩流，避免整块中间缓冲
4. **单元格定长编码**：把 type/size/col/style/payload 拼进栈上数组，一次 `write`（RK 14B、Real 18B、Isst 14B、Bool 11B、Blank 10B）
5. **行头快路径**：≤1024 列固定 27 字节整块
6. **流式**：增量编码行记录，不缓存 `Vec<Vec<Cell>>`；前导前置 + 维度回填（见一.7）
7. **缓冲**：sheet 序列化“移动”而非拷贝；ZIP 输出加 256 KiB BufWriter
8. 预留容量用 `saturating_mul`，避免异常大请求溢出/过度预留

### 读取
1. `BufferReader::read_utf16_le_string(cch)`：一次边界检查 + 直接切片顺序填充
2. SST 解析不再逐 u16 读取（见二.2）

### 不要做（实测负收益）
- 自研 ZIP（回退，改用标准 zip 库）
- `decode_utf16().collect()` 直接转 String（更慢）
- 读取处 `Bytes::slice + iterator.collect`（更慢）
- 预先 `reserve` 大 Vec / 按维度提示预分配（实测无收益或更慢）

---

## 五、逐文件映射

| Rust 文件 | 其他实现对应关注点 |
|---|---|
| `src/format/sheet_writer.rs` | 记录类型、cell 8 字节头、RK/Real、跳 Blank、行头 span、prologue/epilogue、定长编码 |
| `src/format/sheet_reader.rs` | Bool skip4；记录循环 |
| `src/format/sst_table.rs` | 移动存储 + FNV 索引；UTF-16 编解码；分块序列化 |
| `src/io/buffer_reader.rs` | `read_utf16_le_string` |
| `src/io/buffer_writer.rs` | `write_u16_le_slice`（小端 memcpy）、`set_u32_le_at`（维度回填） |
| `src/container/zip_writer.rs` | 压缩级别、BufWriter、begin/chunk/end 流式条目 |
| `src/api/writer.rs` | 流式状态、SheetLimits、无整块拷贝 |
| `src/api/template.rs` | 模板维度裁剪、列上限过滤 |
| `src/limits.rs` | 全局配置 + trait 策略 + SheetLimits |

---

## 六、验证清单（对齐后必测）

1. WPS 打开：1 行/50 行/100 行/1 万行；确认不是“只显示一行/部分行”
2. 8 种数字格式：百分比、千分位、负红、货币、日期、时间
3. 多 sheet 切换；流式分批（0-99/100-199/200-299）连续无断点
4. 宽列：2000 / 12000 / 16384 列；19999 列在 Error/Cut 下的行为
5. 模板：坐标、marker、流式三种；数据从 I13 开始、标题/表头保留
6. ZIP 用独立解析器（如 Python zipfile）校验；读取回环行/列数一致
