use mrml::mjml;
use mrmx_macros::view;

pub(crate) fn basic() -> mjml::Mjml {
    view! {
        <mjml>
          <mj-body>
            <mj-button>Hello world!</mj-button>
          </mj-body>
        </mjml>
    }
}
