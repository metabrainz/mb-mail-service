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
struct ElectionCancelled {
    candidate_name: String,
    candidate_url: String,
    election_url: String,
}

pub(crate) fn election_cancelled(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<ElectionCancelled> = serde_json::from_value(params)?;
    let ElectionCancelled {
        candidate_name: ref candidate_name_raw,
        ref candidate_url,
        ref election_url,
    } = ctx.unwrap_or_default();

    let candidate_name = &encode_text(candidate_name_raw);

    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
            <mj-title>{ tl!(l, election_cancelled.title, candidate_name = candidate_name_raw ).borrow() }</mj-title>
        </mj-head>
        <mj-body width="660px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { header().into() }
                <mj-text>
                    <p>{ Text::from(tl!(l, election_cancelled.salute )).into() }</p>
                    <p>{ Text::from(tl!(l, election_cancelled.opening_message )).into() }</p>
                </mj-text>
                <mj-wrapper mj-class="wrapper">
                    <mj-text>
                    <p>{ Text::from(tl!(l, election_cancelled.cancelled_message, candidate_name, candidate_url )).into() }</p>
                    <p>
                        <a href={election_url}>{ Text::from(tl!(l, election_cancelled.see_results_message )).into()}</a>
                    </p>
                    </mj-text>
                </mj-wrapper>
                <mj-text>
                    <p>{ Text::from(tl!(l, election_cancelled.election_thanks )).into() }</p>
                    <p>{ Text::from(tl!(l, election_cancelled.not_forever_reminder )).into() }</p>
                </mj-text>
                <mj-text>
                    <p>{ Text::from(tl!(l, election_cancelled.election_outro )).into() }</p>
                </mj-text>
            </mj-column>
        </mj-section>
        </mj-body>
      </mjml>

    })
}
