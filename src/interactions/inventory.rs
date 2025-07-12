use std::{fmt::Write, ops::Not};

use anyhow::Result;
use twilight_mention::Mention;
use twilight_model::{
    channel::message::{
        Component, EmojiReactionType,
        component::{ActionRow, SelectMenu, SelectMenuOption, SelectMenuType},
    },
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::{Id, marker::UserMarker},
};

use crate::{consts, core::app_state::AppState};

pub async fn run(state: AppState, user_id: Id<UserMarker>) -> Result<InteractionResponse> {
    let mut content = "# Inventory\n".to_string();
    let mut options = vec![];

    // Energy balance
    let energy = state.cache.energy_balance.get(user_id).await?;
    let _ = writeln!(&mut content, "**Energy:** {energy} ⚡");

    // Custom roles
    if let Some(role) = state.cache.user_custom_roles.get(user_id) {
        let _ = writeln!(&mut content, "**Custom Role:** {}", role.role_id.mention());
        options.push(SelectMenuOption {
            default: false,
            description: Some("Go to Custom Role interact".to_string()),
            emoji: Some(EmojiReactionType::Unicode {
                name: '🎀'.to_string(),
            }),
            label: "Custom Role".to_string(),
            value: consts::interact::INVENTORY_CUSTOMROLE.to_string(),
        });
    }

    // Build select menu
    let components = options.is_empty().not().then(|| {
        let components = vec![Component::SelectMenu(SelectMenu {
            channel_types: None,
            custom_id: consts::interact::INVENTORY.to_string(),
            default_values: None,
            disabled: false,
            kind: SelectMenuType::Text,
            max_values: None,
            min_values: None,
            options: Some(options),
            placeholder: Some("Select an item to use".to_string()),
        })];
        vec![Component::ActionRow(ActionRow { components })]
    });

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some(content),
            components,
            ..Default::default()
        }),
    })
}
