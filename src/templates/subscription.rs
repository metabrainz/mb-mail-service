use mrml::mjml;
use mrmx::WithAttribute;
use mrmx_macros::view;
use serde::Deserialize;
use serde_json::Value;

use crate::{components::header, Mf1Keys};

use super::TemplateError;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct Subscription {
    name: Option<String>,
}

pub(crate) fn subscription(params: Value, _t: &Mf1Keys) -> Result<mjml::Mjml, TemplateError> {
    let ctx: Option<Subscription> = serde_json::from_value(params)?;
    let ctx = ctx.unwrap_or_default();
    Ok(view! {
        <mjml>
        <mj-head>
            <mj-font name="Inter" href="https://fonts.googleapis.com/css?family=Inter" />

            <mj-attributes>
            <mj-all padding="10px 30px" />
                <mj-text font-size="12px" line-height="14.52px" font-weight="400" font-size="12px" font-family="Inter" />
                <mj-class name="wrapper" border-radius="8px" background-color="#F5F5F5" padding="10px 15px" />
            </mj-attributes>
            <mj-style inline="inline">"
                h2 {
                    font-size: 12px;
                    font-weight: 700;
                }
                p {
                    margin: 6px 0;
                }
                ul {
                    padding-left: 20px;
                }
            "</mj-style>
        </mj-head>
        <mj-body width="500px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { header().into() }

                <mj-text>
                    <p>"Hello "{ mrml::text::Text::from(
                        ctx.name.unwrap_or("Jade".into())
                    ).into()},</p>
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
