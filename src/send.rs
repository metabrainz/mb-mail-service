use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use futures::prelude::*;
use lettre::{
    message::{MessageBuilder, MultiPart, SinglePart},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use metrics::counter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::trace;
use utoipa::ToSchema;

use crate::{
    locale_from_optional_code,
    render::{render_mjml, render_template, render_text, EngineError},
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum SendError {
    #[error("Failed to render template: {0}")]
    FailedTemplate(#[from] EngineError),
    #[error("Failed to send mail: {0}")]
    SmtpError(#[from] lettre::transport::smtp::Error),
    #[error("Bad email address: {0}")]
    AddressError(#[from] lettre::address::AddressError),
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

/// All the data needed to send a single email based on a template
#[derive(Deserialize, ToSchema, Clone)]
pub struct SendTemplateItem {
    /// Template to send
    template_id: String,
    /// The address the email is from
    from: String,
    /// The address ultimately sending the email
    /// Should not be set if same as from address, as per RFC
    sender: Option<String>,
    /// Address to send mail to.
    to: String,
    /// Reply-To email header
    reply_to: Option<String>,
    /// Language to render the template with
    lang: Option<String>,
    /// Data to pass to the template
    params: Value,
    /// A unique identifier for the email
    /// Please see https://www.ietf.org/rfc/rfc2822.html#section-3.6.4
    message_id: Option<String>,
    /// The unique identifiers of the emails to which this is replying
    #[serde(default)]
    in_reply_to: Vec<String>,
    /// The unique identifiers of the emails that this email references
    #[serde(default)]
    references: Vec<String>,
}

/// All the data needed to send a single email based on a template
#[derive(Deserialize, ToSchema, Clone)]
pub struct SendMjmlItem {
    /// The MJML body to render and send
    mjml_text: String,

    /// The address the email is from
    from: String,
    /// The address ultimately sending the email
    /// Should not be set if same as from address, as per RFC
    sender: Option<String>,
    /// Address to send mail to.
    to: String,
    /// Reply-To email header
    reply_to: Option<String>,
    /// A unique identifier for the email
    /// Please see https://www.ietf.org/rfc/rfc2822.html#section-3.6.4
    message_id: Option<String>,
    /// The unique identifiers of the emails to which this is replying
    #[serde(default)]
    in_reply_to: Vec<String>,
    /// The unique identifiers of the emails that this email references
    #[serde(default)]
    references: Vec<String>,
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
    request_body = SendTemplateItem,
)]
pub async fn send_mail_route(
    State(mailer): State<MailTransport>,
    Json(item): Json<SendTemplateItem>,
) -> Result<(StatusCode, Json<SendResponse>), SendError> {
    counter!("mails_requested_total").increment(1);
    let res = send_mail_template(&mailer, item).await?;
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

#[utoipa::path(
    post,
    path = "/send_single_mjml",
    responses(
        (status = 200, description = "Email sent successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    request_body = SendMjmlItem,
)]
pub async fn send_mail_mjml_route(
    State(mailer): State<MailTransport>,
    Json(item): Json<SendMjmlItem>,
) -> Result<(StatusCode, Json<SendResponse>), SendError> {
    counter!("mails_requested_total").increment(1);
    let res = send_mail_mjml(&mailer, item).await?;
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
    request_body = Vec<SendTemplateItem>,
)]
pub async fn send_mail_bulk_route(
    State(mailer): State<MailTransport>,
    Json(items): Json<Vec<SendTemplateItem>>,
) -> Result<Json<Vec<SendResponse>>, SendError> {
    counter!("mails_requested_total").increment(items.len().try_into().unwrap());
    let all_results = dashmap::DashMap::new();
    stream::iter(items)
        .enumerate()
        .for_each_concurrent(PAR_SENDERS, |(i, item)| {
            // This is a hack to not move some values
            // https://stackoverflow.com/questions/58459643/
            let mailer = &mailer;
            let all_results = &all_results;
            async move {
                let res = send_mail_template(mailer, item).await;
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
pub async fn send_mail_template(
    mailer: &MailTransport,
    SendTemplateItem {
        template_id,
        from,
        sender,
        to,
        lang,
        params,
        reply_to,
        message_id,
        in_reply_to,
        references,
    }: SendTemplateItem,
) -> Result<lettre::transport::smtp::response::Response, SendError> {
    let lang = locale_from_optional_code(lang)?;
    let (html, title) = render_template(template_id, params, lang).await?;
    let text = render_text(&html).await?;
    let mut email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject_opt(title.as_deref())
        .message_id(message_id);
    if let Some(sender) = sender {
        email = email.sender(sender.parse()?);
    }
    if let Some(reply_to) = reply_to {
        email = email.reply_to(reply_to.parse()?);
    }
    for id in in_reply_to.into_iter() {
        email = email.in_reply_to(id)
    }
    for id in references.into_iter() {
        email = email.references(id)
    }

    let email = email
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

    counter!("mails_sent_total").increment(1);
    Ok(res)
}

#[tracing::instrument(skip(mailer))]
pub async fn send_mail_mjml(
    mailer: &MailTransport,
    SendMjmlItem {
        mjml_text,
        from,
        sender,
        to,
        reply_to,
        message_id,
        in_reply_to,
        references,
    }: SendMjmlItem,
) -> Result<lettre::transport::smtp::response::Response, SendError> {
    let (html, title) = render_mjml(mjml_text).await?;
    let text = render_text(&html).await?;
    let mut email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject_opt(title.as_deref())
        .message_id(message_id);
    if let Some(sender) = sender {
        email = email.sender(sender.parse()?);
    }
    if let Some(reply_to) = reply_to {
        email = email.reply_to(reply_to.parse()?);
    }
    for id in in_reply_to.into_iter() {
        email = email.in_reply_to(id)
    }
    for id in references.into_iter() {
        email = email.references(id)
    }

    let email = email
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

    counter!("mails_sent_total").increment(1);
    Ok(res)
}
