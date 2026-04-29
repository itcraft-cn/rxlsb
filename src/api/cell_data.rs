use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub enum CellData {
    Text(String),
    Number(f64),
    Date(DateTime<Utc>),
    Bool(bool),
    Blank,
    Error(CellError),
}

#[derive(Clone, Debug)]
pub enum CellError {
    Div0, Value, Ref, Name, Num, Na,
}

impl CellData {
    pub fn text(s: impl Into<String>) -> Self { CellData::Text(s.into()) }
    pub fn number(n: f64) -> Self { CellData::Number(n) }
    pub fn date(d: DateTime<Utc>) -> Self { CellData::Date(d) }
    pub fn bool(b: bool) -> Self { CellData::Bool(b) }
    pub fn blank() -> Self { CellData::Blank }
}

pub trait CellSupplier {
    fn get_cell(&self, row: usize, col: usize) -> CellData;
}

impl<F: Fn(usize, usize) -> CellData> CellSupplier for F {
    fn get_cell(&self, row: usize, col: usize) -> CellData { self(row, col) }
}

pub trait RowHandler {
    fn on_row(&mut self, row_idx: usize, cells: &[CellData]);
}

impl<F: FnMut(usize, &[CellData])> RowHandler for F {
    fn on_row(&mut self, row_idx: usize, cells: &[CellData]) { self(row_idx, cells) }
}