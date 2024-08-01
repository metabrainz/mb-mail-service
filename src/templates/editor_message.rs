use std::borrow::Borrow;

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
struct EditorMessage {
    to_name: String,
    from_name: String,
    subject: String,
    message: String,
    contact_url: String,
    revealed_address: bool,
}

pub(crate) fn editor_message(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<EditorMessage> = serde_json::from_value(params)?;
    let EditorMessage {
        to_name,
        ref from_name,
        subject: _,
        message,
        contact_url,
        revealed_address,
    } = ctx.unwrap_or_default();
    // Reply via email is optional
    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
            <mj-title>{ tl!(l, editor_message.title, from_name ).borrow() }</mj-title>
            <mj-style>"
                div.speech {
                    position: relative;
                }
                div .speech::after {
                    display: block;
                    width: 0;
                    content: \"\";
                    border: 15px solid transparent;
                    border-left-color: #F5F5F5;
                    position: absolute;
                    bottom: -15px;
                    left: 15px;
                    z-index: -1;
                }
            "</mj-style>
        </mj-head>
        <mj-body width="500px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { header().into() }

                <mj-text>
                    <p>{ Text::from(tl!(l, greeting_line, name = to_name)).into() }</p>
                    <p>{ Text::from(tl!(l, editor_message.top, from_name )).into() }</p>
                </mj-text>

                <mj-wrapper mj-class="wrapper" css-class="speech" >
                    <mj-text>
                        <strong >{ Text::from(from_name.to_owned() + ": ").into()}</strong>
                        <p>
                            { Text::from(message).into()}
                        </p>
                    </mj-text>
                </mj-wrapper>
                <mj-text>
                    <p><a href={contact_url}>{ Text::from(tl!(l, editor_message.reply_link, from_name )).into() }</a></p>
                    { if revealed_address {
                        view!(<p>{ Text::from(tl!(l, editor_message.reply_email )).into() }</p>).into()
                    } else { view!(<> </>).into() }}
                    <p><em>{ Text::from(tl!(l, metabrainz_signoff)).into() }</em></p>
                </mj-text>
                <mj-divider padding="10px 15px" border-color="#F5F5F5" border-width="3px" />
                <mj-text font-size="12px" color="#8D8D8D">
                    <p>{ Text::from(tl!(l, do_not_reply)).into() }</p>
                    // <p>"Do not reply to this message. If you need help, please "<a href="https://metabrainz.org/contact">contact us</a>.</p>
                </mj-text>
            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>

    })
}
