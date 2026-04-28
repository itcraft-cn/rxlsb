# rxlsb 格式层详细设计

## 一、RecordTypes enum

### 1.1 BIFF12记录类型

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
    BrtFmlaBool = 8,
    BrtFmlaError = 9,
    BrtFmlaNum = 10,
    BrtFmlaString = 11,
    
    BrtSstItem = 19,
    
    BrtBeginSheet = 129,
    BrtEndSheet = 130,
    BrtBeginBook = 131,
    BrtEndBook = 132,
    BrtWsProp = 133,
    BrtWsDim = 148,
    
    BrtBeginSheetData = 145,
    BrtEndSheetData = 146,
    
    BrtBeginSst = 159,
    BrtEndSst = 160,
    
    BrtMergeCell = 176,
    
    BrtBeginStyleSheet = 370,
    BrtEndStyleSheet = 371,
    
    BrtBeginStyleSheetExt14 = 372,
    BrtEndStyleSheetExt14 = 373,
}
```

### 1.2 解析方法

```rust
impl RecordType {
    pub fn from_u32(code: u32) -> Option<Self> {
        match code {
            0 => Some(RecordType::BrtRowHdr),
            1 => Some(RecordType::BrtCellBlank),
            2 => Some(RecordType::BrtCellRk),
            3 => Some(RecordType::BrtCellError),
            4 => Some(RecordType::BrtCellBool),
            5 => Some(RecordType::BrtCellReal),
            6 => Some(RecordType::BrtCellSt),
            7 => Some(RecordType::BrtCellIsst),
            129 => Some(RecordType::BrtBeginSheet),
            130 => Some(RecordType::BrtEndSheet),
            131 => Some(RecordType::BrtBeginBook),
            132 => Some(RecordType::BrtEndBook),
            145 => Some(RecordType::BrtBeginSheetData),
            146 => Some(RecordType::BrtEndSheetData),
            148 => Some(RecordType::BrtWsDim),
            159 => Some(RecordType::BrtBeginSst),
            160 => Some(RecordType::BrtEndSst),
            176 => Some(RecordType::BrtMergeCell),
            370 => Some(RecordType::BrtBeginStyleSheet),
            371 => Some(RecordType::BrtEndStyleSheet),
            _ => None,
        }
    }
    
    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}
```

---

## 二、Biff12Writer trait

### 2.1 trait定义

```rust
use bytes::{BytesMut, Bytes};

pub trait Biff12Writer {
    fn buffer(&mut self) -> &mut BytesMut;
    
    fn write_record_header(&mut self, record_type: RecordType, size: u32) {
        self.buffer().put_u32_le(record_type.to_u32());
        self.buffer().put_u32_le(size);
    }
    
    fn write_empty_record(&mut self, record_type: RecordType) {
        self.write_record_header(record_type, 0);
    }
    
    fn write_varint(&mut self, value: u32) -> usize;
    fn write_varsize(&mut self, value: u32) -> usize;
    fn write_wide_string(&mut self, s: &str) -> usize;
}
```

### 2.2 单元格写入方法

```rust
pub trait Biff12WriterExt: Biff12Writer {
    fn write_cell_real(&mut self, row: u32, col: u32, value: f64, xf_id: u32) {
        let size = 16;  // row(4) + col(4) + xf_id(4) + value(8) = 20, but RK format different
        self.write_record_header(RecordType::BrtCellReal, 16);
        self.buffer().put_u32_le(row);
        self.buffer().put_u32_le(col);
        self.buffer().put_u32_le(xf_id);
        self.buffer().put_f64_le(value);
    }
    
    fn write_cell_rk(&mut self, row: u32, col: u32, value: f64, xf_id: u32) {
        let rk_value = encode_rk(value);
        self.write_record_header(RecordType::BrtCellRk, 16);
        self.buffer().put_u32_le(row);
        self.buffer().put_u32_le(col);
        self.buffer().put_u32_le(xf_id);
        self.buffer().put_u32_le(rk_value);
    }
    
    fn write_cell_isst(&mut self, row: u32, col: u32, sst_idx: u32, xf_id: u32) {
        self.write_record_header(RecordType::BrtCellIsst, 12);
        self.buffer().put_u32_le(row);
        self.buffer().put_u32_le(col);
        self.buffer().put_u32_le(xf_id);
        self.write_varint(sst_idx);
    }
    
