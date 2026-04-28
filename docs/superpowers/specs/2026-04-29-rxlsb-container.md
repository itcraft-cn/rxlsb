# rxlsb 容器层详细设计

## 一、XLSB ZIP容器结构

XLSB文件是ZIP容器，包含以下文件：

```
[Content_Types].xml      # 内容类型声明（XML）
_rels/.rels              # 根关系文件（XML）
docProps/app.xml         # 应用属性（XML）
docProps/core.xml        # 核心属性（XML）
xl/workbook.bin          # Workbook定义（BIFF12）
xl/_rels/workbook.bin.rels # Workbook关系（XML）
xl/styles.bin            # 样式定义（BIFF12）
xl/theme/theme1.xml      # 主题文件（XML）
xl/sharedStrings.bin     # 共享字符串表（BIFF12, optional）
xl/worksheets/sheet1.bin # Sheet数据（BIFF12）
xl/worksheets/sheet2.bin # Sheet数据（BIFF12）
...
```

---

## 二、XlsbContainerWriter

### 2.1 结构定义

```rust
use zip::{ZipWriter, CompressionMethod, write::FileOptions};
use std::fs::File;
use std::io::Write;
use bytes::Bytes;

pub struct XlsbContainerWriter {
    writer: ZipWriter<File>,
    entries: Vec<String>,
}

impl XlsbContainerWriter {
    pub fn create(path: &Path) -> Result<Self> {
        let file = File::create(path)?;
        let writer = ZipWriter::new(file);
        Ok(Self {
            writer,
            entries: vec![],
        })
    }
}
```

### 2.2 添加条目

```rust
impl XlsbContainerWriter {
    pub fn add_entry(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Stored);
        
        self.writer.start_file(name, options)?;
        self.writer.write_all(data)?;
        self.entries.push(name.to_string());
        Ok(())
    }
    
    pub fn add_entry_from_bytes(&mut self, name: &str, data: Bytes) -> Result<()> {
        self.add_entry(name, &data)
    }
    
    pub fn add_entry_from_str(&mut self, name: &str, data: &str) -> Result<()> {
        self.add_entry(name, data.as_bytes())
    }
}
```

### 2.3 完成

```rust
impl XlsbContainerWriter {
    pub fn finish(&mut self) -> Result<()> {
        self.writer.finish()?;
        Ok(())
    }
    
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
    
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}
```

---

## 三、XlsbContainerReader

### 3.1 结构定义

```rust
use zip::ZipArchive;
use bytes::BytesMut;

pub struct XlsbContainerReader {
    archive: ZipArchive<File>,
    path: PathBuf,
}

impl XlsbContainerReader {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let archive = ZipArchive::new(file)?;
        Ok(Self {
            archive,
            path: path.to_path_buf(),
        })
    }
}
```

### 3.2 检查条目

```rust
impl XlsbContainerReader {
    pub fn has_entry(&self, name: &str) -> bool {
        self.archive.by_name(name).is_ok()
    }
    
    pub fn entry_names(&self) -> Vec<String> {
        self.archive.file_names().collect()
    }
    
    pub fn entry_count(&self) -> usize {
        self.archive.len()
    }
}
```

### 3.3 读取条目

```rust
impl XlsbContainerReader {
    pub fn read_entry(&mut self, name: &str) -> Result<Bytes> {
        let mut file = self.archive.by_name(name)?;
        let size = file.size() as usize;
        
        let mut buffer = BytesMut::with_capacity(size);
        std::io::copy(&mut file, &mut buffer)?;
        
        Ok(buffer.freeze())
    }
    
    pub fn read_entry_to_vec(&mut self, name: &str) -> Result<Vec<u8>> {
        let mut file = self.archive.by_name(name)?;
        let size = file.size() as usize;
        
        let mut buffer = Vec::with_capacity(size);
        file.read_to_end(&mut buffer)?;
        
        Ok(buffer)
    }
}
```

