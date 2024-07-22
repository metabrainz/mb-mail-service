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
struct EditNote {
    name: String,
    user_id: String,
    edit_id: u32,
    from_username: String,
    message: String,
}

pub(crate) fn edit_note(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<EditNote> = serde_json::from_value(params)?;
    let EditNote {
        name,
        user_id,
        edit_id,
        from_username,
        message,
    } = ctx.unwrap_or_default();
    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
            <mj-title>{ tl!(l, edit_note.title, edit_id = edit_id.to_string() ).borrow() }</mj-title>
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
                    <p>{ Text::from(tl!(l, greeting_line, name)).into() }</p>
                    <p>{ Text::from(tl!(l, edit_note.top, edit_id = edit_id.to_string() )).into() }</p>
                </mj-text>

                <mj-wrapper mj-class="wrapper" css-class="speech" >
                    <mj-text>
                        <strong >{ Text::from(from_username + ": ").into()}</strong>
                        <p>
                            { Text::from(message).into()}
                        </p>
                    </mj-text>
                </mj-wrapper>
                <mj-text>
                    <p><a href={"https://musicbrainz.org/edit/".to_owned() + &edit_id.to_string()}>{ Text::from(tl!(l, edit_note.reply )).into() }</a></p>
                    <p><em>{ Text::from(tl!(l, metabrainz_signoff)).into() }</em></p>
                </mj-text>
                <mj-divider padding="10px 15px" border-color="#F5F5F5" border-width="3px" />
                <mj-text font-size="12px" color="#8D8D8D">
                    <p>
                        <a href={"https://musicbrainz.org/user/".to_owned() + &user_id + "/subscriptions"}>{ Text::from(tl!(l, change_subscription_settings)).into() }</a>
                    </p>
                    <p>{ Text::from(tl!(l, do_not_reply)).into() }</p>
                    // <p>"Do not reply to this message. If you need help, please "<a href="https://metabrainz.org/contact">contact us</a>.</p>

                </mj-text>

            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>

    })
}
