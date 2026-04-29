use bytes::Bytes;

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
    
    pub fn core_xml() -> Bytes {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
        <cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" \
        xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" \
        xmlns:dcmitype=\"http://purl.org/dc/dcmitype/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\n\
        <dc:creator>rxlsb</dc:creator>\n\
        <dcterms:created xsi:type=\"dcterms:W3CDTF\">2026-04-29T00:00:00Z</dcterms:created>\n\
        </cp:coreProperties>";
        Bytes::copy_from_slice(xml.as_bytes())
    }
    
    pub fn theme_xml() -> Bytes {
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
        <a:majorFont><a:latin typeface=\"Calibri\"/></a:majorFont>\n\
        <a:minorFont><a:latin typeface=\"Calibri\"/></a:minorFont>\n\
        </a:fontScheme>\n\
        <a:fmtScheme name=\"Office\">\n\
        <a:fillStyleLst><a:solidFill><a:srgbClr val=\"FFFFFF\"/></a:solidFill></a:fillStyleLst>\n\
        </a:fmtScheme>\n\
        </a:themeElements>\n\
        </a:theme>";
        Bytes::copy_from_slice(xml.as_bytes())
    }
}