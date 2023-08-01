use std::str::FromStr;

use super::winrate::WinRateError;
use crate::discord::{
    DiscordPayload, DiscordResponse, GameType, InteractionResponse, ResponseType,
};
use crate::error::Result;
use crate::AppState;
use riot_sdk::summoner::Region as SummonerRegion;

pub async fn run(body: &DiscordPayload, state: &AppState) -> Result<DiscordResponse> {
    let data = body.data.as_ref().ok_or(WinRateError::MissingData)?;
    let option = data.options.as_ref().ok_or(WinRateError::MissingOptions)?;
    let summoner_name = option
        .iter()
        .find(|o| o.name == "summoner")
        .ok_or(WinRateError::MissingSummonerOption)?
        .value
        .as_ref()
        .ok_or(WinRateError::MissingOptionValue)?;

    let game_type = option
        .iter()
        .find(|o| o.name == "game")
        .ok_or(WinRateError::MissingGameOption)?
        .value
        .as_ref()
        .ok_or(WinRateError::MissingOptionValue)?;
    let summoner_name = summoner_name.as_str().unwrap();

    let game_type = GameType::from_str(game_type.as_str().unwrap())?;

    match game_type {
        GameType::Tft => {
            let summoner_data = state
                .league_client
                .summoner(SummonerRegion::NA1)
                .get_by_name(summoner_name)
                .send()
                .await?;

            state
                .lp_db_client
                .tracking(lp_db::GameType::Tft)
                .untrack_user()
                .id(&summoner_data.id)
                .send()
                .await?;

            let banner = format!(
                "** --- TFT --- **\n\nTracking successfully removed for **{}**",
                summoner_data.name
            );

            let res = InteractionResponse::new(ResponseType::ChannelMessageWithSource, banner);
            Ok(res)
        }
        GameType::League => {
            let summoner_data = state
                .league_client
                .summoner(SummonerRegion::NA1)
                .get_by_name(summoner_name)
                .send()
                .await?;

            state
                .lp_db_client
                .tracking(lp_db::GameType::League)
                .untrack_user()
                .id(&summoner_data.id)
                .send()
                .await?;

            let banner = format!(
                "** --- League --- **\n\nTracking successfully removed for **{}**",
                summoner_data.name
            );

            let res = InteractionResponse::new(ResponseType::ChannelMessageWithSource, banner);
            Ok(res)
        }
    }
}