### 3.4 专用读取方法

```rust
impl XlsbContainerReader {
    pub fn get_workbook_data(&mut self) -> Result<Bytes> {
        self.read_entry("xl/workbook.bin")
    }
    
    pub fn get_styles_data(&mut self) -> Result<Bytes> {
        self.read_entry("xl/styles.bin")
    }
    
    pub fn get_sst_data(&mut self) -> Result<Option<Bytes>> {
        if self.has_entry("xl/sharedStrings.bin") {
            self.read_entry("xl/sharedStrings.bin").map(Some)
        } else {
            Ok(None)
        }
    }
    
    pub fn get_sheet_data(&mut self, sheet_idx: usize) -> Result<Bytes> {
        let name = format!("xl/worksheets/sheet{}.bin", sheet_idx + 1);
        self.read_entry(&name)
    }
    
    pub fn get_theme_data(&mut self) -> Result<Bytes> {
        self.read_entry("xl/theme/theme1.xml")
    }
    
    pub fn get_app_data(&mut self) -> Result<Bytes> {
        self.read_entry("docProps/app.xml")
    }
    
    pub fn get_core_data(&mut self) -> Result<Bytes> {
        self.read_entry("docProps/core.xml")
    }
}
```

---

## 四、XmlGen

### 4.1 Content_Types.xml

```rust
pub struct XmlGen;

impl XmlGen {
    pub fn content_types(sheet_count: usize, has_sst: bool) -> Bytes {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        xml.push_str("<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n");
        xml.push_str("<Default Extension=\"bin\" ContentType=\"application/vnd.ms-excel.sheet.binary.macroEnabled.main\"/>\n");
        xml.push_str("<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\n");
        xml.push_str("<Default Extension=\"xml\" ContentType=\"application/xml\"/>\n");
        xml.push_str("<Override PartName=\"/docProps/app.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.extended-properties+xml\"/>\n");
        xml.push_str("<Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/>\n");
        xml.push_str("<Override PartName=\"/xl/workbook.bin\" ContentType=\"application/vnd.ms-excel.sheet.binary.macroEnabled.main\"/>\n");
        
        if has_sst {
            xml.push_str("<Override PartName=\"/xl/sharedStrings.bin\" ContentType=\"application/vnd.ms-excel.sharedStrings\"/>\n");
        }
        
        xml.push_str("<Override PartName=\"/xl/styles.bin\" ContentType=\"application/vnd.ms-excel.styles\"/>\n");
        xml.push_str("<Override PartName=\"/xl/theme/theme1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.theme+xml\"/>\n");
        
        for i in 1..=sheet_count {
            xml.push_str(&format!(
                "<Override PartName=\"/xl/worksheets/sheet{}.bin\" ContentType=\"application/vnd.ms-excel.worksheet\"/>\n",
                i
            ));
        }
        
        xml.push_str("</Types>");
        Bytes::copy_from_slice(xml.as_bytes())
    }
}
```

### 4.2 app.xml

```rust
impl XmlGen {
    pub fn app_xml(sheet_count: usize) -> Bytes {
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
            <Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\">\n\
            <Application>rxlsb</Application>\n\
            <SheetCount>{}</SheetCount>\n\
            </Properties>",
            sheet_count
        );
        Bytes::copy_from_slice(xml.as_bytes())
    }
}
```

### 4.3 core.xml

```rust
impl XmlGen {
    pub fn core_xml() -> Bytes {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
        <cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" \
        xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" \
        xmlns:dcmitype=\"http://purl.org/dc/dcmitype/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\n\
        <dc:creator>rxlsb</dc:creator>\n\
        <dcterms:created xsi:type=\"dcterms:W3CDTF\">2026-04-29T00:00:00Z</dcterms:created>\n\
        </cp:coreProperties>".to_string();
        Bytes::copy_from_slice(xml.as_bytes())
    }
}
```

### 4.4 theme.xml

