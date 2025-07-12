use anyhow::Result;
use twilight_model::{
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::{Id, marker::UserMarker},
};
use twilight_util::builder::embed::{EmbedBuilder, ImageSource};

use crate::core::{app_state::AppState, database::RoleIcon};

pub fn run(state: AppState, user_id: Id<UserMarker>) -> Result<InteractionResponse> {
    let role = state
        .cache
        .user_custom_roles
        .lock()
        .unwrap()
        .get(&user_id)
        .cloned();

    let Some(role) = role else {
        return lack_of_custom_role();
    };

    let mut embed_builder = EmbedBuilder::new()
        .title("Custom Role Info")
        .description(role.to_string())
        .color(role.color);

    if let RoleIcon::Custom(hash) = role.icon {
        let image_url = ImageSource::url(format!(
            "https://cdn.discordapp.com/role-icons/{}/{}.png",
            role.role_id, hash
        ))
        .unwrap();

        embed_builder = embed_builder.thumbnail(image_url);
    }
    let embed = embed_builder.build();

    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            embeds: Some(vec![embed]),
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    })
}

pub fn lack_of_custom_role() -> Result<InteractionResponse> {
    todo!()
}
