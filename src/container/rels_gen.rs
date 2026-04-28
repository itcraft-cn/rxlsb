use bytes::Bytes;

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