use mrml::mjml;
use mrmx_macros::view;
use serde_json::Value;

use crate::Locale;

use super::TemplateError;

pub(crate) fn basic(_: Value, _l: Locale) -> Result<mjml::Mjml, TemplateError> {
    Ok(view! {
        <mjml>
          <mj-body>
            <mj-button>"Hello world in English!"</mj-button>
          </mj-body>
        </mjml>
    })
}
