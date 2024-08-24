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
    subscription_settings_url: String,
    edit_subscriptions_url: String,
    #[serde(default)]
    edits: EditTypes,
    #[serde(default)]
    deletes: Vec<DeleteItem>,
}

#[derive(Deserialize, Debug, Default)]
struct EditTypes {
    #[serde(default)]
    artist: Vec<SubItem>,
    #[serde(default)]
    collection: Vec<SubItem>,
    #[serde(default)]
    label: Vec<SubItem>,
    #[serde(default)]
    series: Vec<SubItem>,
    #[serde(default)]
    editor: Vec<SubItem>,
}

#[derive(Deserialize, Debug, Default)]
struct SubItem {
    entity_name: String,
    #[serde(default)]
    entity_comment: Option<String>,
    entity_url: String,
    open_size: u32,
    applied_size: u32,
    // open_url,
    // applied_url,
    // all_url
}

#[derive(Deserialize, Debug, Default)]
struct DeleteItem {
    item_type: String,
    entity_name: String,
    #[serde(default)]
    entity_comment: Option<String>,
    #[serde(default)]
    reason: Option<std::borrow::Cow<'static, str>>,
    #[serde(default)]
    edit_id: Option<u32>,
    #[serde(default)]
    edit_url: Option<String>,
}

// #[derive(Deserialize, Debug, Default)]
// #[serde(rename_all = "lowercase")]
// enum ItemType {
//     Artist,
//     Label,
//     Series,
//     Collection,
//     #[default]
//     Unknown,
// }

// impl ItemType {
//     fn to_string_l(&self, l: Locale) -> std::borrow::Cow<'static, str> {
//         match self {
//             ItemType::Artist => tl!(l, item_type.artist),
//             ItemType::Label => tl!(l, item_type.label),
//             ItemType::Series => tl!(l, item_type.series),
//             ItemType::Collection => tl!(l, item_type.collection),
//             ItemType::Unknown => tl!(l, item_type.unknown),
//         }
//     }
// }

