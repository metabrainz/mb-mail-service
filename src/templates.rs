use mrml::mjml::Mjml;
use serde_json::Value;

use crate::Mf1Keys;

mod basic;
mod subscription;

#[derive(Debug, thiserror::Error)]
pub(crate) enum TemplateError {
    #[error("Failed to parse parameters: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
type Template = fn(Value, &Mf1Keys) -> Result<Mjml, TemplateError>;

pub fn get(template_id: &str) -> Option<Template> {
    match template_id {
        "basic" => Some(basic::basic),
        "subscription" => Some(subscription::subscription),
        _ => None,
    }
}
