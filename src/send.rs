use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use lettre::{
    message::{MessageBuilder, MultiPart, SinglePart},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tracing::trace;

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
#[utoipa::path(
    get,
    path = "/send/{template_id}",
    responses(
        (status = 200, description = "Email sent successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    params(
        ("template_id" = String, Path, description = "Template to send"),
    )
)]
pub async fn send_mail_route(
    Path(template_id): Path<String>,
    State(mailer): State<MailTransport>,
) -> Result<Response, SendError> {
    let res = send_mail(template_id, mailer).await?;
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
#[tracing::instrument(skip(mailer))]
pub async fn send_mail(
    template_id: String,
    mailer: MailTransport,
) -> Result<lettre::transport::smtp::response::Response, SendError> {
    let (html, title) = render_html(template_id).await.unwrap();
    let text = render_text(&html).await.unwrap();
    let email = Message::builder()
        .from("Test Sender <sender@example.com>".parse().unwrap())
        .to("Test Reciever <reciever@example.com>".parse().unwrap())
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
