use std::collections::HashMap;

use anyhow::Result;
use twilight_model::{
    application::interaction::{Interaction, InteractionData},
    gateway::payload::incoming::InteractionCreate,
    id::{
        Id,
        marker::{ApplicationMarker, InteractionMarker},
    },
};

use crate::{
    consts,
    core::app_state::AppState,
    interactions::{
        confirm_trade_custom_role, confirm_trade_nickname, custom_role, custom_role_subscribe,
        inventory, menu, shop, shop_custom_role, shop_nickname, trade_custom_role, trade_nickname,
        unimplemented, zones,
    },
};

pub async fn handle(state: AppState, mut interaction: Box<InteractionCreate>) -> Result<()> {
    let auth = InteractionAuth {
        application_id: interaction.application_id,
        interaction_id: interaction.id,
        interaction_token: std::mem::take(&mut interaction.0.token),
    };

    let interaction_item = InteractionItem::try_from(interaction.0)?;
    let state1 = state.clone();

    let response = match interaction_item {
        InteractionItem::Menu => menu::run(state1)?,
        InteractionItem::Inventory => inventory::run(state1).await?,
        InteractionItem::CustomRole => custom_role::run(),
        InteractionItem::Zones => zones::run(),
        InteractionItem::Shop => shop::run(),
        InteractionItem::ShopCustomRole => shop_custom_role::run(),
        InteractionItem::ShopNickname => shop_nickname::run(),
        InteractionItem::TradeCustomRole => trade_custom_role::run(),
        InteractionItem::TradeNickname => trade_nickname::run(),
        InteractionItem::ConfirmTradeCustomRole(data) => confirm_trade_custom_role::run(data),
        InteractionItem::ConfirmTradeNickname(data) => confirm_trade_nickname::run(data),
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
pub struct SimpleConfirmation(bool);

impl SimpleConfirmation {
    pub fn new(msg: &str) -> SimpleConfirmation {
        SimpleConfirmation(msg == "okay")
    }
    pub const fn okay(self) -> bool {
        self.0
    }
}

pub type ConfirmTradeCustomRole = SimpleConfirmation;
pub struct ConfirmTradeNickname {
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
        consts::interact::PIKABOO => InteractionItem::Menu,
        cmd @ consts::interact::PIKABOO_QUICK => {
            let Some(name) = data.options.first().map(|op| op.name.as_str()) else {
                return Err(anyhow::anyhow!("Subcommand is needed: {cmd}"));
            };
            match name {
                consts::interact::INVENTORY => InteractionItem::Inventory,
                consts::interact::SHOP => InteractionItem::Shop,
                consts::interact::CUSTOM_ROLE => InteractionItem::CustomRole,
                consts::interact::ZONES => InteractionItem::Zones,
                _ => {
                    return Err(anyhow::anyhow!("Unknown Subcommand: {cmd} {name}"));
                }
            }
        }
        consts::interact::PIKABOO_MOD | _ => InteractionItem::Unimplemented,
    })
}

fn component_extractor(interaction: Interaction) -> Result<InteractionItem> {
    use twilight_model::channel::message::component::ComponentType;

    let Some(InteractionData::MessageComponent(data)) = interaction.data else {
        return Err(anyhow::anyhow!("Component without data"));
    };

    Ok(match data.component_type {
        ComponentType::Button => match data.custom_id.as_str() {
            consts::interact::INVENTORY => InteractionItem::Inventory,
            consts::interact::SHOP => InteractionItem::Shop,
            consts::interact::CUSTOM_ROLE => InteractionItem::CustomRole,
            consts::interact::TRADE_CUSTOM_ROLE => InteractionItem::TradeCustomRole,
            consts::interact::TRADE_NICKNAME => InteractionItem::TradeNickname,
            consts::interact::CUSTOM_ROLE_SUBCRIBE => InteractionItem::CustomRoleSubscribe,
            consts::interact::CUSTOM_ROLE_UNSUBCRIBE => InteractionItem::CustomRoleUnsubcribe,
            _ => InteractionItem::Unimplemented,
        },
        ComponentType::TextSelectMenu => match data.custom_id.as_str() {
            consts::interact::SHOP_CUSTOM_ROLE => InteractionItem::ShopCustomRole,
            consts::interact::SHOP_NICKNAME => InteractionItem::ShopNickname,
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
        cid @ consts::interact::CONFIRM_TRADE_CUSTOM_ROLE => {
            let confirmation = inputs
                .remove(consts::interact::CONFIRM_OKAY)
                .ok_or_else(|| anyhow::anyhow!("Modal expects to have a data value: {cid}"))?;
            InteractionItem::ConfirmTradeCustomRole(ConfirmTradeCustomRole::new(&confirmation))
        }
        cid @ consts::interact::CONFIRM_TRADE_NICKNAME => {
            let nickname = inputs
                .remove(consts::interact::NICKNAME)
                .ok_or_else(|| anyhow::anyhow!("Modal expects to have a data value: {cid}"))?;
            InteractionItem::ConfirmTradeNickname(ConfirmTradeNickname { nickname })
        }
        _ => InteractionItem::Unimplemented,
    })
}
