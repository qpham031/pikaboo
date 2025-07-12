use std::collections::HashMap;

use anyhow::{Result, anyhow};
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
        confirm_inventory_nickname, confirm_order, custom_role, custom_role_subscribe, inventory,
        inventory_nickname, menu, server_error_response, shop, shop_blind_box, shop_custom_role,
        shop_energy_boost, shop_nickname, unimplemented,
    },
};

pub async fn handle(state: AppState, mut interaction: Box<InteractionCreate>) -> Result<()> {
    let auth = InteractionAuth {
        application_id: interaction.application_id,
        interaction_id: interaction.id,
        interaction_token: std::mem::take(&mut interaction.0.token),
    };

    let user_id = interaction.author_id().unwrap();
    let interaction_item = InteractionItem::try_from(interaction.0)?;
    let state1 = state.clone();

    let response_rs = match interaction_item {
        InteractionItem::Menu => menu::run(state1),
        InteractionItem::Inventory => inventory::run(state1, user_id).await,
        InteractionItem::CustomRole | InteractionItem::InventoryCustomRole => {
            custom_role::run(state1, user_id)
        }
        InteractionItem::Shop => shop::run(),
        InteractionItem::ShopCustomRole => shop_custom_role::run(),
        InteractionItem::ShopNickname => shop_nickname::run(),
        InteractionItem::ShopEnergyBoost => shop_energy_boost::run(),
        InteractionItem::ShopBlindBox => shop_blind_box::run(),
        InteractionItem::InventoryNickname => inventory_nickname::run(),
        InteractionItem::ConfirmOrder(data) => confirm_order::run(data),
        InteractionItem::ConfirmInventoryNickname(data) => {
            confirm_inventory_nickname::run(state1, data, user_id).await
        }
        InteractionItem::CustomRoleSubscribe => {
            custom_role_subscribe::run(state.clone(), true).await
        }
        InteractionItem::CustomRoleUnsubscribe => {
            custom_role_subscribe::run(state.clone(), false).await
        }
        InteractionItem::Unimplemented => unimplemented::run(),
        InteractionItem::UnimplementedAbnormal => {
            Err(anyhow!("Encounter an unhandled abnormal interaction."))?
        }
    };

    let response = match response_rs {
        Ok(response) => response,
        Err(err) => {
            tracing::error!("Failed to handle interaction: {err}");
            server_error_response()
        }
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
    // View main menu which includes current config and some main options
    Menu,

    // View inventory
    Inventory,

    // Manage custom roles
    CustomRole,

    // View shop
    Shop,

    // Purchase custom role
    ShopCustomRole,

    // Purchase nickname
    ShopNickname,

    // Purchase energy boost
    ShopEnergyBoost,

    // Purchase blind box
    ShopBlindBox,

    // Confirm order
    ConfirmOrder(ConfirmOrder),

    // Set a new nickname and confirm trade
    ConfirmInventoryNickname(ConfirmChangeNickname),

    // Subscribe to custom role (enable auto-renew)
    CustomRoleSubscribe,

    // Unsubscribe from custom role (disable auto-renew)
    CustomRoleUnsubscribe,

    // Use nickname change from inventory
    InventoryNickname,

    // Manage custom roles (same as CustomRole, but accessed from inventory)
    InventoryCustomRole,

    // Unimplemented interactions
    Unimplemented,

    // Unimplemented abnormal interactions, such as Ping, Autocomplete, or other unknown types
    UnimplementedAbnormal,
}

#[derive(Debug, Clone, Copy)]
pub struct ConfirmOrder {
    pub okay: bool,
    pub item: OrderItem,
}

impl ConfirmOrder {
    pub fn new(msg: &str, item: OrderItem) -> ConfirmOrder {
        ConfirmOrder {
            okay: msg == "okay",
            item,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OrderItem {
    CustomRole,
    Nickname,
    EnergyBoost,
    BlindBox,
}

pub struct ConfirmChangeNickname {
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
                Ok(InteractionItem::UnimplementedAbnormal)
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
                consts::interact::CUSTOMROLE => InteractionItem::CustomRole,
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
            consts::interact::CUSTOMROLE => InteractionItem::CustomRole,
            consts::interact::CUSTOMROLE_SUBSCRIBE => InteractionItem::CustomRoleSubscribe,
            consts::interact::CUSTOMROLE_UNSUBSCRIBE => InteractionItem::CustomRoleUnsubscribe,
            _ => InteractionItem::Unimplemented,
        },
        ComponentType::TextSelectMenu => match data.custom_id.as_str() {
            consts::interact::SHOP => match data.values[0].as_str() {
                consts::interact::SHOP_CUSTOMROLE => InteractionItem::ShopCustomRole,
                consts::interact::SHOP_NICKNAME => InteractionItem::ShopNickname,
                consts::interact::SHOP_ENERGYBOOST => InteractionItem::ShopEnergyBoost,
                consts::interact::SHOP_BLINDBOX => InteractionItem::ShopBlindBox,
                _ => InteractionItem::Unimplemented,
            },
            consts::interact::INVENTORY => match data.values[0].as_str() {
                consts::interact::INVENTORY_CUSTOMROLE => InteractionItem::InventoryCustomRole,
                consts::interact::INVENTORY_NICKNAME => InteractionItem::InventoryNickname,
                _ => InteractionItem::Unimplemented,
            },
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
        cid @ (consts::interact::CONFIRM_ORDER_CUSTOMROLE
        | consts::interact::CONFIRM_ORDER_NICKNAME
        | consts::interact::CONFIRM_ORDER_ENERGYBOOST
        | consts::interact::CONFIRM_ORDER_BLINDBOX) => {
            let confirmation = inputs
                .remove(consts::interact::CONFIRM_OKAY)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Modal `{cid}` expects to have a `{}`",
                        consts::interact::CONFIRM_OKAY
                    )
                })?;

            InteractionItem::ConfirmOrder(ConfirmOrder::new(
                &confirmation,
                match cid {
                    consts::interact::CONFIRM_ORDER_CUSTOMROLE => OrderItem::CustomRole,
                    consts::interact::CONFIRM_ORDER_NICKNAME => OrderItem::Nickname,
                    consts::interact::CONFIRM_ORDER_ENERGYBOOST => OrderItem::EnergyBoost,
                    consts::interact::CONFIRM_ORDER_BLINDBOX => OrderItem::BlindBox,
                    _ => unreachable!(),
                },
            ))
        }
        cid @ consts::interact::CHANGE_NICKNAME => {
            let nickname = inputs.remove(consts::interact::NICKNAME).ok_or_else(|| {
                anyhow::anyhow!(
                    "Modal `{cid}` expects to have a `{}`",
                    consts::interact::NICKNAME
                )
            })?;

            let data = ConfirmChangeNickname { nickname };
            InteractionItem::ConfirmInventoryNickname(data)
        }
        _ => InteractionItem::Unimplemented,
    })
}
