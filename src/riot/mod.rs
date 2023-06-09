pub mod matches;
pub mod summoner;

use matches::Region as MatchRegion;
use reqwest::header::HeaderMap;
use std::fmt::Display;
use std::sync::Arc;
use summoner::Region as SummonerRegion;

#[derive(Debug)]
pub enum Error {
    HttpError(reqwest::Error),
    SummonerNotFound,
    TooManyRequests,
    Forbidden,
    Unauthorized,
    RiotError,
    BadRequest,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::HttpError(e) => return e.fmt(f),
            Error::SummonerNotFound => "Summoner not found",
            Error::TooManyRequests => "Too many requests",
            Error::Forbidden => "API key is invalid",
            Error::Unauthorized => "Unauthorized",
            Error::RiotError => "Riot API error",
            Error::BadRequest => "Bad request to Riot API (likely an error on their end)",
        };
        write!(f, "{}", msg)
    }
}
#[derive(Debug, Clone)]
pub enum Queue {
    RankedSolo5x5,
    RankedFlex5x5,
}
impl Display for Queue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let queue = match self {
            Queue::RankedSolo5x5 => "RANKED_SOLO",
            Queue::RankedFlex5x5 => "RANKED_FLEX",
        };
        write!(f, "{}", queue)
    }
}

impl From<Queue> for i64 {
    fn from(queue: Queue) -> Self {
        match queue {
            Queue::RankedSolo5x5 => 420,
            Queue::RankedFlex5x5 => 440,
        }
    }
}
// impl Into<i64> for Queue {
//     fn into(self) -> i64 {
//         match self {
//             Queue::RankedSolo5x5 => 420,
//             Queue::RankedFlex5x5 => 440,
//         }
//     }
// }
impl From<&Queue> for GameType {
    fn from(queue: &Queue) -> Self {
        match queue {
            Queue::RankedSolo5x5 => GameType::Ranked,
            Queue::RankedFlex5x5 => GameType::Ranked,
        }
    }
}

impl From<Queue> for GameType {
    fn from(queue: Queue) -> Self {
        match queue {
            Queue::RankedSolo5x5 => GameType::Ranked,
            Queue::RankedFlex5x5 => GameType::Ranked,
        }
    }
}

pub enum GameType {
    Ranked,
    Normal,
    Tourney,
    Tutorial,
}

impl Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let game_type = match self {
            GameType::Ranked => "ranked",
            GameType::Normal => "normal",
            GameType::Tourney => "tourney",
            GameType::Tutorial => "tutorial",
        };
        write!(f, "{}", game_type)
    }
}
#[derive(Debug)]
pub struct Handle {
    pub web: reqwest::Client,
}
pub struct Client {
    handle: Arc<Handle>,
}

impl Client {
    pub fn new(api_key: &str) -> Self {
        let mut shared_headers = HeaderMap::new();
        shared_headers.insert(
            "X-Riot-Token",
            api_key.parse().expect("Invalid API key format"),
        );
        let client = reqwest::Client::builder()
            .default_headers(shared_headers)
            .build()
            .expect("No TLS backend found");
        Client {
            handle: Arc::new(Handle { web: client }),
        }
    }

