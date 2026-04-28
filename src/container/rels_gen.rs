use bytes::Bytes;

pub struct RelsGen;

impl RelsGen {
    pub fn root_rels() -> Bytes { Bytes::new() }
    pub fn workbook_rels(_sheet_count: usize, _has_sst: bool) -> Bytes { Bytes::new() }
}