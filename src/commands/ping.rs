use crate::discord::{DiscordPayload, DiscordResponse, InteractionResponse, ResponseType};
use crate::error::Result;

pub async fn run(_body: &DiscordPayload) -> Result<DiscordResponse> {
    let res = InteractionResponse::new(ResponseType::Pong, "pong");
    Ok(res)
}
