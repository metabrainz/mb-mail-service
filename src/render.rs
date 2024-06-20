use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::Value;

use crate::templates;

#[derive(Debug, thiserror::Error)]
pub(crate) enum EngineError {
    #[error("Failed to render template: {0}")]
    Render(#[from] mrml::prelude::render::Error),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Failed to convert HTML to text: {0}")]
    FailedTextConversion(#[from] html2text::Error),
}

impl IntoResponse for EngineError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{self}: {self:?}");
        match self {
            EngineError::TemplateNotFound(_) => (StatusCode::NOT_FOUND, format!("{self}")),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")),
        }
        .into_response()
    }
}
pub async fn render_html(
    template_id: String,
    params: Value,
) -> Result<(String, Option<String>), EngineError> {
    let template =
        templates::get(&template_id).ok_or(EngineError::TemplateNotFound(template_id))?;
    let root = template(params);
    let opts = mrml::prelude::render::RenderOptions::default();
    let content = root.render(&opts)?;
    Ok((content, root.get_title()))
}

#[utoipa::path(
    get,
    path = "/templates/{template_id}/html",
    responses(
        (status = 200, description = "Template rendered successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    params(
        ("template_id" = String, Path, description = "Template to render"),
    )
)]
pub async fn render_html_route(Path(template_id): Path<String>) -> Result<Response, EngineError> {
    let (content, _title) = render_html(template_id, Value::Null).await?;

    Ok(([(header::CONTENT_TYPE, "text/html")], content).into_response())
}

pub async fn render_text(html: &str) -> Result<String, EngineError> {
    let text = html2text::config::plain().string_from_read(html.as_bytes(), 50)?;
    Ok(text)
}

#[utoipa::path(
    get,
    path = "/templates/{template_id}/text",
    responses(
        (status = 200, description = "Template rendered successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    params(
        ("template_id" = String, Path, description = "Template to render"),
    )
)]
pub async fn render_text_route(Path(template_id): Path<String>) -> Result<Response, EngineError> {
    let (html, _title) = render_html(template_id, Value::Null).await?;
    let content = render_text(&html).await?;

    Ok((
        [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
        content,
    )
        .into_response())
}

#[cfg(test)]
mod test {
    use expect_test::expect_file;
    use serde_json::{Map, Value};

    #[tokio::test]
    async fn basic_template_html() {
        let (res, _) = super::render_html("basic".to_string(), Value::Null)
            .await
            .unwrap();
        let expected = expect_file!["../fixtures/basic.html"];
        expected.assert_eq(&res);
    }

    #[tokio::test]
    async fn subscription_template_html() {
        let (res, _) = super::render_html("subscription".to_string(), Value::Object(Map::new()))
            .await
            .unwrap();
        let expected = expect_file!["../fixtures/subscription.html"];
        expected.assert_eq(&res);
    }

    #[tokio::test]
    async fn basic_template_text() {
        let (html, _) = super::render_html("basic".to_string(), Value::Null)
            .await
            .unwrap();
        let res: String = super::render_text(&html).await.unwrap();
        let expected = expect_file!["../fixtures/basic.txt"];
        expected.assert_eq(&res);
    }

    #[tokio::test]
    async fn subscription_template_text() {
        let (html, _) = super::render_html("subscription".to_string(), Value::Object(Map::new()))
            .await
            .unwrap();
        let res = super::render_text(&html).await.unwrap();
        let expected = expect_file!["../fixtures/subscription.txt"];
        expected.assert_eq(&res);
    }
}
