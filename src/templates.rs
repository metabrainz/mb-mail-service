use mrml::mjml::Mjml;
use serde_json::Value;

mod basic;
mod subscription;

pub fn get(template_id: &str) -> Option<fn(Value) -> Mjml> {
    match template_id {
        "basic" => Some(basic::basic),
        "subscription" => Some(subscription::subscription),
        _ => None,
    }
}
