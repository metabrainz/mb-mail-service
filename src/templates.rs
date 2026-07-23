use mrml::mjml::Mjml;
use serde_json::Value;

use crate::Locale;

mod basic;
mod cb_review;
mod edit_note;
mod editor_message;
mod editor_report;
mod email_in_use;
mod follow;
mod lost_username;
mod no_vote;
mod notification;
mod personal_recommendation;
mod playlist_notification;
mod recording_pin;
mod reset_password;
mod subscription;
mod thanks;
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
        "cb-review" => Some(cb_review::cb_review),
        "subscription" => Some(subscription::subscription),
        "edit-note" => Some(edit_note::edit_note),
        "follow" => Some(follow::follow),
        "editor-message" => Some(editor_message::editor_message),
        "verify-email" => Some(verify_email::verify_email),
        "email-in-use" => Some(email_in_use::email_in_use),
        "reset-password" => Some(reset_password::reset_password),
        "lost-username" => Some(lost_username::lost_username),
        "no-vote" => Some(no_vote::no_vote),
        "notification" => Some(notification::notification),
        "editor-report" => Some(editor_report::editor_report),
        "personal-recommendation" => Some(personal_recommendation::personal_recommendation),
        "playlist-notification" => Some(playlist_notification::playlist_notification),
        "recording-pin" => Some(recording_pin::recording_pin),
        "thanks" => Some(thanks::thanks),
        _ => {
            tracing::warn!("Unknown email template requested: {}", template_id);
            None
        },
    }
}
