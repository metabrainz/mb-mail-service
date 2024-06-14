use mrml::mjml::Mjml;

mod basic;
mod subscription;

pub fn get(template_id: &str) -> Option<fn() -> Mjml> {
    match template_id {
        "basic" => Some(basic::basic),
        "subscription" => Some(subscription::subscription),
        _ => None,
    }
}
