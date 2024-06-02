use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

use mrml::prelude::parser::{loader::IncludeLoader, ParserOptions};

#[derive(RustEmbed, Debug)]
#[folder = "templates"]
#[include = "*.hbs"]
#[include = "*.mjml"]
struct TemplateFiles;
impl IncludeLoader for TemplateFiles {
    fn resolve(
        &self,
        path: &str,
    ) -> Result<String, mrml::prelude::parser::loader::IncludeLoaderError> {
        tracing::debug!("Loading path {path:?}");
        TemplateFiles::get(path)
            .map(|f| String::from_utf8(f.data.to_vec()).expect("Template was not valid UTF-8"))
            .ok_or_else(|| mrml::prelude::parser::loader::IncludeLoaderError::not_found(path))
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum EngineError {
    #[error("Failed to parse template: {0}")]
    Parse(#[from] mrml::prelude::parser::Error),
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
pub async fn render_html(template_id: String) -> Result<String, EngineError> {
    let path = template_id + ".mjml";
    let template = TemplateFiles::get(&path)
        .map(|f| String::from_utf8(f.data.to_vec()).expect("Template was not valid UTF-8"))
        .ok_or(EngineError::TemplateNotFound(path))?;
    let opts = ParserOptions {
        include_loader: Box::new(TemplateFiles),
    };
    let root = mrml::parse_with_options(template, &opts)?;
    let opts = mrml::prelude::render::RenderOptions::default();
    let content = root.render(&opts)?;
    Ok(content)
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
    let content = render_html(template_id).await?;

    Ok(([(header::CONTENT_TYPE, "text/html")], content).into_response())
}

pub async fn render_text(html: String) -> Result<String, EngineError> {

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
    let html = render_html(template_id).await?;
    let content = render_text(html).await?;

    Ok((
        [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
        content,
    )
        .into_response())
}
#[cfg(test)]
mod test {
    use expect_test::expect_file;

    #[tokio::test]
    async fn basic_template_html() {
        let res: String = super::render_html("basic".to_string()).await.unwrap();
        let expected = expect_file!["../fixtures/basic.html"];
        expected.assert_eq(&res);
    }

    #[tokio::test]
    async fn include_template_html() {
        let res = super::render_html("include".to_string()).await.unwrap();
        let expected = expect_file!["../fixtures/include.html"];
        expected.assert_eq(&res);
    }

    #[tokio::test]
    async fn basic_template_text() {
        let html: String = super::render_html("basic".to_string()).await.unwrap();
        let res: String = super::render_text(html).await.unwrap();
        let expected = expect_file!["../fixtures/basic.txt"];
        expected.assert_eq(&res);
    }

    #[tokio::test]
    async fn include_template_text() {
        let html = super::render_html("include".to_string()).await.unwrap();
        let res = super::render_text(html).await.unwrap();
        let expected = expect_file!["../fixtures/include.txt"];
        expected.assert_eq(&res);
    }

    #[test]
    fn render() -> Result<(), Box<dyn std::error::Error>> {
        use handlebars::Handlebars;
        use serde_json::Map;
        use std::fs::File;

        let mut handlebars = Handlebars::new();

        handlebars
            .register_embed_templates::<crate::render::TemplateFiles>()
            .unwrap();

        println!("Loaded templates");

        let mut data = Map::new();
        data.insert("bin_name".into(), env!("CARGO_BIN_NAME").into());
        let mut output_file = File::create("target/test.html")?;
        handlebars.render_to_write("test.hbs", &data, &mut output_file)?;
        println!("target/test.html generated");

        Ok(())
    }

    // TODO: Iterate over and prerender all the templates?
    // fn render_mrml() {
    //     TemplateFiles::iter().map(f)
    // }
}
