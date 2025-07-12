use anyhow::Result;
use twilight_model::{
    channel::message::{
        Component, EmojiReactionType, MessageFlags,
        component::{ActionRow, SelectMenu, SelectMenuOption, SelectMenuType},
    },
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

use crate::consts;

pub fn run() -> Result<InteractionResponse> {
    let content = "Welcome to the shop! Here you can purchase items, roles, and more.".to_string();

    #[rustfmt::skip]
    const SHOP_ITEMS: [( &str, &str, &str, char); 4] = [
        (consts::interact::SHOP_CUSTOMROLE, "Custom Role", "Get an exclusive custom role!", '🎀'),
        (consts::interact::SHOP_NICKNAME, "Nickname Change", "Change your nickname in the server!", '📝'),
        (consts::interact::SHOP_ENERGYBOOST, "Energy Boost", "Boost your energy for more actions!", '⚡'),
        (consts::interact::SHOP_BLINDBOX, "Blind Box", "A surprise item that can help you!", '❓'),
    ];

    #[rustfmt::skip]
    let select_menu_options = Vec::from(
        SHOP_ITEMS
            .map(|(value, label, description, emoji)| SelectMenuOption {
                label: label.to_string(),
                value: value.to_string(),
                description: Some(description.to_string()),
                emoji: Some(EmojiReactionType::Unicode {
                    name: emoji.to_string(),
                }),
                default: false,
            }));

    let select_menu = Component::SelectMenu(SelectMenu {
        channel_types: None,
        custom_id: consts::interact::SHOP.to_string(),
        default_values: None,
        disabled: false,
        kind: SelectMenuType::Text,
        max_values: None,
        min_values: None,
        options: Some(select_menu_options),
        placeholder: None,
    });

    let components = vec![Component::ActionRow(ActionRow {
        components: vec![select_menu],
    })];

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some(content),
            flags: Some(MessageFlags::EPHEMERAL),
            components: Some(components),
            ..Default::default()
        }),
    })
}
