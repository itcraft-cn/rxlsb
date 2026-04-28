use crate::io::BufferWriter;
use crate::format::RecordType;
use bytes::Bytes;
use crate::error::Result;

pub struct Font {
    name: String,
    size: f64,
    bold: bool,
    italic: bool,
    color: u32,
}

impl Font {
    pub fn default_font() -> Self {
        Self { name: "Calibri".to_string(), size: 11.0, bold: false, italic: false, color: 0 }
    }
}

pub struct Fill;
pub struct Border;
pub struct Xf;

impl Xf {
    pub fn default_xf() -> Self { Self }
}

pub struct StylesRegistry {
    fonts: Vec<Font>,
    fills: Vec<Fill>,
    borders: Vec<Border>,
    xfs: Vec<Xf>,
    num_formats: Vec<String>,
}

impl StylesRegistry {
    pub fn new() -> Self {
        Self {
            fonts: vec![Font::default_font()],
            fills: vec![Fill],
            borders: vec![Border],
            xfs: vec![Xf::default_xf()],
            num_formats: vec![],
        }
    }
    
    pub fn add_num_format(&mut self, format: &str) -> u32 {
        let idx = 164 + self.num_formats.len() as u32;
        self.num_formats.push(format.to_string());
        idx
    }
    
    pub fn count(&self) -> usize { self.xfs.len() }
    
    pub fn serialize(&self) -> Result<Bytes> {
        let mut writer = BufferWriter::new(1024);
        
        writer.write_u32_le(RecordType::BrtBeginStyleSheet.to_u32());
        writer.write_u32_le(0);
        
        writer.write_u32_le(RecordType::BrtEndStyleSheet.to_u32());
        writer.write_u32_le(0);
        
        Ok(writer.freeze())
    }
}