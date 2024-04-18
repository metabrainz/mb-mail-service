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
}

impl IntoResponse for EngineError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Parse(ref inner) => tracing::error!("Unable to parse template: {inner:?}"),
            Self::Render(ref inner) => tracing::error!("Unable to render template: {inner:?}"),
        };
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("{self}"),
        )
            .into_response()
    }
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
pub async fn render_html(Path(template_id): Path<String>) -> Result<Response, EngineError> {
    let path = template_id + ".mjml";
    if let Some(template) = TemplateFiles::get(&path)
        .map(|f| String::from_utf8(f.data.to_vec()).expect("Template was not valid UTF-8"))
    {
        let opts = ParserOptions {
            include_loader: Box::new(TemplateFiles),
        };
        let root = mrml::parse_with_options(template, &opts)?;
        let opts = mrml::prelude::render::RenderOptions::default();
        let content = root.render(&opts)?;

        Ok(([(header::CONTENT_TYPE, "text/html")], content).into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, format!("Not Found: {}", path)).into_response())
    }
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
pub async fn render_text(Path(template_id): Path<String>) -> Result<Response, EngineError> {
    let path = template_id + ".mjml";
    if let Some(template) = TemplateFiles::get(&path)
        .map(|f| String::from_utf8(f.data.to_vec()).expect("Template was not valid UTF-8"))
    {
        let opts = ParserOptions {
            include_loader: Box::new(TemplateFiles),
        };
        let root = mrml::parse_with_options(template, &opts)?;
        let opts = mrml::prelude::render::RenderOptions::default();
        let content = root.render(&opts)?;

        Ok((
            [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
            html2text::config::plain()
                .string_from_read(content.as_bytes(), 50)
                .expect("Failed to convert to HTML"),
        )
            .into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, format!("Not Found: {}", path)).into_response())
    }
}

#[test]
fn render() -> Result<(), Box<dyn std::error::Error>> {
    use handlebars::Handlebars;
    use serde_json::Map;
    use std::fs::File;

    let mut handlebars = Handlebars::new();

    handlebars
        .register_embed_templates::<TemplateFiles>()
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
