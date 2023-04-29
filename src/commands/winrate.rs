use crate::discord::{DiscordPayload, DiscordResponse, InteractionResponse, ResponseType};
use crate::error::Result;
use crate::riot::matches::Region as MatchesRegion;
use crate::riot::summoner::Region as SummonerRegion;
use crate::riot::{Client, Queue};
use std::fmt::Display;

#[derive(Debug)]
pub enum WinRateError {
    SummonerNotFound,
    MissingSummonerOption,
    MissingData,
    MissingOptions,
    MissingOptionValue,
}

impl Display for WinRateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            WinRateError::SummonerNotFound => "Summoner not found",
            WinRateError::MissingSummonerOption => "Missing required summoner option",
            WinRateError::MissingData => "Missing data (from discord)",
            WinRateError::MissingOptions => "Missing options (from discord)",
            WinRateError::MissingOptionValue => "Missing option value (from discord)",
        };
        write!(f, "{}", msg)
    }
}
pub async fn run(body: &DiscordPayload, riot_client: &Client) -> Result<DiscordResponse> {
    let data = body.data.as_ref().ok_or(WinRateError::MissingData)?;
    let option = data.options.as_ref().ok_or(WinRateError::MissingOptions)?;
    let summoner_name = option
        .iter()
        .find(|o| o.name == "summoner")
        .ok_or(WinRateError::MissingSummonerOption)?
        .value
        .as_ref()
        .ok_or(WinRateError::MissingOptionValue)?;
    let summoner_name = summoner_name.as_str().unwrap();

    // TODO implement dynamodb cache fist (redis like) if user is in db skip riot api call
    let summoner_data = riot_client
        .summoner(SummonerRegion::NA1)
        .get_by_name(&summoner_name)
        .send()
        .await?;

    let queue_type = Queue::RankedSolo5x5;
    let queue_type_str = queue_type.to_string();

    // TODO implement dynamodb cache fist (redis like) if game is in db skip riot api call
    let game_ids = riot_client
        .matches(MatchesRegion::AMERICAS)
        .get_ids(&summoner_data.puuid)
        .count(10)
        .queue(queue_type)
        .send()
        .await?;

    let game_details_fut = game_ids
        .iter()
        .map(|game_id| {
            riot_client
                .matches(MatchesRegion::AMERICAS)
                .get_details(game_id)
                .send()
        })
        .collect::<Vec<_>>();

    let game_details = futures::future::join_all(game_details_fut).await;

    let game_details = game_details
        .iter()
        .filter_map(|game| game.as_ref().ok())
        .collect::<Vec<_>>();

    let game_count = game_details.len();

    let won_games = game_details
        .iter()
        .filter(|game| {
            game.info
                .participants
                .iter()
                .find(|p| p.summoner_id == summoner_data.id)
                .unwrap()
                .win
        })
        .count();

    let winrate = won_games as f32 / game_count as f32 * 100.0;

    let res = InteractionResponse::new(
        ResponseType::ChannelMessageWithSource,
        format!(
            "**{}** [{}]: {:.2}% in last {} games",
            summoner_data.name, queue_type_str, winrate, game_count
        ),
    );
    Ok(res)
}
