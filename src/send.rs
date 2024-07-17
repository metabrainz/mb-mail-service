use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use futures::prelude::*;
use lettre::{
    message::{MessageBuilder, MultiPart, SinglePart},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use tracing::trace;
use utoipa::{IntoParams, ToSchema};

use crate::render::{render_html, render_text, EngineError};

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

/// Todo search query
#[derive(Deserialize, IntoParams)]
pub(crate) struct SendMailQuery {
    /// Address to send mail from.
    from: Option<String>,
    /// Address to send mail to.
    to: Option<String>,
    /// Language to render the template with
    lang: Option<String>,
}

#[utoipa::path(
    post,
    path = "/send/{template_id}",
    responses(
        (status = 200, description = "Email sent successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    params(
        ("template_id" = String, Path, description = "Template to send"),
        SendMailQuery
    ),
    request_body = Value
)]
pub async fn send_mail_route(
    Path(template_id): Path<String>,
    State(mailer): State<MailTransport>,
    Query(options): Query<SendMailQuery>,
    Json(body): Json<Value>,
) -> Result<Response, SendError> {
    let res = send_mail(template_id, &mailer, options, body).await?;
    let code = res.code();
    trace!("{:?}", res);
    Ok((
        if res.is_positive() {
            StatusCode::OK
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        },
        [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
        format!(
            "{code}: {code:?}\n{}",
            res.message().fold(String::new(), |s, n| s + n + "\n")
        ),
    )
        .into_response())
}

/// A single item in a bulk job
#[derive(Deserialize, ToSchema, Clone)]
pub struct BulkSendItem {
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
pub enum BulkSendResultItem {
    Success { code: u16, message: String },
    Error { message: String },
}

const PAR_SENDERS: usize = 6;

#[tracing::instrument(skip(mailer, items))]
#[utoipa::path(
    post,
    path = "/send_bulk",
    responses(
        (status = 200, description = "Bulk job ran successfully", body = [BulkSendResultItem])
    ),
    request_body = Vec<BulkSendItem>,
)]
pub async fn send_mail_bulk_route(
    State(mailer): State<MailTransport>,
    Json(items): Json<Vec<BulkSendItem>>,
) -> Result<Json<Vec<BulkSendResultItem>>, SendError> {
    let all_results = dashmap::DashMap::new();
    stream::iter(items)
        .enumerate()
        .for_each_concurrent(
            PAR_SENDERS,
            |(
                i,
                BulkSendItem {
                    template_id,
                    from,
                    to,
                    lang,
                    params,
                },
            )| {
                // This is a hack to not move some values
                // https://stackoverflow.com/questions/58459643/
                let mailer = &mailer;
                let all_results = &all_results;
                async move {
                    let res = send_mail(
                        template_id,
                        mailer,
                        SendMailQuery { from, to, lang },
                        params,
                    )
                    .await;
                    all_results.insert(i, res);
                }
            },
        )
        .await;

    Ok(Json(
        all_results
            .into_read_only()
            .values()
            .map(|i| match i {
                Ok(res) => BulkSendResultItem::Success {
                    code: res.code().into(),
                    message: res.message().fold(String::new(), |s, n| s + n + "\n"),
                },

                Err(e) => BulkSendResultItem::Error {
                    message: e.to_string(),
                },
            })
            .collect(),
    ))
}

#[tracing::instrument(skip(mailer))]
pub async fn send_mail(
    template_id: String,
    mailer: &MailTransport,
    SendMailQuery { from, to, lang }: SendMailQuery,
    params: Value,
) -> Result<lettre::transport::smtp::response::Response, SendError> {
    let lang = lang
        .map(|l| crate::Locale::from_str(&l).unwrap())
        .unwrap_or_default();
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
