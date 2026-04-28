use crate::io::BufferWriter;
use crate::format::RecordType;
use bytes::Bytes;
use crate::error::Result;

pub struct StylesRegistry;

impl StylesRegistry {
    pub fn new() -> Self { Self }
    
    pub fn serialize(&self) -> Result<Bytes> {
        let mut writer = BufferWriter::new(2048);
        
        writer.write_varint(RecordType::BrtBeginCellStyleXFs.to_u32());
        writer.write_varsize(0);
        
        Self::write_formats(&mut writer)?;
        Self::write_fonts(&mut writer)?;
        Self::write_fills(&mut writer)?;
        Self::write_borders(&mut writer)?;
        Self::write_xfs(&mut writer)?;
        Self::write_styles(&mut writer)?;
        
        writer.write_varint(RecordType::BrtEndCellStyleXFs.to_u32());
        writer.write_varsize(0);
        
        Ok(writer.freeze())
    }
    
    fn write_formats(writer: &mut BufferWriter) -> Result<()> {
        writer.write_varint(RecordType::BrtBeginFmts.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(0);
        
        writer.write_varint(RecordType::BrtEndFmts.to_u32());
        writer.write_varsize(0);
        Ok(())
    }
    
    fn write_fonts(writer: &mut BufferWriter) -> Result<()> {
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
        Ok(())
    }
    
    fn write_fills(writer: &mut BufferWriter) -> Result<()> {
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
        Ok(())
    }
    
    fn write_borders(writer: &mut BufferWriter) -> Result<()> {
        writer.write_varint(RecordType::BrtBeginBorders.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(1);
        
        writer.write_varint(RecordType::BrtBorder.to_u32());
        writer.write_varsize(24);
        writer.write_bytes(&[0u8; 24]);
        
        writer.write_varint(RecordType::BrtEndBorders.to_u32());
        writer.write_varsize(0);
        Ok(())
    }
    
    fn write_xfs(writer: &mut BufferWriter) -> Result<()> {
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
        Ok(())
    }
    
    fn write_styles(writer: &mut BufferWriter) -> Result<()> {
        writer.write_varint(RecordType::BrtBeginStyles.to_u32());
        writer.write_varsize(4);
        writer.write_u32_le(1);
        
        writer.write_varint(RecordType::BrtXF.to_u32());
        writer.write_varsize(16);
        writer.write_bytes(&[
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x08, 0x10, 0x00, 0x00
        ]);
        
        writer.write_varint(RecordType::BrtEndStyles.to_u32());
        writer.write_varsize(0);
        Ok(())
    }
}