use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use futures::prelude::*;
use lettre::{
    message::{MessageBuilder, MultiPart, SinglePart},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::trace;
use utoipa::ToSchema;

use crate::{
    locale_from_optional_code,
    render::{render_html, render_text, EngineError},
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum SendError {
    #[error("Failed to render template: {0}")]
    FailedTemplate(#[from] EngineError),
    #[error("Failed to send mail: {0}")]
    SendError(#[from] lettre::transport::smtp::Error),
}

impl IntoResponse for SendError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{self}: {self:?}");
        match self {
            Self::FailedTemplate(err) => err.into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response(),
        }
    }
}

pub type MailTransport = AsyncSmtpTransport<Tokio1Executor>;
trait OptionalSubject {
    fn subject_opt<S>(self, subject: Option<S>) -> Self
    where
        S: Into<String>;
}

impl OptionalSubject for MessageBuilder {
    fn subject_opt<S: Into<String>>(self, subject: Option<S>) -> Self {
        if let Some(subject) = subject {
            self.subject(subject)
        } else {
            self
        }
    }
}

/// All the data needed to send a single email
#[derive(Deserialize, ToSchema, Clone)]
pub struct SendItem {
    /// Template to send
    template_id: String,
    /// Address to send mail from.
    from: Option<String>,
    /// Address to send mail to.
    to: Option<String>,
    /// Language to render the template with
    lang: Option<String>,
    /// Data to pass to the template
    params: Value,
}

#[derive(Serialize, ToSchema, Clone)]
#[serde(tag = "t", content = "c")]
pub enum SendResponse {
    Success { code: u16, message: String },
    Error { message: String },
}

#[utoipa::path(
    post,
    path = "/send_single",
    responses(
        (status = 200, description = "Email sent successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    request_body = SendItem,
)]
pub async fn send_mail_route(
    State(mailer): State<MailTransport>,
    Json(item): Json<SendItem>,
) -> Result<(StatusCode, Json<SendResponse>), SendError> {
    let res = send_mail(&mailer, item).await?;
    trace!("{:?}", res);

    Ok((
        if res.is_positive() {
            StatusCode::OK
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        },
        Json(SendResponse::Success {
            code: res.code().into(),
            message: res.message().fold(String::new(), |s, n| s + n + "\n"),
        }),
    ))
}

const PAR_SENDERS: usize = 16;

#[tracing::instrument(skip(mailer, items))]
#[utoipa::path(
    post,
    path = "/send_bulk",
    responses(
        (status = 200, description = "Bulk job ran successfully", body = [SendResponse])
    ),
    request_body = Vec<SendItem>,
)]
pub async fn send_mail_bulk_route(
    State(mailer): State<MailTransport>,
    Json(items): Json<Vec<SendItem>>,
) -> Result<Json<Vec<SendResponse>>, SendError> {
    let all_results = dashmap::DashMap::new();
    stream::iter(items)
        .enumerate()
        .for_each_concurrent(PAR_SENDERS, |(i, item)| {
            // This is a hack to not move some values
            // https://stackoverflow.com/questions/58459643/
            let mailer = &mailer;
            let all_results = &all_results;
            async move {
                let res = send_mail(mailer, item).await;
                all_results.insert(i, res);
            }
        })
        .await;

    Ok(Json(
        all_results
            .into_read_only()
            .values()
            .map(|i| match i {
                Ok(res) => SendResponse::Success {
                    code: res.code().into(),
                    message: res.message().fold(String::new(), |s, n| s + n + "\n"),
                },

                Err(e) => SendResponse::Error {
                    message: e.to_string(),
                },
            })
            .collect(),
    ))
}

#[tracing::instrument(skip(mailer))]
pub async fn send_mail(
    mailer: &MailTransport,
    SendItem {
        template_id,
        from,
        to,
        lang,
        params,
    }: SendItem,
) -> Result<lettre::transport::smtp::response::Response, SendError> {
    let lang = locale_from_optional_code(lang)?;
    let (html, title) = render_html(template_id, params, lang).await?;
    let text = render_text(&html).await?;
    let email = Message::builder()
        .from(
            from.unwrap_or_else(|| "Test Sender <sender@example.com>".to_string())
                .parse()
                .unwrap(),
        )
        .to(to
            .unwrap_or_else(|| "Test Receiver <reciever@example.com>".to_string())
            .parse()
            .unwrap())
        .subject_opt(title.as_deref())
        .multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_PLAIN)
                        .body(text), // Every message should have a plain text fallback.
                )
                .singlepart(
                    SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_HTML)
                        .body(html),
                ),
        )
        .expect("failed to build email");
    let res = mailer.send(email).await?;

    Ok(res)
}
