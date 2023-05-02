use crate::db::GameItem;
use crate::discord::{DiscordPayload, DiscordResponse, InteractionResponse, ResponseType};
use crate::error::Result;
use crate::riot::matches::Region as MatchesRegion;
use crate::riot::summoner::Region as SummonerRegion;
use crate::riot::Queue;
use crate::AppState;
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, PutRequest, WriteRequest};
use serde_dynamo::aws_sdk_dynamodb_0_25::{from_items, to_item};
use std::collections::HashMap;
use std::env;
use std::fmt::Display;

#[derive(Debug)]
pub enum WinRateError {
    SummonerNotFound,
    MissingSummonerOption,
    MissingData,
    MissingOptions,
    MissingOptionValue,
    GetItemNoTableResults,
    GetItemNoResults,
}

impl Display for WinRateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            WinRateError::SummonerNotFound => "Summoner not found",
            WinRateError::MissingSummonerOption => "Missing required summoner option",
            WinRateError::MissingData => "Missing data (from discord)",
            WinRateError::MissingOptions => "Missing options (from discord)",
            WinRateError::MissingOptionValue => "Missing option value (from discord)",
            WinRateError::GetItemNoTableResults => "No table results from dynamodb",
            WinRateError::GetItemNoResults => "No results from dynamodb",
        };
        write!(f, "{}", msg)
    }
}
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
    let summoner_name = summoner_name.as_str().unwrap();

    let summoner_data = state
        .riot_client
        .summoner(SummonerRegion::NA1)
        .get_by_name(&summoner_name)
        .send()
        .await?;

    let queue_type = Queue::RankedSolo5x5;
    let queue_type_str = queue_type.to_string();

    let game_ids = state
        .riot_client
        .matches(MatchesRegion::AMERICAS)
        .get_ids(&summoner_data.puuid)
        .count(10)
        .queue(queue_type)
        .send()
        .await?;

    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME env var not set");
    let keys = game_ids
        .iter()
        .map(|game_id| {
            let mut key = HashMap::new();
            key.insert("id".to_string(), AttributeValue::S(game_id.clone()));
            key.insert("sk".to_string(), AttributeValue::S("#".to_string()));
            key
        })
        .collect::<Vec<_>>();

    let items = KeysAndAttributes::builder().set_keys(Some(keys)).build();

    let batch_get_res = state
        .db_client
        .batch_get_item()
        .request_items(&table_name, items)
        .send()
        .await?;

    let table_res = batch_get_res
        .responses
        .ok_or(WinRateError::GetItemNoTableResults)?;
    let items = table_res
        .get(&table_name)
        .ok_or(WinRateError::GetItemNoResults)?;
    let items: Vec<GameItem> = from_items(items.clone())?;

    let mut game_details: Vec<GameItem> = Vec::new();
    let missing_game_ids = game_ids
        .iter()
        .filter(
            |game_id| match items.iter().find(|item| item.id == **game_id) {
                Some(i) => {
                    game_details.push(i.clone());
                    false
                }
                None => true,
            },
        )
        .collect::<Vec<_>>();

    let game_details_fut = missing_game_ids
        .iter()
        .map(|game_id| {
            state
                .riot_client
                .matches(MatchesRegion::AMERICAS)
                .get_details(game_id)
                .send()
        })
        .collect::<Vec<_>>();

    println!("Fetching {} games from Riot API", game_details_fut.len());
    let game_details_res = futures::future::join_all(game_details_fut).await;
    let game_details_res = game_details_res
        .iter()
        .filter_map(|res| res.as_ref().ok())
        .collect::<Vec<_>>();

    if game_details_res.len() >= 1 {
        let put_items = game_details_res
            .iter()
            .map(|game| {
                WriteRequest::builder()
                    .put_request(
                        PutRequest::builder()
                            .set_item(to_item::<GameItem>((*game).clone().into()).ok())
                            .build(),
                    )
                    .build()
            })
            .collect::<Vec<_>>();

        state
            .db_client
            .batch_write_item()
            .request_items(&table_name, put_items)
            .send()
            .await?;
    }

    game_details.extend(
        game_details_res
            .iter()
            .map(|game| (*game).clone().into())
            .collect::<Vec<GameItem>>(),
    );

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
