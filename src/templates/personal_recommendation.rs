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
struct PersonalRecommendation {
    to_name: String,
    from_name: String,
    message: String,
    track_name: String,
    track_artist: String,
    track_url: String,
    album_art_url: String,
    notification_settings_url: String,
}

pub(crate) fn personal_recommendation(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<PersonalRecommendation> = serde_json::from_value(params)?;
    let PersonalRecommendation {
        to_name: ref to_name_raw,
        from_name: ref from_name_raw,
        message,
        track_name,
        track_artist,
        track_url,
        album_art_url,
        notification_settings_url,
    } = ctx.unwrap_or_default();

    let to_name = &encode_text(to_name_raw);
    let from_name = &encode_text(from_name_raw);
    let message = encode_text(&message);

    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
            <mj-title>{ tl!(l, personal_recommendation.title, from_name = from_name_raw).borrow() }</mj-title>
        </mj-head>
        <mj-body width="660px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { meb_header().into() }
                <mj-text>
                    <p>{ Text::from(tl!(l, greeting_line, name = to_name)).into() }</p>
                    <p>{ Text::from(tl!(l, personal_recommendation.info, from_name = from_name)).into() }</p>
                </mj-text>

                <mj-section border="1px solid #E0E0E0" border-radius="4px" padding="10px">
                    <mj-column>
                        <mj-text font-style="italic" color="#333">
                                <p>{ Text::from(message).into() }</p>
                        </mj-text>

                        <mj-image
                            border-radius="4px"
                            src={album_art_url}
                            alt={format!("Cover art for {} by {}", track_name, track_artist)}
                        />
                        <mj-text
                            align="center"
                            font-size="20px"
                            font-weight="bold"
                            padding="15px 5px 5px 5px"
                        >
                            { Text::from(track_name).into() }
                        </mj-text>
                        <mj-text
                            align="center"
                            font-size="16px"
                            color="#555"
                            padding="0px 5px 15px 5px"
                        >
                            { Text::from(track_artist).into() }
                        </mj-text>
                        <mj-button
                            href={track_url}
                            background-color="#e94363"
                            border-radius="20px"
                            width="100%"
                        >
                           { Text::from(tl!(l, personal_recommendation.button_text)).into() }
                        </mj-button>
                    </mj-column>
                </mj-section>

                <mj-text>
                    <p><em>{ Text::from(tl!(l, metabrainz_signoff)).into() }</em></p>
                </mj-text>
                <mj-divider padding="10px 15px" border-color="#F5F5F5" border-width="3px" />
                <mj-text font-size="12px" color="#8D8D8D">
                    <p>{ Text::from(tl!(l, lb_notification_about)).into() }</p>
                    <p>
                        <a href={notification_settings_url}>{ Text::from(tl!(l, change_notification_settings)).into() }</a>
                    </p>
                    <p>{ Text::from(tl!(l, do_not_reply)).into() }</p>
                </mj-text>
            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>
    })
}
