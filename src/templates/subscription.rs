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
    name: Option<String>,
}

pub(crate) fn subscription(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<Subscription> = serde_json::from_value(params)?;
    let ctx = ctx.unwrap_or_default();
    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
        </mj-head>
        <mj-body width="500px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { header().into() }

                <mj-text>
                    <p>{ Text::from(tl!(l, greeting_line, name = ctx.name.unwrap_or("Jade".into()) )).into() }</p>
                    <p>"New edits have been added for artists that you've subscribed to."</p>
                </mj-text>

                <mj-wrapper mj-class="wrapper" >
                    <mj-text>
                        <h2 >"Changes for your subscribed artists:"</h2>
                        <ul>
                            <li><a href="https://musicbrainz.org/artist/8d8d8a80-f74f-4f21-a44c-518cd6944ed2/edits">"Nathan (English EDM artist)"</a>" (0 open, 1 applied)"</li>
                        </ul>
                    </mj-text>
                </mj-wrapper>
                <mj-wrapper mj-class="wrapper" >
                    <mj-text>
                        <h2>"All open edits:"</h2>
                        <ul>
                            <li><a href="https://musicbrainz.org/edit/subscribed?open=1">All open edits for your subscribed entities</a></li>
                            <li><a href="https://musicbrainz.org/edit/subscribed_editors?open=1">All open edits by your subscribed editors</a></li>
                        </ul>


                    </mj-text>
                </mj-wrapper>
                <mj-text>
                    <p>Thanks for subscribing and voting!</p>
                    <p><em>"—  The MetaBrainz community"</em></p>
                </mj-text>
                <mj-divider padding="10px 15px" border-color="#F5F5F5" border-width="3px" />
                <mj-text font-size="12px" color="#8D8D8D">
                    <p>
                        "This is a notification that edits have been added for artists, labels, collections and editors to whom you subscribed on the MusicBrainz web site. "
                        <a href="https://musicbrainz.org/user/Jellis16/subscriptions">Click here to view or edit your subscription list</a>"."
                    </p>
                    <p>"Do not reply to this message. If you need help, please "<a href="https://metabrainz.org/contact">contact us</a>.</p>

                </mj-text>

            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>

    })
}
