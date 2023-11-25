pub mod account;
pub mod league;
pub mod matches;
pub mod summoner;
pub mod tft;

use crate::matches::Region as MatchRegion;
use crate::summoner::Region as SummonerRegion;
use reqwest::header::HeaderMap;
use std::fmt::Display;
use std::sync::Arc;

use self::account::AccountRegion;

pub type Result<T> = core::result::Result<T, Error>;

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
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::HttpError(e)
    }
}
#[derive(Debug, Clone)]
pub enum Queue {
    RankedSolo5x5,
    RankedFlex5x5,
    TFTRanked,
    TFTHyperRoll,
    TFTNormal,
}

impl Queue {
    pub fn friendly_name(&self) -> &'static str {
        match self {
            Queue::RankedSolo5x5 => "Ranked Solo/Duo",
            Queue::RankedFlex5x5 => "Ranked Flex",
            Queue::TFTRanked => "Ranked",
            Queue::TFTHyperRoll => "Hyper Roll",
            Queue::TFTNormal => "Normal",
        }
    }
}
impl Display for Queue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let queue = match self {
            Queue::RankedSolo5x5 => "RANKED_SOLO",
            Queue::RankedFlex5x5 => "RANKED_FLEX",
            Queue::TFTRanked => "RANKED_TFT",
            Queue::TFTHyperRoll => "RANKED_TFT_HYPERROLL",
            Queue::TFTNormal => "NORMAL_TFT",
        };
        write!(f, "{}", queue)
    }
}
impl From<i64> for Queue {
    fn from(queue: i64) -> Self {
        match queue {
            420 => Queue::RankedSolo5x5,
            440 => Queue::RankedFlex5x5,
            1100 => Queue::TFTRanked,
            1130 => Queue::TFTHyperRoll,
            1090 => Queue::TFTNormal,
            _ => panic!("Unknown queue type {}", queue),
        }
    }
}
impl From<Queue> for i64 {
    fn from(queue: Queue) -> Self {
        match queue {
            Queue::RankedSolo5x5 => 420,
            Queue::RankedFlex5x5 => 440,
            Queue::TFTRanked => 1100,
            Queue::TFTHyperRoll => 1130,
            Queue::TFTNormal => 1090,
        }
    }
}

impl From<&Queue> for GameType {
    fn from(queue: &Queue) -> Self {
        match queue {
            Queue::RankedSolo5x5 => GameType::Ranked,
            Queue::RankedFlex5x5 => GameType::Ranked,
            Queue::TFTRanked => GameType::Ranked,
            Queue::TFTHyperRoll => GameType::Ranked,
            Queue::TFTNormal => GameType::Normal,
        }
    }
}

impl From<Queue> for GameType {
    fn from(queue: Queue) -> Self {
        match queue {
            Queue::RankedSolo5x5 => GameType::Ranked,
            Queue::RankedFlex5x5 => GameType::Ranked,
            Queue::TFTRanked => GameType::Ranked,
            Queue::TFTHyperRoll => GameType::Ranked,
            Queue::TFTNormal => GameType::Normal,
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
/// Client for interacting with the Riot Account APIs
pub struct AccountClient {
    handle: Arc<Handle>,
}

impl AccountClient {
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
        Self {
            handle: Arc::new(Handle { web: client }),
        }
    }
    pub fn account(&self, region: AccountRegion) -> account::AccountClient {
        account::AccountClient::new(self.handle.clone(), region)
    }
}

impl Clone for AccountClient {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}
/// Client for interacting with the League of Legends specific Riot APIs
pub struct LeagueClient {
    handle: Arc<Handle>,
}

impl LeagueClient {
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
        Self {
            handle: Arc::new(Handle { web: client }),
        }
    }

    pub fn summoner(&self, region: SummonerRegion) -> league::summoner::SummonerClient {
        league::summoner::SummonerClient::new(self.handle.clone(), region)
    }
    pub fn matches(&self, region: MatchRegion) -> league::matches::MatchClient {
        league::matches::MatchClient::new(self.handle.clone(), region)
    }
}

impl Clone for LeagueClient {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

/// Client for interacting with the Team Fight Tactics specific Riot APIs
pub struct TftClient {
    handle: Arc<Handle>,
}

impl TftClient {
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
        Self {
            handle: Arc::new(Handle { web: client }),
        }
    }