    fn write_cell_st(&mut self, row: u32, col: u32, s: &str, xf_id: u32) {
        let header_size = 12;  // row + col + xf_id
        let string_size = utf16le_length(s) * 2 + 2;  // varsize + UTF-16LE bytes
        self.write_record_header(RecordType::BrtCellSt, header_size + string_size as u32);
        self.buffer().put_u32_le(row);
        self.buffer().put_u32_le(col);
        self.buffer().put_u32_le(xf_id);
        self.write_wide_string(s);
    }
    
    fn write_cell_bool(&mut self, row: u32, col: u32, value: bool, xf_id: u32) {
        self.write_record_header(RecordType::BrtCellBool, 12);
        self.buffer().put_u32_le(row);
        self.buffer().put_u32_le(col);
        self.buffer().put_u32_le(xf_id);
        self.buffer().put_u8(value as u8);
    }
    
    fn write_cell_blank(&mut self, row: u32, col: u32, xf_id: u32) {
        self.write_record_header(RecordType::BrtCellBlank, 12);
        self.buffer().put_u32_le(row);
        self.buffer().put_u32_le(col);
        self.buffer().put_u32_le(xf_id);
    }
    
    fn write_row_header(&mut self, row: u32, first_col: u32, last_col: u32) {
        self.write_record_header(RecordType::BrtRowHdr, 8);
        self.write_varint(row);
        self.write_varint(first_col);
        self.write_varint(last_col - first_col + 1);
    }
}
```

### 2.3 RK编码

```rust
fn encode_rk(value: f64) -> u32 {
    if value == 0.0 {
        return 0;
    }
    
    let int_value = value as i32;
    if int_value as f64 == value && int_value >= -262144 && int_value <= 262143 {
        let rk = ((int_value << 2) | 2) as u32;
        return rk;
    }
    
    let multiplied = value * 100.0;
    let int_multiplied = multiplied as i32;
    if int_multiplied as f64 == multiplied && int_multiplied >= -262144 && int_multiplied <= 262143 {
        let rk = ((int_multiplied << 2) | 3) as u32;
        return rk;
    }
    
    0  // 无法编码，使用 BrtCellReal
}

fn decode_rk(rk_value: u32) -> f64 {
    let flags = rk_value & 3;
    let value = rk_value >> 2;
    
    match flags {
        0 | 1 => {
            let ieee_bytes = value.to_le_bytes();
            f64::from_le_bytes(ieee_bytes)
        }
        2 => value as f64,
        3 => value as f64 / 100.0,
        _ => 0.0,
    }
}
```

---

## 三、Biff12Reader trait

### 3.1 trait定义

```rust
use bytes::Bytes;

pub trait Biff12Reader {
    fn buffer(&self) -> &Bytes;
    fn position(&mut self) -> &mut usize;
    
    fn read_record_header(&mut self) -> Result<(RecordType, u32)> {
        let pos = *self.position();
        if pos + 8 > self.buffer().len() {
            return Err(XlsbError::BufferOverflow { position: pos, length: self.buffer().len() });
        }
        
        let type_code = u32::from_le_bytes([
            self.buffer()[pos],
            self.buffer()[pos + 1],
            self.buffer()[pos + 2],
            self.buffer()[pos + 3],
        ]);
        
        let size = u32::from_le_bytes([
            self.buffer()[pos + 4],
            self.buffer()[pos + 5],
            self.buffer()[pos + 6],
            self.buffer()[pos + 7],
        ]);
        
        *self.position() += 8;
        
        let record_type = RecordType::from_u32(type_code)
            .ok_or(XlsbError::InvalidRecordType(type_code))?;
        
        Ok((record_type, size))
    }
    
