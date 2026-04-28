use crate::error::Result;

pub struct TemplateFiller;

impl TemplateFiller {
    pub fn builder() -> TemplateFillerBuilder { TemplateFillerBuilder }
}

pub struct TemplateFillerBuilder;

impl TemplateFillerBuilder {
    pub fn template(self, _t: impl Into<std::path::PathBuf>) -> Self { self }
    pub fn output(self, _o: impl Into<std::path::PathBuf>) -> Self { self }
    pub fn build(self) -> Result<TemplateFiller> { Ok(TemplateFiller) }
}