```rust
impl XmlGen {
    pub fn theme_xml() -> Bytes {
        static THEME: &[u8] = include_bytes!("../../resources/theme1.xml");
        Bytes::from_static(THEME)
    }
    
    pub fn default_theme() -> Bytes {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
        <a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" name=\"Office Theme\">\n\
        <a:themeElements>\n\
        <a:clrScheme name=\"Office\">\n\
        <a:dk1><a:sysClr val=\"windowText\" lastClr=\"000000\"/></a:dk1>\n\
        <a:lt1><a:sysClr val=\"window\" lastClr=\"FFFFFF\"/></a:lt1>\n\
        <a:dk2><a:srgbClr val=\"1F497D\"/></a:dk2>\n\
        <a:lt2><a:srgbClr val=\"EEECE1\"/></a:lt2>\n\
        <a:accent1><a:srgbClr val=\"4F81BD\"/></a:accent1>\n\
        <a:accent2><a:srgbClr val=\"C0504D\"/></a:accent2>\n\
        <a:accent3><a:srgbClr val=\"9BBB59\"/></a:accent3>\n\
        <a:accent4><a:srgbClr val=\"8064A2\"/></a:accent4>\n\
        <a:accent5><a:srgbClr val=\"4BACC6\"/></a:accent5>\n\
        <a:accent6><a:srgbClr val=\"F79646\"/></a:accent6>\n\
        <a:hlink><a:srgbClr val=\"0000FF\"/></a:hlink>\n\
        <a:folHlink><a:srgbClr val=\"800080\"/></a:folHlink>\n\
        </a:clrScheme>\n\
        <a:fontScheme name=\"Office\">\n\
        <a:majorFont>\n\
        <a:latin typeface=\"Calibri\"/>\n\
        <a:ea typeface=\"\"/>\n\
        <a:cs typeface=\"\"/>\n\
        </a:majorFont>\n\
        <a:minorFont>\n\
        <a:latin typeface=\"Calibri\"/>\n\
        <a:ea typeface=\"\"/>\n\
        <a:cs typeface=\"\"/>\n\
        </a:minorFont>\n\
        </a:fontScheme>\n\
        <a:fmtScheme name=\"Office\">\n\
        <a:fillStyleLst>\n\
        <a:solidFill><a:srgbClr val=\"FFFFFF\"/></a:solidFill>\n\
        </a:fillStyleLst>\n\
        <a:lnStyleLst>\n\
        <a:ln w=\"9525\" cap=\"flat\" cmpd=\"sng\" algn=\"ctr\"><a:solidFill><a:srgbClr val=\"000000\"/></a:solidFill><a:prstDash val=\"solid\"/></a:ln>\n\
        </a:lnStyleLst>\n\
        </a:fmtScheme>\n\
        </a:themeElements>\n\
        </a:theme>";
        Bytes::copy_from_slice(xml.as_bytes())
    }
}
```

---

## 五、RelsGen

### 5.1 根关系文件

```rust
pub struct RelsGen;

impl RelsGen {
    pub fn root_rels() -> Bytes {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n\
        <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.bin\"/>\n\
        <Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/>\n\
        <Relationship Id=\"rId3\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties\" Target=\"docProps/app.xml\"/>\n\
        </Relationships>";
        Bytes::copy_from_slice(xml.as_bytes())
    }
}
```

### 5.2 Workbook关系文件

```rust
impl RelsGen {
    pub fn workbook_rels(sheet_count: usize, has_sst: bool) -> Bytes {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        xml.push_str("<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n");
        
        for i in 1..=sheet_count {
            xml.push_str(&format!(
                "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet{}.bin\"/>\n",
                i, i
            ));
        }
        
        let sst_rid = sheet_count + 1;
        if has_sst {
            xml.push_str(&format!(
                "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings\" Target=\"sharedStrings.bin\"/>\n",
                sst_rid
            ));
        }
        
        let styles_rid = if has_sst { sheet_count + 2 } else { sheet_count + 1 };
        xml.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.bin\"/>\n",
            styles_rid
        ));
        
        let theme_rid = styles_rid + 1;
        xml.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" Target=\"theme/theme1.xml\"/>\n",
            theme_rid
        ));
        
        xml.push_str("</Relationships>");
        Bytes::copy_from_slice(xml.as_bytes())
    }
}
```