    fn read_varint(&mut self) -> Result<u32>;
    fn read_wide_string(&mut self) -> Result<String>;
}
```

### 3.2 单元格解析方法

```rust
pub trait Biff12ReaderExt: Biff12Reader {
    fn parse_cell(&mut self, record_type: RecordType) -> Result<CellRecord> {
        match record_type {
            RecordType::BrtCellBlank => self.parse_cell_blank(),
            RecordType::BrtCellReal => self.parse_cell_real(),
            RecordType::BrtCellRk => self.parse_cell_rk(),
            RecordType::BrtCellIsst => self.parse_cell_isst(),
            RecordType::BrtCellSt => self.parse_cell_st(),
            RecordType::BrtCellBool => self.parse_cell_bool(),
            RecordType::BrtCellError => self.parse_cell_error(),
            _ => Err(XlsbError::InvalidRecordType(record_type.to_u32())),
        }
    }
    
    fn parse_cell_blank(&mut self) -> Result<CellRecord> {
        let row = self.read_u32_le()?;
        let col = self.read_u32_le()?;
        let xf_id = self.read_u32_le()?;
        Ok(CellRecord::Blank { row, col, xf_id })
    }
    
    fn parse_cell_real(&mut self) -> Result<CellRecord> {
        let row = self.read_u32_le()?;
        let col = self.read_u32_le()?;
        let xf_id = self.read_u32_le()?;
        let value = self.read_f64_le()?;
        Ok(CellRecord::Real { row, col, value, xf_id })
    }
    
    fn parse_cell_rk(&mut self) -> Result<CellRecord> {
        let row = self.read_u32_le()?;
        let col = self.read_u32_le()?;
        let xf_id = self.read_u32_le()?;
        let rk_value = self.read_u32_le()?;
        let value = decode_rk(rk_value);
        Ok(CellRecord::Rk { row, col, value, xf_id })
    }
    
    fn parse_cell_isst(&mut self) -> Result<CellRecord> {
        let row = self.read_u32_le()?;
        let col = self.read_u32_le()?;
        let xf_id = self.read_u32_le()?;
        let sst_idx = self.read_varint()?;
        Ok(CellRecord::Isst { row, col, sst_idx, xf_id })
    }
    
    fn parse_cell_st(&mut self) -> Result<CellRecord> {
        let row = self.read_u32_le()?;
        let col = self.read_u32_le()?;
        let xf_id = self.read_u32_le()?;
        let text = self.read_wide_string()?;
        Ok(CellRecord::St { row, col, text, xf_id })
    }
    
    fn parse_cell_bool(&mut self) -> Result<CellRecord> {
        let row = self.read_u32_le()?;
        let col = self.read_u32_le()?;
        let xf_id = self.read_u32_le()?;
        let value = self.read_u8()? != 0;
        Ok(CellRecord::Bool { row, col, value, xf_id })
    }
}
```

---

## 四、CellRecord enum

### 4.1 定义

```rust
pub enum CellRecord {
    Blank { row: u32, col: u32, xf_id: u32 },
    Real { row: u32, col: u32, value: f64, xf_id: u32 },
    Rk { row: u32, col: u32, value: f64, xf_id: u32 },
    Isst { row: u32, col: u32, sst_idx: u32, xf_id: u32 },
    St { row: u32, col: u32, text: String, xf_id: u32 },
    Bool { row: u32, col: u32, value: bool, xf_id: u32 },
    Error { row: u32, col: u32, error: CellError, xf_id: u32 },
}
```

### 4.2 转换为CellData

```rust
impl CellRecord {
    pub fn to_cell_data(&self, sst: Option<&SstTable>) -> CellData {
        match self {
            CellRecord::Blank { .. } => CellData::blank(),
            CellRecord::Real { value, .. } => CellData::number(*value),
            CellRecord::Rk { value, .. } => CellData::number(*value),
            CellRecord::Isst { sst_idx, .. } => {
                sst.and_then(|s| s.get_string(*sst_idx))
                    .map(CellData::text)
                    .unwrap_or(CellData::blank())
            }
            CellRecord::St { text, .. } => CellData::text(text.clone()),
            CellRecord::Bool { value, .. } => CellData::bool(*value),
            CellRecord::Error { error, .. } => CellData::error(*error),
        }
    }
    
    pub fn row(&self) -> u32 {
        match self {
            CellRecord::Blank { row, .. } => *row,
            CellRecord::Real { row, .. } => *row,
            CellRecord::Rk { row, .. } => *row,
            CellRecord::Isst { row, .. } => *row,
            CellRecord::St { row, .. } => *row,
            CellRecord::Bool { row, .. } => *row,
            CellRecord::Error { row, .. } => *row,
        }
    }
    
