use mrml::mjml::Mjml;
use serde_json::Value;

use crate::Locale;

mod basic;
mod edit_note;
mod editor_message;
mod editor_report;
mod email_in_use;
mod lost_username;
mod no_vote;
mod reset_password;
mod subscription;
mod verify_email;

#[derive(Debug, thiserror::Error)]
pub(crate) enum TemplateError {
    #[error("Failed to parse parameters: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
type Template = fn(Value, Locale) -> Result<Mjml, TemplateError>;

pub fn get(template_id: &str) -> Option<Template> {
    match template_id {
        "basic" => Some(basic::basic),
        "subscription" => Some(subscription::subscription),
        "edit-note" => Some(edit_note::edit_note),
        "editor-message" => Some(editor_message::editor_message),
        "verify-email" => Some(verify_email::verify_email),
        "email-in-use" => Some(email_in_use::email_in_use),
        "reset-password" => Some(reset_password::reset_password),
        "lost-username" => Some(lost_username::lost_username),
        "no-vote" => Some(no_vote::no_vote),
        "editor-report" => Some(editor_report::editor_report),
        _ => None,
    }
}
