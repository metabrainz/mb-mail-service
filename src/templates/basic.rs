use mrml::mjml;
use mrmx_macros::view;
use serde_json::Value;

pub(crate) fn basic(_: Value) -> mjml::Mjml {
    view! {
        <mjml>
          <mj-body>
            <mj-button>Hello world!</mj-button>
          </mj-body>
        </mjml>
    }
}
