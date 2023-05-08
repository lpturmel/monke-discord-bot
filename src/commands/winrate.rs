use crate::db::GameItem;
use crate::discord::{DiscordPayload, DiscordResponse, InteractionResponse, ResponseType};
use crate::error::Result;
use crate::riot::matches::details::Participant;
use crate::riot::matches::Region as MatchesRegion;
use crate::riot::summoner::Region as SummonerRegion;
use crate::riot::Queue;
use crate::AppState;
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, PutRequest, WriteRequest};
use serde_dynamo::aws_sdk_dynamodb_0_25::{from_items, to_item};
use std::collections::HashMap;
use std::env;
use std::fmt::Display;

const WIN: &'static str = "✅";
const LOSS: &'static str = "❌";

#[derive(Debug)]
pub enum WinRateError {
    SummonerNotFound,
    MissingSummonerOption,
    MissingData,
    MissingOptions,
    MissingOptionValue,
    GetItemNoTableResults,
    GetItemNoResults,
    SummonerNotPartOfGame,
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
            WinRateError::SummonerNotPartOfGame => "Summoner not found in game participants",
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
            key.insert("id".to_string(), AttributeValue::S(game_id.to_string()));
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
            .into_iter()
            .map(|m| GameItem::from_match_details(m)),
    );

    game_details.sort_by(|a, b| b.info.game_creation.cmp(&a.info.game_creation));

    let game_count = game_details.len();

    let user_games = game_details.iter().map(|game| {
        game.info
            .participants
            .iter()
            .find(|p| p.summoner_id == summoner_data.id)
            .ok_or(WinRateError::SummonerNotPartOfGame)
    });
    let user_games = user_games.collect::<std::result::Result<Vec<_>, WinRateError>>()?;

    let won_games = user_games.iter().filter(|p| p.win).count();

    let winrate = won_games as f32 / game_count as f32 * 100.0;

    let mut game_lines = user_games
        .iter()
        .map(|p| print_game_line(&p.champion_name, p.kills, p.deaths, p.assists, p.win))
        .collect::<String>();

    if summoner_name.to_lowercase() == "xrayzor" {
        game_lines.push_str(&rayan_kayn_kda_str(&user_games));
    }
    let res = InteractionResponse::new(
        ResponseType::ChannelMessageWithSource,
        format!(
            "**{}** [{}]: {:.2}% in last {} games\n\n{}",
            summoner_data.name, queue_type_str, winrate, game_count, game_lines
        ),
    );
    Ok(res)
}
fn rayan_kayn_kda_str(games: &Vec<&Participant>) -> String {
    let kayn_games = games
        .iter()
        .filter(|p| p.champion_name.to_lowercase() == "kayn")
        .collect::<Vec<_>>();
    let won_games = kayn_games.iter().filter(|p| p.win).count();

    let winrate = won_games as f32 / kayn_games.len() as f32 * 100.0;

    let avg_kills = kayn_games.iter().map(|p| p.kills).sum::<i64>() / kayn_games.len() as i64;
    let avg_deaths = kayn_games.iter().map(|p| p.deaths).sum::<i64>() / kayn_games.len() as i64;
    let avg_assists = kayn_games.iter().map(|p| p.assists).sum::<i64>() / kayn_games.len() as i64;
    let average_kda = kayn_games
        .iter()
        .map(|p| get_numeric_kda(p.kills, p.deaths, p.assists))
        .sum::<f32>()
        / kayn_games.len() as f32;

    format!(
        "\n---IMPORTANT ---\n\n**Rayan** Kayn: {}% winrate in {} games with {:.2} ({:.2}/{:.2}/{:.2}) KDA",
        winrate,
        kayn_games.len(),
        average_kda,
        avg_kills,
        avg_deaths,
        avg_assists
    )
}

pub fn print_game_line(
    champion_name: &str,
    kills: i64,
    deaths: i64,
    assists: i64,
    win: bool,
) -> String {
    let win_str = if win { WIN } else { LOSS };
    let kda_num = get_numeric_kda(kills, deaths, assists);
    let kda_str = match kda_num {
        x if x == f32::INFINITY => "Perfect".to_string(),
        _ => format!("{:.2}", kda_num),
    };
    format!(
        "\n{} - {} {}/{}/{} **{}** KDA\n",
        win_str, champion_name, kills, deaths, assists, kda_str
    )
}

fn get_numeric_kda(kills: i64, deaths: i64, assists: i64) -> f32 {
    if deaths == 0 {
        f32::INFINITY
    } else {
        (kills + assists) as f32 / deaths as f32
    }
}
