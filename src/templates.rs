use mrml::mjml::Mjml;
use serde_json::Value;

mod basic;
mod subscription;

#[derive(Debug, thiserror::Error)]
pub(crate) enum TemplateError {
    #[error("Failed to parse parameters: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub fn get(template_id: &str) -> Option<fn(Value) -> Result<Mjml, TemplateError>> {
    match template_id {
        "basic" => Some(basic::basic),
        "subscription" => Some(subscription::subscription),
        _ => None,
    }
}
