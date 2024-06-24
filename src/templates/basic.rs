use mrml::mjml;
use mrmx_macros::view;
use serde_json::Value;

use super::TemplateError;

pub(crate) fn basic(_: Value) -> Result<mjml::Mjml, TemplateError> {
    Ok(view! {
        <mjml>
          <mj-body>
            <mj-button>Hello world!</mj-button>
          </mj-body>
        </mjml>
    })
}
