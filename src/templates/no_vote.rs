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
struct NoVote {
    to_name: String,
    response_url: String,
    prefs_url: String,
    edit_id: u32,
    voter_name: String,
    close_time: String,
}

pub(crate) fn no_vote(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<NoVote> = serde_json::from_value(params)?;
    let NoVote {
        to_name,
        ref response_url,
        prefs_url,
        edit_id,
        ref voter_name,
        ref close_time,
    } = ctx.unwrap_or_default();
    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
            <mj-title>{ tl!(l, no_vote.title, edit_id = edit_id.to_string() ).borrow() }</mj-title>
        </mj-head>
        <mj-body width="500px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { header().into() }

                <mj-text>
                    <p>{ Text::from(tl!(l, greeting_line, name = to_name)).into() }</p>
                    <p>{ Text::from(tl!(l, no_vote.top, voter_name, edit_id = edit_id.to_string() )).into() }</p>
                    <p>{ Text::from(tl!(l, no_vote.reply )).into() }</p>
                </mj-text>

                <mj-wrapper mj-class="wrapper">
                    <mj-text>
                        <p>
                            <a href={response_url}>{ Text::from(response_url).into()}</a>
                        </p>
                    </mj-text>
                </mj-wrapper>


                <mj-text>
                    <p>{ Text::from(tl!(l, link_copy_info)).into() }</p>
                    <p>{ Text::from(tl!(l, no_vote.single_email)).into() }</p>
                    <p>{ Text::from(tl!(l, no_vote.close_time, close_time)).into() }</p>
                </mj-text>
                <mj-text>
                    <p><em>{ Text::from(tl!(l, metabrainz_signoff)).into() }</em></p>
                </mj-text>
                <mj-divider padding="10px 15px" border-color="#F5F5F5" border-width="3px" />
                <mj-text font-size="12px" color="#8D8D8D">
                    <p>
                        <a href={prefs_url}>{ Text::from(tl!(l, change_subscription_settings)).into() }</a>
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
