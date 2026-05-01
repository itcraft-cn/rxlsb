use std::collections::HashMap;

pub struct NumberFormatRegistry {
    format_string_to_id: HashMap<String, u16>,
    custom_formats: HashMap<u16, String>,
    next_custom_id: u16,
}

impl NumberFormatRegistry {
    const CUSTOM_FORMAT_START_ID: u16 = 164;
    
    pub fn new() -> Self {
        let mut registry = Self {
            format_string_to_id: HashMap::new(),
            custom_formats: HashMap::new(),
            next_custom_id: Self::CUSTOM_FORMAT_START_ID,
        };
        registry.initialize_built_in_formats();
        registry
    }
    
    fn initialize_built_in_formats(&mut self) {
        // Excel built-in formats (ID 0-22)
        // See: https://docs.microsoft.com/en-us/dotnet/api/documentformat.openxml.spreadsheet.numberingformat
        let built_in_formats = [
            (0, "General"),
            (1, "0"),
            (2, "0.00"),
            (3, "#,##0"),
            (4, "#,##0.00"),
            (5, "$#,##0;($#,##0)"),
            (6, "$#,##0;[Red]($#,##0)"),
            (7, "$#,##0.00;($#,##0.00)"),
            (8, "$#,##0.00;[Red]($#,##0.00)"),
            (9, "0%"),
            (10, "0.00%"),
            (11, "0.00E+00"),
            (12, "# ?/?"),
            (13, "# ??/??"),
            (14, "mm-dd-yy"),
            (15, "d-mmm-yy"),
            (16, "d-mmm"),
            (17, "mmm-yy"),
            (18, "h:mm AM/PM"),
            (19, "h:mm:ss AM/PM"),
            (20, "h:mm"),
            (21, "h:mm:ss"),
            (22, "m/d/yy h:mm"),
        ];
        
        for (id, format) in built_in_formats {
            self.format_string_to_id.insert(format.to_string(), id);
        }
    }
    
    pub fn add_format(&mut self, format_string: &str) -> u16 {
        if let Some(&id) = self.format_string_to_id.get(format_string) {
            return id;
        }
        
        let new_id = self.next_custom_id;
        self.next_custom_id += 1;
        
        self.format_string_to_id.insert(format_string.to_string(), new_id);
        self.custom_formats.insert(new_id, format_string.to_string());
        
        new_id
    }
    
    #[allow(dead_code)]
    pub fn get_format_id(&self, format_string: &str) -> Option<u16> {
        self.format_string_to_id.get(format_string).copied()
    }
    
    pub fn get_custom_formats(&self) -> &HashMap<u16, String> {
        &self.custom_formats
    }
    
    pub fn get_or_add_format(&mut self, format_string: &str) -> u16 {
        if let Some(&id) = self.format_string_to_id.get(format_string) {
            return id;
        }
        self.add_format(format_string)
    }
}

impl Default for NumberFormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}