---

## 六、容器写入流程

### 6.1 标准写入顺序

```rust
impl XlsbContainerWriter {
    pub fn write_xlsb_structure(&mut self,
                                workbook_data: &Bytes,
                                styles_data: &Bytes,
                                sst_data: Option<&Bytes>,
                                sheets_data: &[Bytes]) -> Result<()> {
        self.add_entry("[Content_Types].xml",
            &XmlGen::content_types(sheets_data.len(), sst_data.is_some()))?;
        
        self.add_entry("_rels/.rels", &RelsGen::root_rels())?;
        self.add_entry("docProps/app.xml",
            &XmlGen::app_xml(sheets_data.len()))?;
        self.add_entry("docProps/core.xml", &XmlGen::core_xml())?;
        self.add_entry("xl/theme/theme1.xml", &XmlGen::theme_xml())?;
        
        self.add_entry("xl/workbook.bin", workbook_data)?;
        self.add_entry("xl/_rels/workbook.bin.rels",
            &RelsGen::workbook_rels(sheets_data.len(), sst_data.is_some()))?;
        
        self.add_entry("xl/styles.bin", styles_data)?;
        
        if let Some(sst) = sst_data {
            self.add_entry("xl/sharedStrings.bin", sst)?;
        }
        
        for (i, sheet_data) in sheets_data.iter().enumerate() {
            self.add_entry(&format!("xl/worksheets/sheet{}.bin", i + 1), sheet_data)?;
        }
        
        Ok(())
    }
}
```

---

## 七、容器读取流程

### 7.1 解析XLSB结构

```rust
impl XlsbContainerReader {
    pub fn parse_xlsb_structure(&mut self) -> Result<XlsbStructure> {
        let workbook_data = self.get_workbook_data()?;
        let styles_data = self.get_styles_data()?;
        let sst_data = self.get_sst_data()?;
        
        let workbook = WorkbookReader::deserialize(&workbook_data)?;
        let styles = StylesRegistry::deserialize(&styles_data)?;
        
        let sst = sst_data.map(|data| SstTable::deserialize(&data))
            .transpose()?;
        
        let sheet_count = workbook.sheet_count();
        let sheets = (0..sheet_count)
            .map(|i| self.get_sheet_data(i))
            .collect::<Result<Vec<_>>>()?;
        
        Ok(XlsbStructure {
            workbook,
            styles,
            sst,
            sheets,
        })
    }
}

pub struct XlsbStructure {
    pub workbook: WorkbookReader,
    pub styles: StylesRegistry,
    pub sst: Option<SstTable>,
    pub sheets: Vec<Bytes>,
}
```

---

## 八、错误处理

### 8.1 ZIP错误映射

```rust
impl From<zip::result::ZipError> for XlsbError {
    fn from(e: zip::result::ZipError) -> Self {
        match e {
            zip::result::ZipError::Io(io_err) => XlsbError::Io(io_err),
            zip::result::ZipError::FileNotFound => XlsbError::FileNotFound,
            zip::result::ZipError::InvalidArchive(msg) => XlsbError::InvalidArchive(msg),
            _ => XlsbError::Zip(e),
        }
    }
}

#[derive(Error, Debug)]
pub enum XlsbError {
    #[error("文件未找到: {0}")]
    FileNotFound,
    
    #[error("无效的ZIP文件: {0}")]
    InvalidArchive(String),
    
    #[error("ZIP条目未找到: {0}")]
    EntryNotFound(String),
}
```