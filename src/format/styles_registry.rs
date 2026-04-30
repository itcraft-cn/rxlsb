use crate::io::BufferWriter;
use crate::format::{RecordType, number_format::NumberFormatRegistry};
use bytes::Bytes;

pub struct StylesRegistry {
    format_registry: NumberFormatRegistry,
    styles: Vec<CellStyleFormat>,
}

#[derive(Clone, Debug)]
struct CellStyleFormat {
    num_fmt_id: u16,
    font_id: u16,
    #[allow(dead_code)]
    fill_id: u16,
    #[allow(dead_code)]
    border_id: u16,
}

impl StylesRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            format_registry: NumberFormatRegistry::new(),
            styles: Vec::new(),
        };
        registry.initialize_default_style();
        registry
    }
    
    fn initialize_default_style(&mut self) {
        self.styles.push(CellStyleFormat {
            num_fmt_id: 0,
            font_id: 0,
            fill_id: 0,
            border_id: 0,
        });
    }
    
    pub fn add_style(&mut self, num_fmt_id: u16, font_id: u16, fill_id: u16, border_id: u16) -> u32 {
        let style = CellStyleFormat {
            num_fmt_id,
            font_id,
            fill_id,
            border_id,
        };
        self.styles.push(style);
        (self.styles.len() - 1) as u32
    }
    
    pub fn get_style_id_for_format(&mut self, format_string: &str) -> u32 {
        let format_id = self.format_registry.get_or_add_format(format_string);
        
        for (i, style) in self.styles.iter().enumerate() {
            if style.num_fmt_id == format_id {
                return i as u32;
            }
        }
        
        self.add_style(format_id, 0, 0, 0)
    }
    
    #[allow(dead_code)]
    pub fn get_format_registry(&mut self) -> &mut NumberFormatRegistry {
        &mut self.format_registry
    }
    
    #[allow(dead_code)]
    pub fn get_default_style_id(&self) -> u32 {
        0
    }
    
    pub fn serialize(&self) -> Bytes {
        let mut writer = BufferWriter::new(4096);
        
        writer.write_varint(RecordType::BrtBeginCellStyleXFs.to_u32());
        writer.write_varsize(0);
        
        self.write_formats(&mut writer);
        Self::write_fonts(&mut writer);
        Self::write_fills(&mut writer);
        Self::write_borders(&mut writer);
        self.write_xfs(&mut writer);
        self.write_styles(&mut writer);
        
        writer.write_varint(RecordType::BrtEndCellStyleXFs.to_u32());
        writer.write_varsize(0);
        
        writer.freeze()
    }
    
    fn write_formats(&self, writer: &mut BufferWriter) {
        let custom_formats = self.format_registry.get_custom_formats();
        
        writer.write_varint(RecordType::BrtBeginFmts.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(custom_formats.len() as u32);
        
        for (format_id, format_string) in custom_formats {
            self.write_brt_fmt(writer, *format_id, format_string);
        }
        
        writer.write_varint(RecordType::BrtEndFmts.to_u32());
        writer.write_varsize(0);
    }
    
    fn write_brt_fmt(&self, writer: &mut BufferWriter, format_id: u16, format_string: &str) {
        let utf16_chars: Vec<u16> = format_string.encode_utf16().collect();
        let char_count = utf16_chars.len() as u32;
        let record_size = 2 + 4 + char_count * 2;
        
        writer.write_varint(RecordType::BrtFmt.to_u32());
        writer.write_varsize(record_size as u32);
        writer.write_u16_le(format_id);
        writer.write_u32_le(char_count);
        for ch in utf16_chars {
            writer.write_u16_le(ch);
        }
    }
    
    fn write_fonts(writer: &mut BufferWriter) {
        writer.write_varint(RecordType::BrtBeginFonts.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(1);
        
        writer.write_varint(RecordType::BrtFont.to_u32());
        writer.write_varsize(29);
        writer.write_bytes(&[
            0xDC, 0x00, 0x00, 0x00,
            0x90, 0x01, 0x00, 0x00,
            0x00, 0x00,
            0x86, 0x00,
            0x07, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0xFF,
            0x02, 0x02, 0x00, 0x00, 0x00,
            0x8B, 0x5B, 0x53, 0x4F
        ]);
        
        writer.write_varint(RecordType::BrtEndFonts.to_u32());
        writer.write_varsize(0);
    }
    
    fn write_fills(writer: &mut BufferWriter) {
        writer.write_varint(RecordType::BrtBeginFills.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(2);
        
        writer.write_varint(RecordType::BrtFill.to_u32());
        writer.write_varsize(4);
        writer.write_bytes(&[0x00, 0x00, 0x00, 0x00]);
        
        writer.write_varint(RecordType::BrtFill.to_u32());
        writer.write_varsize(4);
        writer.write_bytes(&[0x02, 0x00, 0x80, 0x00]);
        
        writer.write_varint(RecordType::BrtEndFills.to_u32());
        writer.write_varsize(0);
    }
    
    fn write_borders(writer: &mut BufferWriter) {
        writer.write_varint(RecordType::BrtBeginBorders.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(1);
        
        writer.write_varint(RecordType::BrtBorder.to_u32());
        writer.write_varsize(24);
        writer.write_bytes(&[0u8; 24]);
        
        writer.write_varint(RecordType::BrtEndBorders.to_u32());
        writer.write_varsize(0);
    }
    
    fn write_xfs(&self, writer: &mut BufferWriter) {
        writer.write_varint(RecordType::BrtBeginXFs.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(1);
        
        writer.write_varint(RecordType::BrtXF.to_u32());
        writer.write_varsize(16);
        writer.write_bytes(&[
            0xFF, 0xFF, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x08, 0x10, 0x00, 0x00
        ]);
        
        writer.write_varint(RecordType::BrtEndXFs.to_u32());
        writer.write_varsize(0);
    }
    
    fn write_styles(&self, writer: &mut BufferWriter) {
        let style_count = self.styles.len() as u32;
        
        writer.write_varint(RecordType::BrtBeginStyles.to_u32());
        writer.write_varsize(4 + style_count * 20);
        writer.write_u32_le(style_count);
        
        for (idx, style) in self.styles.iter().enumerate() {
            writer.write_varint(RecordType::BrtXF.to_u32());
            writer.write_varsize(16);
            
            let is_first = idx == 0;
            
            // BrtXF structure (16 bytes):
            // bytes 0-1: ixf (always 0x00 0x00)
            // bytes 2-3: ifmt (numFmtId) - THIS IS THE KEY!
            // bytes 4-11: unused (0x00)
            // bytes 12-13: flags (0x08 0x10)
            // byte 14: styleId (0x00 for first, 0x01+ for others)
            // byte 15: unused (0x00)
            writer.write_bytes(&[
                0x00, 0x00,                                  // ixf
                style.num_fmt_id as u8, (style.num_fmt_id >> 8) as u8,  // ifmt (formatId)
                0x00, 0x00, 0x00, 0x00,                      // unused
                0x00, 0x00, 0x00, 0x00,                      // unused
                0x08, 0x10,                                  // flags
                if is_first { 0x00 } else { idx as u8 },    // styleId
                0x00                                          // unused
            ]);
        }
        
        writer.write_varint(RecordType::BrtEndStyles.to_u32());
        writer.write_varsize(0);
    }
}

impl Default for StylesRegistry {
    fn default() -> Self {
        Self::new()
    }
}