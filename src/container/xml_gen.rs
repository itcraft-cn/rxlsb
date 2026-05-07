use bytes::Bytes;
use chrono::Utc;
use once_cell::sync::Lazy;

static THEME_XML: Lazy<Bytes> = Lazy::new(|| {
    Bytes::from(include_str!("../resources/theme.xml"))
});

static APP_XML_TEMPLATE: &str = include_str!("../resources/app.xml");

static CONTENT_TYPES_HEADER: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">
<Default Extension=\"bin\" ContentType=\"application/vnd.ms-excel.sheet.binary.macroEnabled.main\"/>
<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>
<Default Extension=\"xml\" ContentType=\"application/xml\"/>
<Override PartName=\"/docProps/app.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.extended-properties+xml\"/>
<Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/>";

static CONTENT_TYPES_SST: &str = "<Override PartName=\"/xl/sharedStrings.bin\" ContentType=\"application/vnd.ms-excel.sharedStrings\"/>";
static CONTENT_TYPES_STYLES: &str = "<Override PartName=\"/xl/styles.bin\" ContentType=\"application/vnd.ms-excel.styles\"/>";
static CONTENT_TYPES_THEME: &str = "<Override PartName=\"/xl/theme/theme1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.theme+xml\"/>";

pub struct XmlGen;

impl XmlGen {
    pub fn content_types(sheet_count: usize, has_sst: bool) -> Bytes {
        let mut xml = String::new();
        xml.push_str(CONTENT_TYPES_HEADER);
        
        if has_sst {
            xml.push_str(CONTENT_TYPES_SST);
        }
        
        xml.push_str(CONTENT_TYPES_STYLES);
        xml.push_str(CONTENT_TYPES_THEME);
        
        for i in 1..=sheet_count {
            xml.push_str(&format!(
                "<Override PartName=\"/xl/worksheets/sheet{}.bin\" ContentType=\"application/vnd.ms-excel.worksheet\"/>",
                i
            ));
        }
        
        xml.push_str("</Types>");
        Bytes::copy_from_slice(xml.as_bytes())
    }
    
    pub fn app_xml(sheet_count: usize) -> Bytes {
        let xml = format!(
            "{}\n<SheetCount>{}</SheetCount>\n</Properties>",
            APP_XML_TEMPLATE, sheet_count
        );
        Bytes::copy_from_slice(xml.as_bytes())
    }
    
    pub fn core_xml() -> Bytes {
        let created_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" xmlns:dcmitype=\"http://purl.org/dc/dcmitype/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">
<dc:creator>rxlsb</dc:creator>
<dcterms:created xsi:type=\"dcterms:W3CDTF\">{}</dcterms:created>
</cp:coreProperties>",
            created_time
        );
        Bytes::copy_from_slice(xml.as_bytes())
    }
    
    pub fn theme_xml() -> Bytes {
        THEME_XML.clone()
    }
}