    pub fn col(&self) -> u32 {
        match self {
            CellRecord::Blank { col, .. } => *col,
            CellRecord::Real { col, .. } => *col,
            CellRecord::Rk { col, .. } => *col,
            CellRecord::Isst { col, .. } => *col,
            CellRecord::St { col, .. } => *col,
            CellRecord::Bool { col, .. } => *col,
            CellRecord::Error { col, .. } => *col,
        }
    }
}
```

---

## 五、SstTable

### 5.1 结构定义

```rust
use std::collections::HashMap;
use bytes::BytesMut;

pub struct SstTable {
    strings: Vec<String>,
    hash_map: HashMap<String, u32>,
    total_count: u32,
}

impl SstTable {
    pub fn new() -> Self {
        Self {
            strings: vec![],
            hash_map: HashMap::new(),
            total_count: 0,
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            strings: Vec::with_capacity(capacity),
            hash_map: HashMap::with_capacity(capacity),
            total_count: 0,
        }
    }
}
```

### 5.2 添加字符串

```rust
impl SstTable {
    pub fn add_string(&mut self, s: &str) -> u32 {
        self.total_count += 1;
        
        if let Some(idx) = self.hash_map.get(s) {
            return *idx;
        }
        
        let idx = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.hash_map.insert(s.to_string(), idx);
        idx
    }
    
    pub fn get_string(&self, idx: u32) -> Option<&str> {
        self.strings.get(idx as usize).map(|s| s.as_str())
    }
    
    pub fn find_string(&self, s: &str) -> Option<u32> {
        self.hash_map.get(s).copied()
    }
    
    pub fn count(&self) -> usize {
        self.strings.len()
    }
    
    pub fn total_count(&self) -> u32 {
        self.total_count
    }
}
```

### 5.3 序列化

```rust
impl SstTable {
    pub fn serialize(&self) -> Result<Bytes> {
        let mut writer = BufferWriter::new(1024);
        
        writer.write_empty_record(RecordType::BrtBeginSst);
        
        for s in &self.strings {
            writer.write_record_header(RecordType::BrtSstItem, utf16le_length(s) * 2 + 2);
            writer.write_wide_string(s);
        }
        
        writer.write_empty_record(RecordType::BrtEndSst);
        
        Ok(writer.freeze())
    }
}
```

### 5.4 反序列化

```rust
impl SstTable {
    pub fn deserialize(data: &Bytes) -> Result<Self> {
        let mut reader = BufferReader::new(data.clone());
        
        let (record_type, _) = reader.read_record_header()?;
        if record_type != RecordType::BrtBeginSst {
            return Err(XlsbError::InvalidRecordType(record_type.to_u32()));
        }
        
        let mut table = SstTable::new();
        
        while reader.has_remaining() {
            let (record_type, size) = reader.read_record_header()?;
            
            if record_type == RecordType::BrtEndSst {
                break;
            }
            
            if record_type == RecordType::BrtSstItem {
                let text = reader.read_wide_string()?;
                table.add_string(&text);
            }
        }
        
        Ok(table)
    }
}
```

---

## 六、StylesRegistry

### 6.1 结构定义

```rust
pub struct Font {
    name: String,
    size: f64,
    bold: bool,
    italic: bool,
    underline: bool,
    color: u32,
}

pub struct Fill {
    pattern_type: PatternType,
    fg_color: u32,
    bg_color: u32,
}

pub enum PatternType {
    None = 0,
    Solid = 1,
    MediumGray = 2,
    DarkGray = 3,
    LightGray = 4,
    DarkHorizontal = 5,
    DarkVertical = 6,
}

pub struct Border {
    left: BorderSide,
    right: BorderSide,
    top: BorderSide,
    bottom: BorderSide,
}

pub struct BorderSide {
    style: BorderStyle,
    color: u32,
}

pub enum BorderStyle {
    None = 0,
    Thin = 1,
    Medium = 2,
    Thick = 3,
}

pub struct Xf {
    font_id: u32,
    fill_id: u32,
    border_id: u32,
    num_format_id: u32,
    horizontal_align: HorizontalAlign,
    vertical_align: VerticalAlign,
    wrap_text: bool,
}

