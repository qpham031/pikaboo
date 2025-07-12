use anyhow::Result;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};

pub fn run() -> Result<InteractionResponse> {
    Ok(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some("Unimplemented interaction".to_string()),
            ..Default::default()
        }),
    })
}
