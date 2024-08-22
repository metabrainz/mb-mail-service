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
struct EditorReport {
    reported_name: String,
    from_name: String,
    reported_url: String,
    from_url: String,
    message: String,
    /// If the sender has shared their email address.
    ///
    /// If this is true, the user should be able to
    /// reply to the email to reply to the message.
    revealed_address: bool,
    /// If this is a copy for the message sender.
    ///
    /// If this is true, a banner should be added
    /// to the email
    #[serde(default)]
    is_self_copy: bool,
}

pub(crate) fn editor_report(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<EditorReport> = serde_json::from_value(params)?;
    let EditorReport {
        ref reported_name,
        ref from_name,
        ref reported_url,
        ref from_url,
        message,
        revealed_address,
        is_self_copy,
    } = ctx.unwrap_or_default();
    // Reply via email is optional
    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }

            { if !is_self_copy {
                view!{
                    <mj-title>{ tl!(l, editor_report.title, from_name, reported_name ).borrow() }</mj-title>
                }.into()
            } else { view!{
                <mj-title>{ tl!(l, editor_report.copy_title, reported_name ).borrow() }</mj-title>
            }.into() }}
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
                { if is_self_copy {
                    view!{
                        <mj-wrapper
                            border="1px solid #e2ce85"
                            background-color="#f4ebcb"
                            padding="5px 15px" >
                            <mj-text>
                                <p>{ Text::from(tl!(l, editor_report.message_copy, reported_name )).into() }</p>
                            </mj-text>
                        </mj-wrapper>
                    }.into()
                } else { view!(<></>).into() }}

                <mj-text>
                    <p>{ Text::from(tl!(l, editor_report.top, from_name, reported_name )).into() }</p>
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
                    <p>{ Text::from(tl!(l, editor_report.reporter_account )).into() }</p>
                </mj-text>
                <mj-wrapper mj-class="wrapper">
                    <mj-text>
                        <p>
                            <a href={from_url}>{ Text::from(from_url).into()}</a>
                        </p>
                    </mj-text>
                </mj-wrapper>
                <mj-text>
                    <p>{ Text::from(tl!(l, editor_report.reported_account )).into() }</p>
                </mj-text>
                <mj-wrapper mj-class="wrapper">
                    <mj-text>
                        <p>
                            <a href={reported_url}>{ Text::from(reported_url).into()}</a>
                        </p>
                    </mj-text>
                </mj-wrapper>
                <mj-text>
                    { if revealed_address {
                        view!(<p>{ Text::from(tl!(l, editor_report.reply_email )).into() }</p>).into()
                    } else { view!(<p>{ Text::from(tl!(l, editor_report.reply_no_email )).into() }</p>).into() }}
                    <p><em>{ Text::from(tl!(l, metabrainz_signoff)).into() }</em></p>
                </mj-text>
                <mj-divider padding="10px 15px" border-color="#F5F5F5" border-width="3px" />
                <mj-text font-size="12px" color="#8D8D8D">
                { if !revealed_address {
                    view!(<p>{ Text::from(tl!(l, do_not_reply)).into() }</p>).into()
                } else { view!(<></>).into()  }}
                </mj-text>
            </mj-column>
        </mj-section>
        </mj-body>
      </mjml>

    })
}
