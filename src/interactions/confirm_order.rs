use anyhow::Result;
use twilight_model::http::interaction::InteractionResponse;

use crate::events::interaction_create::ConfirmOrder;

pub fn run(data: ConfirmOrder) -> Result<InteractionResponse> {
    todo!()
}