pub enum HorizontalAlign {
    General = 0,
    Left = 1,
    Center = 2,
    Right = 3,
}

pub enum VerticalAlign {
    Top = 0,
    Center = 1,
    Bottom = 2,
}

pub struct StylesRegistry {
    fonts: Vec<Font>,
    fills: Vec<Fill>,
    borders: Vec<Border>,
    xfs: Vec<Xf>,
    num_formats: Vec<String>,
    default_xf_id: u32,
}

impl StylesRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            fonts: vec![],
            fills: vec![],
            borders: vec![],
            xfs: vec![],
            num_formats: vec![],
            default_xf_id: 0,
        };
        
        registry.add_defaults();
        registry
    }
    
    fn add_defaults(&mut self) {
        self.fonts.push(Font::default());
        self.fills.push(Fill::none());
        self.borders.push(Border::none());
        self.xfs.push(Xf::default());
    }
}
```

### 6.2 添加样式

```rust
impl StylesRegistry {
    pub fn add_font(&mut self, font: Font) -> u32 {
        let idx = self.fonts.len() as u32;
        self.fonts.push(font);
        idx
    }
    
    pub fn add_fill(&mut self, fill: Fill) -> u32 {
        let idx = self.fills.len() as u32;
        self.fills.push(fill);
        idx
    }
    
    pub fn add_border(&mut self, border: Border) -> u32 {
        let idx = self.borders.len() as u32;
        self.borders.push(border);
        idx
    }
    
    pub fn add_xf(&mut self, xf: Xf) -> u32 {
        let idx = self.xfs.len() as u32;
        self.xfs.push(xf);
        idx
    }
    
    pub fn add_num_format(&mut self, format: &str) -> u32 {
        let idx = 164 + self.num_formats.len() as u32;
        self.num_formats.push(format.to_string());
        idx
    }
}
```

### 6.3 序列化

```rust
impl StylesRegistry {
    pub fn serialize(&self) -> Result<Bytes> {
        let mut writer = BufferWriter::new(1024);
        
        writer.write_empty_record(RecordType::BrtBeginStyleSheet);
        
        self.serialize_fonts(&mut writer);
        self.serialize_fills(&mut writer);
        self.serialize_borders(&mut writer);
        self.serialize_xfs(&mut writer);
        
        writer.write_empty_record(RecordType::BrtEndStyleSheet);
        
        Ok(writer.freeze())
    }
    
    fn serialize_fonts(&self, writer: &mut BufferWriter) {
        for font in &self.fonts {
            writer.write_record_header(RecordType::BrtFont, 24);
            writer.write_wide_string(&font.name);
            writer.put_f64_le(font.size);
            writer.put_u32_le(font.color);
            writer.put_u8(font.bold as u8);
            writer.put_u8(font.italic as u8);
            writer.put_u8(font.underline as u8);
        }
    }
}
```

---

## 七、SheetWriter

### 7.1 结构定义

```rust
pub struct SheetWriter<'a> {
    buffer: BytesMut,
    sst: &'a mut SstTable,
    styles: &'a mut StylesRegistry,
    col_count: usize,
    max_row: usize,
    max_col: usize,
    streaming: bool,
    rows_written: usize,
}
```

### 7.2 批量写入

```rust
impl SheetWriter {
    pub fn write_batch(&mut self, supplier: impl CellSupplier,
                       row_count: usize, col_count: usize) -> Result<()> {
        self.buffer = BytesMut::with_capacity(row_count * col_count * 20);
        
        self.write_empty_record(RecordType::BrtBeginSheet)?;
        self.write_empty_record(RecordType::BrtWsProp)?;
        self.write_dimension(0, 0, row_count - 1, col_count - 1)?;
        self.write_empty_record(RecordType::BrtBeginSheetData)?;
        
        for row in 0..row_count {
            let first_col = 0;
            let last_col = col_count - 1;
            self.write_row_header(row, first_col, last_col)?;
            
            for col in 0..col_count {
                let cell_data = supplier.get_cell(row, col);
                self.write_cell(row, col, cell_data)?;
            }
        }
        
        self.write_empty_record(RecordType::BrtEndSheetData)?;
        self.write_empty_record(RecordType::BrtEndSheet)?;
        
        self.max_row = row_count;
        self.max_col = col_count;
        Ok(())
    }
}
```

### 7.3 写入单元格

```rust
impl SheetWriter {
    fn write_cell(&mut self, row: u32, col: u32, data: CellData) -> Result<()> {
        let xf_id = self.get_xf_id(&data);
        
        match data {
            CellData::Text(s) => {
                if s.len() <= 3 {
                    self.write_cell_st(row, col, &s, xf_id);
                } else {
                    let sst_idx = self.sst.add_string(&s);
                    self.write_cell_isst(row, col, sst_idx, xf_id);
                }
            }
            CellData::Number(n) => {
                if can_encode_rk(n) {
                    self.write_cell_rk(row, col, n, xf_id);
                } else {
                    self.write_cell_real(row, col, n, xf_id);
                }
            }
            CellData::Bool(b) => {
                self.write_cell_bool(row, col, b, xf_id);
            }
            CellData::Blank => {
                self.write_cell_blank(row, col, xf_id);
            }
            CellData::Error(e) => {
                self.write_cell_error(row, col, e, xf_id);
            }
            CellData::Date(d) => {
                let excel_serial = excel_date_serial(&d);
                self.write_cell_real(row, col, excel_serial, xf_id);
            }
        }
        Ok(())
    }
    
