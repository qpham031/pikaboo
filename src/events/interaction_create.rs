use std::collections::HashMap;

use anyhow::Result;
use twilight_model::{
    application::interaction::{Interaction, InteractionData},
    gateway::payload::incoming::InteractionCreate,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::{
        Id,
        marker::{ApplicationMarker, InteractionMarker},
    },
};

use crate::{
    core::app_state::AppState,
    interactions::{
        confirm_trade_custom_role, confirm_trade_nickname, custom_role, custom_role_subscribe,
        inventory, menu, shop, shop_custom_role, shop_nickname, trade_custom_role, trade_nickname,
        unimplemented, zones,
    },
};

pub async fn handle(mut interaction: Box<InteractionCreate>, state: AppState) -> Result<()> {
    let auth = InteractionAuth {
        application_id: interaction.application_id,
        interaction_id: interaction.id,
        interaction_token: std::mem::take(&mut interaction.0.token),
    };

    let interaction_item = InteractionItem::try_from(interaction.0)?;

    let response = match interaction_item {
        InteractionItem::Menu => menu::run(),
        InteractionItem::Inventory => inventory::run(),
        InteractionItem::CustomRole => custom_role::run(),
        InteractionItem::Zones => zones::run(),
        InteractionItem::Shop => shop::run(),
        InteractionItem::ShopCustomRole => shop_custom_role::run(),
        InteractionItem::ShopNickname => shop_nickname::run(),
        InteractionItem::TradeCustomRole => trade_custom_role::run(),
        InteractionItem::TradeNickname => trade_nickname::run(),
        InteractionItem::ConfirmTradeCustomRole(data) => confirm_trade_custom_role::run(),
        InteractionItem::ConfirmTradeNickname(data) => confirm_trade_nickname::run(),
        InteractionItem::CustomRoleSubscribe => {
            custom_role_subscribe::run(state.clone(), true).await?
        }
        InteractionItem::CustomRoleUnsubcribe => {
            custom_role_subscribe::run(state.clone(), false).await?
        }
        InteractionItem::Unimplemented => unimplemented::run(),
    };

    state
        .app
        .interaction(auth.application_id)
        .create_response(auth.interaction_id, &auth.interaction_token, &response)
        .await?;
    Ok(())
}

pub struct InteractionAuth {
    pub application_id: Id<ApplicationMarker>,
    pub interaction_id: Id<InteractionMarker>,
    pub interaction_token: String,
}

enum InteractionItem {
    Menu,
    Inventory,
    CustomRole,
    Zones,
    Shop,
    ShopCustomRole,
    ShopNickname,
    TradeCustomRole,
    TradeNickname,
    ConfirmTradeCustomRole(ConfirmTradeCustomRole),
    ConfirmTradeNickname(ConfirmTradeNickname),
    CustomRoleSubscribe,
    CustomRoleUnsubcribe,
    Unimplemented,
}

#[derive(Debug, Clone, Copy)]
struct SimpleConfirmation(bool);

impl SimpleConfirmation {
    pub fn new(msg: &str) -> SimpleConfirmation {
        SimpleConfirmation(msg == "okay")
    }
    pub const fn okay(self) -> bool {
        self.0
    }
}

type ConfirmTradeCustomRole = SimpleConfirmation;
struct ConfirmTradeNickname {
    pub nickname: String,
}

impl TryFrom<Interaction> for InteractionItem {
    type Error = anyhow::Error;

    fn try_from(interaction: Interaction) -> std::result::Result<Self, Self::Error> {
        use twilight_model::application::interaction::InteractionType;

        match interaction.kind {
            InteractionType::ApplicationCommand => Ok(command_extractor(interaction)?),
            InteractionType::MessageComponent => Ok(component_extractor(interaction)?),
            InteractionType::ModalSubmit => Ok(modal_extractor(interaction)?),
            InteractionType::ApplicationCommandAutocomplete | InteractionType::Ping | _ => {
                Err(anyhow::anyhow!("Encounter an unhandled interaction."))
            }
        }
    }
}

fn command_extractor(interaction: Interaction) -> Result<InteractionItem> {
    let Some(InteractionData::ApplicationCommand(data)) = interaction.data else {
        return Err(anyhow::anyhow!("Command without data"));
    };
    Ok(match data.name.as_str() {
        "pikaboo" => InteractionItem::Menu,
        "pikaboo-quick" => {
            let Some(name) = data.options.first().map(|op| op.name.as_str()) else {
                return Err(anyhow::anyhow!("Subcommand is needed: {}", "pikaboo-quick"));
            };
            match name {
                "inventory" => InteractionItem::Inventory,
                "shop" => InteractionItem::Shop,
                "customrole" => InteractionItem::CustomRole,
                "zones" => InteractionItem::Zones,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unknown Subcommand: {} {name}",
                        "pikaboo-quick"
                    ));
                }
            }
        }
        "pikaboo-mod" | _ => InteractionItem::Unimplemented,
    })
}

fn component_extractor(interaction: Interaction) -> Result<InteractionItem> {
    use twilight_model::channel::message::component::ComponentType;

    let Some(InteractionData::MessageComponent(data)) = interaction.data else {
        return Err(anyhow::anyhow!("Component without data"));
    };

    Ok(match data.component_type {
        ComponentType::Button => match data.custom_id.as_str() {
            "inventory" => InteractionItem::Inventory,
            "shop" => InteractionItem::Shop,
            "customrole" => InteractionItem::CustomRole,
            "tradecustomrole" => InteractionItem::TradeCustomRole,
            "tradenickname" => InteractionItem::TradeNickname,
            "customrolesubscribe" => InteractionItem::CustomRoleSubscribe,
            "customroleunsubscribe" => InteractionItem::CustomRoleUnsubcribe,
            _ => InteractionItem::Unimplemented,
        },
        ComponentType::TextSelectMenu => match data.custom_id.as_str() {
            "shopcustomrole" => InteractionItem::ShopCustomRole,
            "shopnickname" => InteractionItem::ShopNickname,
            _ => InteractionItem::Unimplemented,
        },
        ComponentType::ActionRow
        | ComponentType::TextInput
        | ComponentType::UserSelectMenu
        | ComponentType::RoleSelectMenu
        | ComponentType::MentionableSelectMenu
        | ComponentType::ChannelSelectMenu
        | ComponentType::Unknown(_)
        | _ => InteractionItem::Unimplemented,
    })
}

fn modal_extractor(interaction: Interaction) -> Result<InteractionItem> {
    let Some(InteractionData::ModalSubmit(data)) = interaction.data else {
        return Err(anyhow::anyhow!("Modal without data"));
    };

    let mut inputs = data
        .components
        .into_iter()
        .filter_map(|mut ar| ar.components.pop())
        .filter_map(|item| Some((item.custom_id, item.value?)))
        .collect::<HashMap<_, _>>();

    Ok(match data.custom_id.as_str() {
        "confirmtradecustomrole" => {
            let confirmation = inputs
                .values()
                .next()
                .map(|text| text.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Modal expects to have a data value: {}",
                        "confirmtradecustomrole"
                    )
                })?;
            InteractionItem::ConfirmTradeCustomRole(ConfirmTradeCustomRole::new(confirmation))
        }
        "confirmtradenickname" => {
            let nickname = inputs.remove("nickname").ok_or_else(|| {
                anyhow::anyhow!(
                    "Modal expects to have a data value: {}",
                    "confirmtradenickname"
                )
            })?;
            InteractionItem::ConfirmTradeNickname(ConfirmTradeNickname { nickname })
        }
        _ => InteractionItem::Unimplemented,
    })
}
