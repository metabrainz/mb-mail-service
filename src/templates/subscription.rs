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
struct Subscription {
    to_name: String,
}

pub(crate) fn subscription(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<Subscription> = serde_json::from_value(params)?;

    let Subscription { to_name } = ctx.unwrap_or_default();
    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
            <mj-title>{ tl!(l, subscription.title ).borrow() }</mj-title>
        </mj-head>
        <mj-body width="500px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { header().into() }

                <mj-text>
                    <p>{ Text::from(tl!(l, greeting_line, name = to_name )).into() }</p>
                    <p>{ Text::from(tl!(l, subscription.info )).into() }</p>
                </mj-text>

                <mj-wrapper mj-class="wrapper" >
                    <mj-text>
                        <h2 >{ Text::from(tl!(l, subscription.subscribed_artists_changes )).into() }</h2>
                        <ul>
                            <li><a href="https://musicbrainz.org/artist/8d8d8a80-f74f-4f21-a44c-518cd6944ed2/edits">"Nathan (English EDM artist)"</a>" (0 open, 1 applied)"</li>
                        </ul>
                    </mj-text>
                </mj-wrapper>
                <mj-wrapper mj-class="wrapper" >
                    <mj-text>
                        <h2>{ Text::from(tl!(l, subscription.open_edits )).into() }</h2>
                        <ul>
                            <li><a href="https://musicbrainz.org/edit/subscribed?open=1">{ Text::from(tl!(l, subscription.open_edits_subscribed_entities )).into() }</a></li>
                            <li><a href="https://musicbrainz.org/edit/subscribed_editors?open=1">{ Text::from(tl!(l, subscription.open_edits_subscribed_editors )).into() }</a></li>
                        </ul>


                    </mj-text>
                </mj-wrapper>
                <mj-text>
                    <p>{ Text::from(tl!(l, subscription.thanks)).into() }</p>
                    <p><em>{ Text::from(tl!(l, metabrainz_signoff)).into() }</em></p>
                </mj-text>
                <mj-divider padding="10px 15px" border-color="#F5F5F5" border-width="3px" />
                <mj-text font-size="12px" color="#8D8D8D">

                    <p>{ Text::from(tl!(l, subscription.about)).into() }</p>
                    <p>
                        <a href="https://musicbrainz.org/user/Jellis16/subscriptions">{ Text::from(tl!(l, change_subscription_settings)).into() }</a>
                    </p>
                    <p>{ Text::from(tl!(l, do_not_reply)).into() }</p>
                </mj-text>

            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>

    })
}
