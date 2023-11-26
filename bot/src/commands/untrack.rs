use std::str::FromStr;

use super::winrate::WinRateError;
use crate::discord::{
    DiscordPayload, DiscordResponse, GameType, InteractionResponse, ResponseType,
};
use crate::error::Result;
use crate::AppState;
use riot_sdk::account::AccountRegion;
use riot_sdk::summoner::Region as SummonerRegion;

pub async fn run(body: &DiscordPayload, state: &AppState) -> Result<DiscordResponse> {
    let data = body.data.as_ref().ok_or(WinRateError::MissingData)?;
    let option = data.options.as_ref().ok_or(WinRateError::MissingOptions)?;
    let game_name = option
        .iter()
        .find(|o| o.name == "game_name")
        .ok_or(WinRateError::MissingGameNameOption)?
        .value
        .as_ref()
        .ok_or(WinRateError::MissingOptionValue)?;
    let tag_line = option
        .iter()
        .find(|o| o.name == "tag_line")
        .ok_or(WinRateError::MissingTagLineOption)?
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
    let tag_line = tag_line.as_str().unwrap();
    let game_name = game_name.as_str().unwrap();

    let game_type = GameType::from_str(game_type.as_str().unwrap())?;

    let riot_id_data = state
        .account_client
        .account(AccountRegion::AMERICAS)
        .get_by_riot_id(game_name, tag_line)
        .send()
        .await
        .map_err(|_| WinRateError::RiotIdNotFound)?;
    let riot_id = format!("{}#{}", riot_id_data.game_name, riot_id_data.tag_line);
    match game_type {
        GameType::Tft => {
            let summoner_data = state
                .league_client
                .summoner(SummonerRegion::NA1)
                .get_by_puuid(&riot_id_data.puuid)
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
                riot_id
            );

            let res = InteractionResponse::new(ResponseType::ChannelMessageWithSource, banner);
            Ok(res)
        }
        GameType::League => {
            let summoner_data = state
                .league_client
                .summoner(SummonerRegion::NA1)
                .get_by_puuid(&riot_id_data.puuid)
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
                riot_id
            );

            let res = InteractionResponse::new(ResponseType::ChannelMessageWithSource, banner);
            Ok(res)
        }
    }
}
