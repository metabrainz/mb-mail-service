use mrml::{mjml, text::Text};
use mrmx_macros::view;
use serde_json::Value;

use crate::Mf1Keys;

use super::TemplateError;

pub(crate) fn basic(_: Value, t: &Mf1Keys) -> Result<mjml::Mjml, TemplateError> {
    Ok(view! {
        <mjml>
          <mj-body>
            <mj-button>Hello world in {Text::from(t.lang).into()}!</mj-button>
          </mj-body>
        </mjml>
    })
}