    fn get_xf_id(&mut self, data: &CellData) -> u32 {
        match data {
            CellData::Number(n) => {
                if data.format_code().contains("%") {
                    self.styles.add_num_format("0.00%")
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}
```

---

## 八、SheetReader

### 8.1 结构定义

```rust
pub struct SheetReader {
    buffer: Bytes,
    sst: Option<&'a SstTable>,
    position: usize,
}
```

### 8.2 流式读取

```rust
impl SheetReader {
    pub fn for_each_row(&mut self, handler: impl RowHandler) -> Result<()> {
        let mut current_row: u32 = 0;
        let mut cells: Vec<CellData> = vec![];
        
        while self.has_remaining() {
            let (record_type, size) = self.read_record_header()?;
            
            match record_type {
                RecordType::BrtRowHdr => {
                    if !cells.is_empty() {
                        handler.on_row(current_row as usize, &cells);
                        cells.clear();
                    }
                    current_row = self.read_varint()?;
                }
                
                RecordType::BrtCellReal |
                RecordType::BrtCellRk |
                RecordType::BrtCellIsst |
                RecordType::BrtCellSt |
                RecordType::BrtCellBool |
                RecordType::BrtCellBlank |
                RecordType::BrtCellError => {
                    let cell_record = self.parse_cell(record_type)?;
                    let cell_data = cell_record.to_cell_data(self.sst);
                    cells.push(cell_data);
                }
                
                RecordType::BrtEndSheetData => {
                    if !cells.is_empty() {
                        handler.on_row(current_row as usize, &cells);
                    }
                    break;
                }
                
                _ => {
                    self.skip(size)?;
                }
            }
        }
        
        Ok(())
    }
}
```

---

## 九、WorkbookWriter

### 9.1 结构定义

```rust
pub struct WorkbookWriter {
    sheets: Vec<SheetEntry>,
}

struct SheetEntry {
    name: String,
    id: u32,
}
```

### 9.2 添加Sheet

```rust
impl WorkbookWriter {
    pub fn new() -> Self {
        Self { sheets: vec![] }
    }
    
    pub fn add_sheet(&mut self, name: &str) {
        let id = self.sheets.len() as u32 + 1;
        self.sheets.push(SheetEntry {
            name: name.to_string(),
            id,
        });
    }
}
```

### 9.3 序列化

```rust
impl WorkbookWriter {
    pub fn serialize(&self) -> Result<Bytes> {
        let mut writer = BufferWriter::new(256);
        
        writer.write_empty_record(RecordType::BrtBeginBook);
        
        for (i, sheet) in self.sheets.iter().enumerate() {
            writer.write_record_header(RecordType::BrtBundleSh, 12);
            writer.write_varint(i as u32);
            writer.write_varint(0);  // hidden state
            writer.write_varint(0);  // very hidden
            writer.write_wide_string(&sheet.name);
        }
        
        writer.write_empty_record(RecordType::BrtEndBook);
        
        Ok(writer.freeze())
    }
}
```