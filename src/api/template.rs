use std::path::PathBuf;
use crate::error::{XlsbError, Result};

pub struct TemplateFiller;

impl TemplateFiller {
    pub fn builder() -> TemplateFillerBuilder {
        TemplateFillerBuilder { template: None, output: None }
    }
}

pub struct TemplateFillerBuilder {
    template: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl TemplateFillerBuilder {
    pub fn template(mut self, t: impl Into<PathBuf>) -> Self {
        self.template = Some(t.into());
        self
    }
    
    pub fn output(mut self, o: impl Into<PathBuf>) -> Self {
        self.output = Some(o.into());
        self
    }
    
    pub fn build(self) -> Result<TemplateFiller> {
        Ok(TemplateFiller)
    }
}