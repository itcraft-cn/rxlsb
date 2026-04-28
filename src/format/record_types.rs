#[derive(Clone, Copy)]
pub enum RecordType {
    BrtRowHdr = 0,
    BrtCellBlank = 1,
    BrtCellRk = 2,
    BrtCellError = 3,
    BrtCellBool = 4,
    BrtCellReal = 5,
    BrtCellSt = 6,
    BrtCellIsst = 7,
    BrtSstItem = 19,
    BrtBeginSheet = 129,
    BrtEndSheet = 130,
    BrtBeginBook = 131,
    BrtEndBook = 132,
    BrtBeginBundleShs = 143,
    BrtEndBundleShs = 144,
    BrtBeginSheetData = 145,
    BrtEndSheetData = 146,
    BrtWsProp = 147,
    BrtWsDim = 148,
    BrtBundleSh = 156,
    BrtBeginSst = 159,
    BrtEndSst = 160,
    BrtBeginStyleSheet = 370,
    BrtEndStyleSheet = 371,
}

impl RecordType {
    pub fn from_u32(code: u32) -> Option<Self> {
        match code {
            0 => Some(RecordType::BrtRowHdr),
            1 => Some(RecordType::BrtCellBlank),
            2 => Some(RecordType::BrtCellRk),
            3 => Some(RecordType::BrtCellError),
            4 => Some(RecordType::BrtCellBool),
            5 => Some(RecordType::BrtCellReal),
            6 => Some(RecordType::BrtCellSt),
            7 => Some(RecordType::BrtCellIsst),
            19 => Some(RecordType::BrtSstItem),
            129 => Some(RecordType::BrtBeginSheet),
            130 => Some(RecordType::BrtEndSheet),
            131 => Some(RecordType::BrtBeginBook),
            132 => Some(RecordType::BrtEndBook),
            143 => Some(RecordType::BrtBeginBundleShs),
            144 => Some(RecordType::BrtEndBundleShs),
            145 => Some(RecordType::BrtBeginSheetData),
            146 => Some(RecordType::BrtEndSheetData),
            147 => Some(RecordType::BrtWsProp),
            148 => Some(RecordType::BrtWsDim),
            156 => Some(RecordType::BrtBundleSh),
            159 => Some(RecordType::BrtBeginSst),
            160 => Some(RecordType::BrtEndSst),
            370 => Some(RecordType::BrtBeginStyleSheet),
            371 => Some(RecordType::BrtEndStyleSheet),
            _ => None,
        }
    }
    
    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}