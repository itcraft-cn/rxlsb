use crate::io::BufferWriter;
use crate::format::{RecordType, number_format::NumberFormatRegistry};
use bytes::Bytes;

#[derive(Clone, Debug)]
struct CellXF {
    num_fmt_id: u16,
    font_id: u16,
    fill_id: u16,
    border_id: u16,
}

#[derive(Clone, Debug)]
struct StyleXF {
    num_fmt_id: u16,
    font_id: u16,
    fill_id: u16,
    border_id: u16,
}

pub struct StylesRegistry {
    format_registry: NumberFormatRegistry,
    cell_xfs: Vec<CellXF>,
    style_xfs: Vec<StyleXF>,
}

impl StylesRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            format_registry: NumberFormatRegistry::new(),
            cell_xfs: Vec::new(),
            style_xfs: Vec::new(),
        };
        registry.initialize_default_xfs();
        registry
    }
    
    fn initialize_default_xfs(&mut self) {
        // Add default cell XF (ifmt=0) at index 0
        self.cell_xfs.push(CellXF {
            num_fmt_id: 0,
            font_id: 0,
            fill_id: 0,
            border_id: 0,
        });
        
        // Add default style XF
        self.style_xfs.push(StyleXF {
            num_fmt_id: 0,
            font_id: 0,
            fill_id: 0,
            border_id: 0,
        });
    }
    
    pub fn get_style_id_for_format(&mut self, format_string: &str) -> u32 {
        let format_id = self.format_registry.get_or_add_format(format_string);
        
        // Check if this format already has a cell XF
        for (i, xf) in self.cell_xfs.iter().enumerate() {
            if xf.num_fmt_id == format_id {
                return i as u32;
            }
        }
        
        // Create new cell XF for this format
        self.cell_xfs.push(CellXF {
            num_fmt_id: format_id,
            font_id: 0,
            fill_id: 0,
            border_id: 0,
        });
        
        (self.cell_xfs.len() - 1) as u32
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
        self.write_cell_xfs(&mut writer);
        self.write_style_xfs(&mut writer);
        
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
    
    fn write_cell_xfs(&self, writer: &mut BufferWriter) {
        let count = self.cell_xfs.len() as u32;
        
        writer.write_varint(RecordType::BrtBeginXFs.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(count);
        
        for xf in &self.cell_xfs {
            writer.write_varint(RecordType::BrtXF.to_u32());
            writer.write_varsize(16);
            
            // BrtXF for cell XF (ixf=0xffff) - matches WPS structure
            // bytes 0-1: ixf (u16, 0xffff for cell XF)
            // bytes 2-3: ifmt (u16, numFmtId)
            // bytes 4-5: iFont (u16)
            // bytes 6-7: padding (u16, 0x0000)
            // bytes 8-9: iFill (u16)
            // bytes 10-11: padding (u16, 0x0000)
            // bytes 12-13: flags (u16, 0x0810)
            // bytes 14: borderId (u8)
            // bytes 15: padding (u8, 0x00)
            writer.write_u16_le(0xFFFF);                           // ixf = 0xffff
            writer.write_u16_le(xf.num_fmt_id);                    // ifmt
            writer.write_u16_le(xf.font_id);                       // iFont
            writer.write_u16_le(0x0000);                           // padding
            writer.write_u16_le(xf.fill_id);                       // iFill
            writer.write_u16_le(0x0000);                           // padding
            writer.write_u16_le(0x0810);                           // flags
            writer.write_u8(xf.border_id as u8);                   // borderId
            writer.write_u8(0x00);                                 // padding
        }
        
        writer.write_varint(RecordType::BrtEndXFs.to_u32());
        writer.write_varsize(0);
    }
    
    fn write_style_xfs(&self, writer: &mut BufferWriter) {
        let count = self.style_xfs.len() as u32;
        
        writer.write_varint(RecordType::BrtBeginStyles.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(count);
        
        for (idx, xf) in self.style_xfs.iter().enumerate() {
            writer.write_varint(RecordType::BrtXF.to_u32());
            writer.write_varsize(16);
            
            let is_first = idx == 0;
            
            // BrtXF for style XF (ixf=0x0000) - matches WPS structure
            // bytes 0-1: ixf (u16, 0x0000 for style XF)
            // bytes 2-3: ifmt (u16, numFmtId)
            // bytes 4-11: unused (all 0x00)
            // bytes 12-13: flags (u16, 0x0810)
            // bytes 14: styleId (u8, 0x00 for first, 0x01 for others)
            // bytes 15: unused (u8, 0x00)
            writer.write_u16_le(0x0000);                           // ixf = 0x0000
            writer.write_u16_le(xf.num_fmt_id);                    // ifmt
            writer.write_u32_le(0x00000000);                       // unused (bytes 4-7)
            writer.write_u32_le(0x00000000);                       // unused (bytes 8-11)
            writer.write_u16_le(0x0810);                           // flags
            writer.write_u8(if is_first { 0x00 } else { 0x01 });   // styleId
            writer.write_u8(0x00);                                 // unused
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