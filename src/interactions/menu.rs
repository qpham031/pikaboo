use std::fmt::Write;

use anyhow::Result;
use twilight_mention::Mention;
use twilight_model::{
    channel::message::{
        Component, EmojiReactionType, MessageFlags,
        component::{ActionRow, Button, ButtonStyle},
        embed::EmbedField,
    },
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};
use twilight_util::builder::embed::EmbedBuilder;

use crate::{consts, core::app_state::AppState};

pub fn run(state: AppState) -> Result<InteractionResponse> {
    let config = state.config.read().unwrap();

    // Cooldown field
    let cooldown = format!("{} seconds", config.cooldown);
    let cooldown_field = EmbedField {
        inline: true,
        name: "Cooldown".to_string(),
        value: cooldown,
    };

    // Zones field
    let mut zones = String::new();
    config.zones.iter().for_each(|zone| {
        let _ = write!(&mut zones, "{}", zone.mention());
    });

    let zone_field = EmbedField {
        inline: true,
        name: "Zones".to_string(),
        value: zones,
    };

    // Build the embed
    let title = "Power Up Your Server! ⚡".to_string();
    let description = "Automatically generate Server Energy [⚡] just by being active in the zones! Think of it like an endless digital power plant, fueled by your activity. This renewable energy is all yours to unlock exclusive perks and cool custom rewards!".to_string();
    let color = consts::colors::MENU_COLOR;
    let embed = EmbedBuilder::new()
        .title(title)
        .description(description)
        .field(zone_field)
        .field(cooldown_field)
        .color(color)
        .build();
    let embeds = vec![embed];

    // Build components
    #[rustfmt::skip]
    const MENU_ITEMS: [(&str, &str, ButtonStyle, char); 5] = [
        ("Inventory", consts::interact::INVENTORY, ButtonStyle::Primary, '📦'),
        ("Shop", consts::interact::SHOP, ButtonStyle::Primary, '🛒'),
        ("Games", consts::interact::GAMES, ButtonStyle::Primary, '🎮'),
        ("About", consts::interact::ABOUT, ButtonStyle::Secondary, '📙'),
        ("FAQs", consts::interact::FAQS, ButtonStyle::Secondary, '❓'),
    ];
    let components = Vec::from(MENU_ITEMS.map(|(label, custom_id, style, emoji)| {
        Component::Button(Button {
            custom_id: Some(custom_id.to_string()),
            disabled: false,
            emoji: Some(EmojiReactionType::Unicode {
                name: emoji.to_string(),
            }),
            label: Some(label.to_string()),
            style,
            url: None,
            sku_id: None,
        })
    }));
    let components = vec![Component::ActionRow(ActionRow { components })];

    // Build the response
    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            embeds: Some(embeds),
            components: Some(components),
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    };

    Ok(response)
}
