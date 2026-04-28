use crate::error::Result;
use crate::format::RecordType;

pub trait Biff12Writer {
    fn write_record_header(&mut self, _record_type: RecordType, _size: u32) {}
    fn write_empty_record(&mut self, _record_type: RecordType) {}
}

pub trait Biff12Reader {
    fn read_record_header(&mut self) -> Result<(RecordType, u32)> { Ok((RecordType::BrtRowHdr, 0)) }
}