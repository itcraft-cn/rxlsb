use std::collections::HashMap;

pub struct SstTable {
    strings: Vec<String>,
    hash_map: HashMap<String, u32>,
}

impl SstTable {
    pub fn new() -> Self {
        Self { strings: vec![], hash_map: HashMap::new() }
    }
    
    pub fn add_string(&mut self, s: &str) -> u32 {
        if let Some(idx) = self.hash_map.get(s) { return *idx; }
        let idx = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.hash_map.insert(s.to_string(), idx);
        idx
    }
    
    pub fn get_string(&self, idx: u32) -> Option<&str> {
        self.strings.get(idx as usize).map(|s| s.as_str())
    }
    
    pub fn count(&self) -> usize { self.strings.len() }
}