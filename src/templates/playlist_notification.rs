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
struct PlaylistNotification {
    to_name: String,
    playlist_name: String,
    playlist_url: String,
    notification_settings_url: String,
}
pub(crate) fn playlist_notification(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<PlaylistNotification> = serde_json::from_value(params)?;
    let PlaylistNotification {
        to_name: ref to_name_raw,
        playlist_name: ref playlist_name_raw,
        playlist_url,
        notification_settings_url,
    } = ctx.unwrap_or_default();

    let to_name = &encode_text(to_name_raw);
    let playlist_name = &encode_text(playlist_name_raw);

    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
            <mj-title>{ tl!(l, playlist_notification.title, playlist_name).borrow() }</mj-title>
        </mj-head>
        <mj-body width="660px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { lb_header().into() }

                <mj-text font-size="14px">
                    <p>{ Text::from(tl!(l, greeting_line, name = to_name)).into() }</p>
                </mj-text>

                <mj-wrapper mj-class="wrapper">
                    <mj-section padding="0"><mj-column padding="0">
                        <mj-text align="center">
                           <p>{ Text::from(tl!(l, playlist_notification.info, playlist_name)).into() }</p>
                        </mj-text>

                        <mj-button
                            href={playlist_url}
                            background-color="#353070"
                            border-radius="8px"
                            font-size="14px"
                        >
                           { Text::from(tl!(l, playlist_notification.button_text)).into() }
                        </mj-button>
                    </mj-column></mj-section>
                </mj-wrapper>

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