    pub fn summoner(&self, region: SummonerRegion) -> tft::summoner::SummonerClient {
        tft::summoner::SummonerClient::new(self.handle.clone(), region)
    }
    pub fn matches(&self, region: MatchRegion) -> tft::matches::MatchClient {
        tft::matches::MatchClient::new(self.handle.clone(), region)
    }
}

impl Clone for TftClient {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}
#[derive(Debug)]
pub enum Division {
    Iron,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Emerald,
    Diamond,
    Master,
    Grandmaster,
    Challenger,
}

#[derive(Debug)]
pub enum Rank {
    I,
    II,
    III,
    IV,
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank = match self {
            Rank::I => "I",
            Rank::II => "II",
            Rank::III => "III",
            Rank::IV => "IV",
        };
        write!(f, "{}", rank)
    }
}

#[derive(Debug)]
pub struct PlayerRank {
    division: Division,
    rank: Rank,
    league_points: i32,
}

impl PlayerRank {
    pub fn formatted_rank(&self) -> String {
        let division_str = match self.division {
            Division::Iron => format!("Iron {}", self.rank),
            Division::Bronze => format!("Bronze {}", self.rank),
            Division::Silver => format!("Silver {}", self.rank),
            Division::Gold => format!("Gold {}", self.rank),
            Division::Platinum => format!("Platinum {}", self.rank),
            Division::Emerald => format!("Emerald {}", self.rank),
            Division::Diamond => format!("Diamond {}", self.rank),
            Division::Master => "Master".to_string(),
            Division::Grandmaster => "Grandmaster".to_string(),
            Division::Challenger => "Challenger".to_string(),
        };
        format!("**{}** {} LP", division_str, self.league_points)
    }
    pub fn parse_str(division: &str, rank: &str, league_points: i32) -> Self {
        let division = match division {
            "IRON" => Division::Iron,
            "BRONZE" => Division::Bronze,
            "SILVER" => Division::Silver,
            "GOLD" => Division::Gold,
            "PLATINUM" => Division::Platinum,
            "EMERALD" => Division::Emerald,
            "DIAMOND" => Division::Diamond,
            "MASTER" => Division::Master,
            "GRANDMASTER" => Division::Grandmaster,
            "CHALLENGER" => Division::Challenger,
            _ => panic!("Invalid division"),
        };

        let rank = match rank {
            "I" => Rank::I,
            "II" => Rank::II,
            "III" => Rank::III,
            "IV" => Rank::IV,
            _ => panic!("Invalid rank"),
        };

        PlayerRank {
            division,
            rank,
            league_points,
        }
    }
    pub fn new(division: Division, rank: Rank, league_points: i32) -> Self {
        PlayerRank {
            division,
            rank,
            league_points,
        }
    }

    // This function is used to convert Division and Rank into a numerical equivalent.
    // It's assumed each rank within a division has a 100 points difference,
    // and each division has a 400 points difference.
    // This conversion might vary depending on the actual game design.
    pub fn to_points(&self) -> i32 {
        let division_points = match &self.division {
            Division::Iron => 0,
            Division::Bronze => 400,
            Division::Silver => 800,
            Division::Gold => 1200,
            Division::Platinum => 1600,
            Division::Emerald => 2000,
            Division::Diamond => 2400,
            Division::Master => 2800,
            Division::Grandmaster => 2800,
            Division::Challenger => 2800,
        };

        let rank_points = match &self.rank {
            Rank::I => 300,
            Rank::II => 200,
            Rank::III => 100,
            Rank::IV => 0,
        };

        match &self.division {
            Division::Master => division_points + self.league_points,
            Division::Grandmaster => division_points + self.league_points,
            Division::Challenger => division_points + self.league_points,
            _ => division_points + rank_points + self.league_points,
        }
    }

    pub fn points_difference(&self, other: &PlayerRank) -> i32 {
        other.to_points() - self.to_points()
    }
}

trait ServiceUrl
where
    Self: Display,
{
    fn base_url(&self) -> String {
        format!("https://{}.api.riotgames.com", self)
    }
}

#[cfg(test)]
mod tests {
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
    use super::*;
    #[test]
    fn easy_rank_compare() {
        let rank_one = PlayerRank::parse_str("GOLD", "I", 64);
        let rank_two = PlayerRank::parse_str("GOLD", "I", 60);

        let diff = rank_one.points_difference(&rank_two);
        assert_eq!(diff, -4);
    }
    #[test]
    fn high_div_rank_compare() {
        let rank_one = PlayerRank::parse_str("GRANDMASTER", "I", 764);
        let rank_two = PlayerRank::parse_str("MASTER", "I", 77);

        let diff = rank_one.points_difference(&rank_two);
        assert_eq!(diff, -687);
    }
    #[test]
    fn cross_div_rank_compare() {
        let rank_one = PlayerRank::parse_str("GOLD", "I", 77);
        let rank_two = PlayerRank::parse_str("PLATINUM", "IV", 4);

        let diff = rank_one.points_difference(&rank_two);
        assert_eq!(diff, 27);
    }
}
