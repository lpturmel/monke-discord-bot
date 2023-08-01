use std::str::FromStr;

use crate::discord::{
    DiscordPayload, DiscordResponse, GameType, InteractionResponse, ResponseType,
};
use crate::error::Result;
use crate::AppState;

use super::winrate::WinRateError;

pub async fn run(body: &DiscordPayload, state: &AppState) -> Result<DiscordResponse> {
    let data = body.data.as_ref().ok_or(WinRateError::MissingData)?;
    let option = data.options.as_ref().ok_or(WinRateError::MissingOptions)?;
    let game_type = option
        .iter()
        .find(|o| o.name == "game")
        .ok_or(WinRateError::MissingGameOption)?
        .value
        .as_ref()
        .ok_or(WinRateError::MissingOptionValue)?;
    let game_type = GameType::from_str(game_type.as_str().unwrap())?;

    match game_type {
        GameType::Tft => {
            let tracked_users = state
                .lp_db_client
                .tracking(lp_db::GameType::Tft)
                .list()
                .send()
                .await?;

            let mut banner = String::from("** --- TFT --- **\n\nTracked summoners:\n");

            for (i, user) in tracked_users.iter().enumerate() {
                banner.push_str(&format!("{}.\t{}\n", (i + 1), user.summoner_name));
            }

            let res = InteractionResponse::new(ResponseType::ChannelMessageWithSource, banner);
            Ok(res)
        }
        GameType::League => {
            let tracked_users = state
                .lp_db_client
                .tracking(lp_db::GameType::League)
                .list()
                .send()
                .await?;

            let mut banner = String::from("** --- League --- **\n\nTracked summoners:\n");

            for (i, user) in tracked_users.iter().enumerate() {
                banner.push_str(&format!("{}.\t{}\n", (i + 1), user.summoner_name));
            }

            let res = InteractionResponse::new(ResponseType::ChannelMessageWithSource, banner);
            Ok(res)
        }
    }
}
