pub struct NumberFormatRegistry {
    custom_formats: Vec<(u16, String)>,
    next_custom_id: u16,
}

impl NumberFormatRegistry {
    const CUSTOM_FORMAT_START_ID: u16 = 164;
    
    pub fn new() -> Self {
        Self {
            custom_formats: Vec::new(),
            next_custom_id: Self::CUSTOM_FORMAT_START_ID,
        }
    }
    
    pub fn get_or_add_format(&mut self, format_string: &str) -> u16 {
        // Check if already added (match jxlsb behavior)
        for (ifmt, fmt) in &self.custom_formats {
            if fmt == format_string {
                return *ifmt;
            }
        }
        
        // Add new custom format (sequential ifmt like jxlsb)
        let new_id = self.next_custom_id;
        self.next_custom_id += 1;
        
        self.custom_formats.push((new_id, format_string.to_string()));
        
        new_id
    }
    
    pub fn get_custom_formats(&self) -> &Vec<(u16, String)> {
        &self.custom_formats
    }
}

impl Default for NumberFormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}