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
struct ElectionStart {
    candidate_name: String,
    proposer_name: String,
    candidate_url: String,
    election_url: String,
    proposer_url: String,
}

pub(crate) fn election_start(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<ElectionStart> = serde_json::from_value(params)?;
    let ElectionStart {
        candidate_name: ref candidate_name_raw,
        proposer_name: ref proposer_name_raw,
        ref candidate_url,
        ref election_url,
        ref proposer_url,
    } = ctx.unwrap_or_default();

    let candidate_name = &encode_text(candidate_name_raw);
    let proposer_name = &encode_text(proposer_name_raw);

    Ok(view! {
        <mjml>
        <mj-head>
            { head().into() }
            <mj-title>{ tl!(l, election_start.title, candidate_name = candidate_name_raw ).borrow() }</mj-title>
        </mj-head>
        <mj-body width="660px" padding="0">
            <mj-section padding="20px 0">
            <mj-column padding="0">
                { header().into() }
                <mj-text>
                    <p>{ Text::from(tl!(l, election_start.salute )).into() }</p>
                    <p>{ Text::from(tl!(l, election_start.opening_message )).into() }</p>
                </mj-text>

                <mj-wrapper mj-class="wrapper">
                    <mj-text>
                        <p>{ Text::from(tl!(l, election_start.candidate, candidate_name, candidate_url )).into() }</p>
                        <br />
                        <p>{ Text::from(tl!(l, election_start.proposer, proposer_name, proposer_url )).into() }</p>
                    </mj-text>
                </mj-wrapper>

                <mj-text>
                    <ul>
                        <li>{ Text::from(tl!(l, election_start.election_rules_1 )).into() }</li>
                        <li>{ Text::from(tl!(l, election_start.election_rules_2 )).into() }</li>
                        <li>{ Text::from(tl!(l, election_start.election_rules_3 )).into() }</li>
                    </ul>
                    <p>
                        <a href="https://musicbrainz.org/doc/Auto-Editor_Election">{ Text::from(tl!(l, election_start.election_rules_link_text )).into()}</a>
                    </p>
                    <p>
                        <a href="https://musicbrainz.org/doc/Editor#Auto-editor">{ Text::from(tl!(l, election_start.autoeditor_link_text )).into()}</a>
                    </p>
                </mj-text>

                <mj-text>
                    <p><strong>{ Text::from(tl!(l, election_start.participate )).into() }</strong></p>
                    <p>
                        <a href={election_url}>{ Text::from(encode_text(election_url)).into()}</a>
                    </p>
                </mj-text>
            </mj-column>
        </mj-section>
        </mj-body>
      </mjml>

    })
}
