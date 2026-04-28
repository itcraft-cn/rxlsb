use bytes::Bytes;

pub struct XmlGen;

impl XmlGen {
    pub fn content_types(_sheet_count: usize, _has_sst: bool) -> Bytes { Bytes::new() }
    pub fn app_xml(_sheet_count: usize) -> Bytes { Bytes::new() }
    pub fn core_xml() -> Bytes { Bytes::new() }
    pub fn theme_xml() -> Bytes { Bytes::new() }
}