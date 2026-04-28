use crate::io::BufferReader;
use crate::format::{RecordType, SstTable};
use crate::api::{CellData, RowHandler};
use crate::error::Result;
use bytes::Bytes;

pub struct SheetReader<'a> {
    reader: BufferReader,
    sst: Option<&'a SstTable>,
}

impl<'a> SheetReader<'a> {
    pub fn new(data: Bytes, sst: Option<&'a SstTable>) -> Self {
        Self {
            reader: BufferReader::new(data),
            sst,
        }
    }
    
    pub fn for_each_row(&mut self, mut handler: impl RowHandler) -> Result<()> {
        let mut current_row: u32 = 0;
        let mut cells: Vec<CellData> = vec![];
        
        while self.reader.has_remaining() {
            let record_type_code = self.reader.read_u32_le()?;
            let size = self.reader.read_u32_le()?;
            
            let record_type = RecordType::from_u32(record_type_code);
            
            match record_type {
                Some(RecordType::BrtRowHdr) => {
                    if !cells.is_empty() {
                        handler.on_row(current_row as usize, &cells);
                        cells.clear();
                    }
                    current_row = self.reader.read_u32_le()?;
                    self.reader.skip(size as usize - 4)?;
                }
                
                Some(RecordType::BrtCellReal) => {
                    self.reader.skip(8)?;
                    let value = self.reader.read_f64_le()?;
                    cells.push(CellData::number(value));
                }
                
                Some(RecordType::BrtCellSt) => {
                    self.reader.skip(12)?;
                    let text = self.reader.read_wide_string()?;
                    cells.push(CellData::text(text));
                }
                
                Some(RecordType::BrtCellIsst) => {
                    self.reader.skip(12)?;
                    let sst_idx = self.reader.read_varint()?;
                    let text = self.sst.and_then(|s| s.get_string(sst_idx))
                        .unwrap_or("");
                    cells.push(CellData::text(text));
                }
                
                Some(RecordType::BrtCellBool) => {
                    self.reader.skip(12)?;
                    let value = self.reader.read_u8()? != 0;
                    cells.push(CellData::bool(value));
                }
                
                Some(RecordType::BrtCellBlank) => {
                    self.reader.skip(12)?;
                    cells.push(CellData::blank());
                }
                
                Some(RecordType::BrtEndSheetData) => {
                    if !cells.is_empty() {
                        handler.on_row(current_row as usize, &cells);
                    }
                    break;
                }
                
                _ => {
                    self.reader.skip(size as usize)?;
                }
            }
        }
        
        Ok(())
    }
}