    pub fn summoner(&self, region: SummonerRegion) -> summoner::SummonerClient {
        summoner::SummonerClient::new(self.handle.clone(), region)
    }
    pub fn matches(&self, region: MatchRegion) -> matches::MatchClient {
        matches::MatchClient::new(self.handle.clone(), region)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::env;
//     use std::{collections::HashMap, time::Instant};
//
//     use aws_config::profile::ProfileFileCredentialsProvider;
//     use aws_sdk_dynamodb::{
//         types::{AttributeValue, KeysAndAttributes, PutRequest, WriteRequest},
//         Client as DynamoClient,
//     };
//     use serde_dynamo::aws_sdk_dynamodb_0_25::{from_items, to_item};
//
//     use crate::commands::winrate::print_game_line;
//     use crate::db::GameItem;
//
//     use super::*;
//     #[test]
//     fn test_print_game_line() {
//         let first_line = print_game_line(false, "Vladimir", 12, 0, 5, true);
//         let second_line = print_game_line(true, "Nami", 1, 1, 28, true);
//
//         println!("{}", first_line);
//         println!("{}", second_line);
//     }
//     #[test]
//     fn test_queue_into() {
//         let queue: i64 = Queue::RankedSolo5x5.into();
//         assert_eq!(queue, 420);
//     }
//     #[test]
//     fn test_game_type_display() {
//         let game_type = GameType::Ranked;
//         assert_eq!(game_type.to_string(), "ranked");
//     }
//
//     #[tokio::test]
//     async fn test_get_summoner() {
//         let key = std::env::var("RIOT_API_KEY").expect("No API key found");
//         let client = Client::new(&key);
//
//         let summoner = client
//             .summoner(SummonerRegion::NA1)
//             .get_by_name("GhostJester")
//             .send()
//             .await
//             .unwrap();
//         assert_eq!(summoner.name, "GhostJester");
//     }
//
//     #[tokio::test]
//     async fn test_get_match_ids() {
//         let key = std::env::var("RIOT_API_KEY").expect("No API key found");
//         let client = Client::new(&key);
//
//         let matches = client
//             .matches(MatchRegion::AMERICAS)
//             .get_ids(
//                 "elIz6dJiPhipthToLKMHB8pdY_Z500BYDLpR_Yw3lWPWELbZm2lGDvSZvmenU6ZnuEQMQ-6HPLMJiA",
//             )
//             .count(30)
//             .send()
//             .await
//             .unwrap();
//         assert_eq!(matches.len(), 30);
//     }
//
//     #[tokio::test]
//     async fn test_get_match_details() {
//         let key = std::env::var("RIOT_API_KEY").expect("No API key found");
//         let client = Client::new(&key);
//
//         let details = client
//             .matches(MatchRegion::AMERICAS)
//             .get_details("NA1_4637629131")
//             .send()
//             .await
//             .unwrap();
//         assert!(details
//             .info
//             .participants
//             .iter()
//             .any(|p| p.summoner_name == "GhostJester"));
//     }
//
//     #[tokio::test]
//     async fn test_get_match_by_summoner_name() {
//         let now = Instant::now();
//         let key = std::env::var("RIOT_API_KEY").expect("No API key found");
//         let client = Client::new(&key);
//         let profile_name = "lpturmel";
//
//         let credentials_provider = ProfileFileCredentialsProvider::builder()
//             .profile_name(profile_name)
//             .build();
//
//         let config = aws_config::from_env()
//             .credentials_provider(credentials_provider)
//             .region("us-east-1")
//             .load()
//             .await;
//
//         let db_client = DynamoClient::new(&config);
//
//         let summoner = client
//             .summoner(SummonerRegion::NA1)
//             .get_by_name("rems")
//             .send()
//             .await
//             .unwrap();
//         let queue_type = Queue::RankedSolo5x5;
//
//         let game_ids = client
//             .matches(MatchRegion::AMERICAS)
//             .get_ids(&summoner.puuid)
//             .count(10)
//             .queue(queue_type)
//             .send()
//             .await
//             .unwrap();
//         let game_ids_count = game_ids.len();
//
//         let table_name = env::var("TABLE_NAME").expect("TABLE_NAME env var not set");
//         let keys = game_ids
//             .iter()
//             .map(|game_id| {
//                 let mut key = HashMap::new();
//                 key.insert("id".to_string(), AttributeValue::S(game_id.to_string()));
//                 key.insert("sk".to_string(), AttributeValue::S("#".to_string()));
//                 key
//             })
//             .collect::<Vec<_>>();
//
//         let items = KeysAndAttributes::builder().set_keys(Some(keys)).build();
//
//         let batch_get_res = db_client
//             .batch_get_item()
//             .request_items(&table_name, items)
//             .send()
//             .await
//             .unwrap();
//
//         let table_res = batch_get_res.responses.unwrap();
//         let items = table_res.get(&table_name).unwrap();
//
//         let items: Vec<GameItem> = from_items(items.clone()).unwrap();
//
//         let mut game_details: Vec<GameItem> = Vec::new();
//         let missing_game_ids = game_ids
//             .iter()
//             .filter(
//                 |game_id| match items.iter().find(|item| item.id == **game_id) {
//                     Some(i) => {
//                         game_details.push(i.clone());
//                         false
//                     }
//                     None => true,
//                 },
//             )
//             .collect::<Vec<_>>();
//
//         let game_details_fut = missing_game_ids
//             .iter()
//             .map(|game_id| {
//                 client
//                     .matches(MatchRegion::AMERICAS)
//                     .get_details(game_id)
//                     .send()
//             })
//             .collect::<Vec<_>>();
//
//         let game_details_res = futures::future::join_all(game_details_fut).await;
//         let game_details_res = game_details_res
//             .iter()
//             .filter_map(|res| res.as_ref().ok())
//             .collect::<Vec<_>>();
//
//         if !game_details_res.is_empty() {
//             let put_items = game_details_res
//                 .iter()
//                 .map(|game| {
//                     WriteRequest::builder()
//                         .put_request(
//                             PutRequest::builder()
//                                 .set_item(to_item::<GameItem>((*game).clone().into()).ok())
//                                 .build(),
//                         )
//                         .build()
//                 })
//                 .collect::<Vec<_>>();
//
//             let _write_fut = db_client
//                 .batch_write_item()
//                 .request_items(&table_name, put_items)
//                 .send()
//                 .await;
//         }
//
//         game_details.extend(
//             game_details_res
//                 .iter()
//                 .map(|game| (*game).clone().into())
//                 .collect::<Vec<GameItem>>(),
//         );
//         game_details.sort_by(|a, b| b.info.game_creation.cmp(&a.info.game_creation));
//
//         let game_count = game_details.len();
//
//         let user_games = game_details.iter().map(|game| {
//             game.info
//                 .participants
//                 .iter()
//                 .find(|p| {
//                     if game.info.game_id == 4653903057 {
//                         println!("p: {:?}", p);
//                     }
//                     p.summoner_id == summoner.id
//                 })
//                 .unwrap()
//         });
//         let user_games = user_games.collect::<Vec<_>>();
//         let user_games_no_remake = user_games
//             .iter()
//             .filter(|p| !p.game_ended_in_early_surrender);
//
//         let won_games = user_games_no_remake.filter(|p| p.win).count();
//
//         let game_lines = user_games
//             .iter()
//             .map(|p| match p.game_ended_in_early_surrender {
//                 true => print_game_line(true, &p.champion_name, 0, 0, 0, false),
//                 false => {
//                     print_game_line(false, &p.champion_name, p.kills, p.deaths, p.assists, p.win)
//                 }
//             })
//             .collect::<String>();
//
//         let winrate = won_games as f32 / game_count as f32 * 100.0;
//
//         println!("Winrate: {}", winrate);
//
//         println!("{game_lines}");
//
//         println!("Test took {}ms", now.elapsed().as_millis());
//
//         assert_eq!(game_ids_count, game_count);
//     }
// }
