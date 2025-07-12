use twilight_model::{
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

pub mod confirm_inventory_nickname;

pub mod confirm_order;
pub mod custom_role;
pub mod custom_role_subscribe;
pub mod inventory;
pub mod inventory_nickname;
pub mod menu;
pub mod shop;
pub mod shop_blind_box;
pub mod shop_custom_role;
pub mod shop_energy_boost;
pub mod shop_nickname;
pub mod unimplemented;
pub mod zones;

pub fn server_error_response() -> InteractionResponse {
    InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some("Oops! Something went wrong.".to_string()),
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    }
}