pub(crate) fn subscription(params: Value, l: Locale) -> Result<Mjml, TemplateError> {
    let ctx: Option<Subscription> = serde_json::from_value(params)?;

    let Subscription {
        to_name,
        subscription_settings_url,
        edit_subscriptions_url,
        edits,
        deletes,
    } = ctx.unwrap_or_default();
    dbg!(&edits);
    let mut sections = view! {<></>};
    if !edits.artist.is_empty() {
        sections.children.push(edits_for_type_template(
            tl!(l, subscription.changes_sections.subscribed_artists),
            edits.artist,
            l,
        ))
    }
    if !edits.collection.is_empty() {
        sections.children.push(edits_for_type_template(
            tl!(l, subscription.changes_sections.subscribed_collections),
            edits.collection,
            l,
        ))
    }
    if !edits.label.is_empty() {
        sections.children.push(edits_for_type_template(
            tl!(l, subscription.changes_sections.subscribed_labels),
            edits.label,
            l,
        ))
    }
    if !edits.series.is_empty() {
        sections.children.push(edits_for_type_template(
            tl!(l, subscription.changes_sections.subscribed_series),
            edits.series,
            l,
        ))
    }
    if !edits.editor.is_empty() {
        sections.children.push(edits_for_type_template(
            tl!(l, subscription.changes_sections.subscribed_editors),
            edits.editor,
            l,
        ))
    }
    if !deletes.is_empty() {
        let mut list = view! {<ul></ul>};
        for DeleteItem {
            item_type,
            entity_name,
            entity_comment,
            reason,
            edit_id,
            edit_url,
        } in deletes
        {
            let formatted_name = if let Some(comment) = entity_comment {
                tl!(
                    l,
                    subscription.entity_with_comment,
                    name = entity_name,
                    comment
                )
            } else {
                tl!(l, subscription.entity, name = entity_name)
            };
            let reason = reason.unwrap_or(tl!(l, subscription.deleted_default_reason));
            let text = if let Some(edit_id) = edit_id {
                Text::from(tl!(
                    l,
                    subscription.deleted_item_with_edit,
                    item_type,
                    entity = formatted_name,
                    reason,
                    edit_id = edit_id.to_string()
                ))
            } else {
                Text::from(tl!(
                    l,
                    subscription.deleted_item,
                    item_type,
                    entity = formatted_name,
                    reason
                ))
            };
            let item: mrml::node::Node<mrml::mj_body::MjBodyChild> =
                if let Some(edit_url) = edit_url {
                    view! {
                        <li><a href={edit_url}>{ text.into() }</a></li>
                    }
                } else {
                    view! {
                        <li>{ text.into() }</li>
                    }
                };
            list.children.push(item.into());
        }
        sections.children.push(
            view! {
                <mj-wrapper mj-class="wrapper" >
                    <mj-text>
                        <h2 >{ Text::from(tl!(l, subscription.deleted_merged)).into() }</h2>
                        <p>{ Text::from(tl!(l, subscription.deleted_merged_info)).into() }</p>
                        {list.into()}
                    </mj-text>
                </mj-wrapper>
            }
            .into(),
        )
    }
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
                { sections.into() }
                <mj-text>
                    <h2>{ Text::from(tl!(l, subscription.open_edits )).into() }</h2>
                    <ul>
                        <li><a href="https://musicbrainz.org/edit/subscribed?open=1">{ Text::from(tl!(l, subscription.open_edits_subscribed_entities )).into() }</a></li>
                        <li><a href="https://musicbrainz.org/edit/subscribed_editors?open=1">{ Text::from(tl!(l, subscription.open_edits_subscribed_editors )).into() }</a></li>
                    </ul>
                </mj-text>
                <mj-text>
                    <p>{ Text::from(tl!(l, subscription.thanks)).into() }</p>
                    <p><em>{ Text::from(tl!(l, metabrainz_signoff)).into() }</em></p>
                </mj-text>
                <mj-divider padding="10px 15px" border-color="#F5F5F5" border-width="3px" />
                <mj-text font-size="12px" color="#8D8D8D">
                    <p>{ Text::from(tl!(l, subscription.about)).into() }</p>
                    <p>
                        <a href={subscription_settings_url}>{ Text::from(tl!(l, change_subscription_settings)).into() }</a>
                    </p>
                    <p>
                        <a href={edit_subscriptions_url}>{ Text::from(tl!(l, subscription.edit_subscriptions)).into() }</a>
                    </p>
                    <p>{ Text::from(tl!(l, do_not_reply)).into() }</p>
                </mj-text>

            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>

    })
}

fn edits_for_type_template(
    heading: std::borrow::Cow<'static, str>,
    items: Vec<SubItem>,
    l: Locale,
) -> mrml::mj_body::MjBodyChild {
    view! {
        <mj-wrapper mj-class="wrapper" >
            <mj-text>
                <h2 >{ Text::from(heading).into() }</h2>
                {item_list_template(items, l).into()}
            </mj-text>
        </mj-wrapper>
    }
    .into()
}

fn item_list_template(
    items: Vec<SubItem>,
    l: Locale,
) -> mrml::node::Node<mrml::mj_body::MjBodyChild> {
    let mut list = view! {<ul></ul>};
    for item in items {
        list.children.push(item_template(item, l).into());
    }
    list
}

fn item_template(item: SubItem, l: Locale) -> mrml::node::Node<mrml::mj_body::MjBodyChild> {
    let SubItem {
        ref entity_name,
        ref entity_comment,
        entity_url,
        open_size,
        applied_size,
    } = item;
    view! {
        <li><a href={entity_url}>{
            if let Some(comment) = entity_comment {
                Text::from(tl!(l, subscription.entity_with_comment , name = entity_name, comment)).into()
            } else {
                Text::from(tl!(l, subscription.entity , name = entity_name)).into()
            }}</a>" "
            { Text::from(tl!(l, subscription.open_applied_count , open = open_size.to_string(), applied = applied_size.to_string())).into() }</li>
    }
}
