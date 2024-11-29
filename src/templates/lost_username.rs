use std::borrow::Borrow;

use html_escape::encode_text;
use mf1::t_l_string as tl;
use mrml::{mjml::Mjml, text::Text};
use mrmx::WithAttribute;
use mrmx_macros::view;
use serde::Deserialize;
use serde_json::Value;

use crate::{components::*, Locale};

use super::TemplateError;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct EmailInUse {
    to_name: String,
    lost_password_url: String,
    // ip?
}

pub(crate) fn lost_username(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<EmailInUse> = serde_json::from_value(params)?;
    let EmailInUse {
        ref to_name,
        ref lost_password_url,
    } = ctx.unwrap_or_default();

    let to_name = &encode_text(to_name);

    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
            <mj-title>{ tl!(l, lost_username.title ).borrow() }</mj-title>
        </mj-head>
        <mj-body width="560px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { header().into() }

                <mj-text>
                    <p>{ Text::from(tl!(l, greeting_line, name = to_name)).into() }</p>
                    <p>{ Text::from(tl!(l, lost_username.info )).into() }</p>
                    <p>{ Text::from(tl!(l, lost_username.username_is, user_name = to_name )).into() }</p>
                    <p>{ Text::from(tl!(l, lost_username.reset_password )).into() }</p>
                </mj-text>

                <mj-wrapper mj-class="wrapper">
                    <mj-text>
                        <p>
                            <a href={lost_password_url}>{ Text::from(encode_text(lost_password_url)).into()}</a>
                        </p>
                    </mj-text>
                </mj-wrapper>

                <mj-text>
                    <p>{ Text::from(tl!(l, link_copy_info)).into() }</p>
                    <p>{ Text::from(tl!(l, lost_username.in_error)).into() }</p>
                </mj-text>

                <mj-text>
                    <p><em>{ Text::from(tl!(l, metabrainz_signoff)).into() }</em></p>
                </mj-text>
                <mj-divider padding="10px 15px" border-color="#F5F5F5" border-width="3px" />
                <mj-text font-size="12px" color="#8D8D8D">
                    <p>{ Text::from(tl!(l, do_not_reply)).into() }</p>
                </mj-text>
            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>

    })
}
