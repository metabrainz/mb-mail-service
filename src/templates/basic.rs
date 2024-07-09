use mf1::t_l_string as tl;
use mrml::{mjml, text::Text};
use mrmx_macros::view;
use serde_json::Value;

use crate::Locale;

use super::TemplateError;

pub(crate) fn basic(_: Value, l: Locale) -> Result<mjml::Mjml, TemplateError> {
    Ok(view! {
        <mjml>
          <mj-body>
            <mj-button>"Hello world in "{Text::from(tl!(l, lang)).into()}!</mj-button>
          </mj-body>
        </mjml>
    })
}
