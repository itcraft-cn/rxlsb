use crate::io::BufferWriter;
use crate::format::RecordType;

#[allow(dead_code)]
pub trait Biff12Writer {
    fn buffer(&mut self) -> &mut BufferWriter;
    
    fn write_record_header(&mut self, record_type: RecordType, size: u32) {
        self.buffer().write_u32_le(record_type.to_u32());
        self.buffer().write_u32_le(size);
    }
    
    fn write_empty_record(&mut self, record_type: RecordType) {
        self.write_record_header(record_type, 0);
    }
}