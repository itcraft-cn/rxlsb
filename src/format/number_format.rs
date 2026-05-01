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
    
    fn get_built_in_format_id(format_string: &str) -> Option<u16> {
        match format_string {
            "mm-dd-yy" => Some(14),
            "d-mmm-yy" => Some(15),
            "d-mmm" => Some(16),
            "mmm-yy" => Some(17),
            "h:mm AM/PM" => Some(18),
            "h:mm:ss AM/PM" => Some(19),
            "h:mm" => Some(20),
            "h:mm:ss" => Some(21),
            "m/d/yy h:mm" => Some(22),
            _ => None,
        }
    }
    
    pub fn get_or_add_format(&mut self, format_string: &str) -> u16 {
        if let Some(id) = Self::get_built_in_format_id(format_string) {
            return id;
        }
        
        for (ifmt, fmt) in &self.custom_formats {
            if fmt == format_string {
                return *ifmt;
            }
        }
        
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