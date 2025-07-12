use anyhow::Result;
use twilight_model::{
    http::interaction::InteractionResponse,
    id::{Id, marker::UserMarker},
};

use crate::{core::app_state::AppState, events::interaction_create::ConfirmChangeNickname};

pub async fn run(
    state: AppState,
    data: ConfirmChangeNickname,
    user_id: Id<UserMarker>,
) -> Result<InteractionResponse> {
    todo!()
}
