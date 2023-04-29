use super::matches::{self, Region as MatchRegion};
use super::summoner::{self, Region as SummonerRegion};
use reqwest::header::HeaderMap;
use std::fmt::Display;
use std::sync::Arc;

#[derive(Debug)]
pub enum Error {
    HttpError(reqwest::Error),
    SummonerNotFound,
    TooManyRequests,
    Forbidden,
    Unauthorized,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::HttpError(e) => return e.fmt(f),
            Error::SummonerNotFound => "Summoner not found",
            Error::TooManyRequests => "Too many requests",
            Error::Forbidden => "API key is invalid",
            Error::Unauthorized => "Unauthorized",
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

impl Into<i64> for Queue {
    fn into(self) -> i64 {
        match self {
            Queue::RankedSolo5x5 => 420,
            Queue::RankedFlex5x5 => 440,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_queue_into() {
        let queue: i64 = Queue::RankedSolo5x5.into();
        assert_eq!(queue, 420);
    }
    #[test]
    fn test_game_type_display() {
        let game_type = GameType::Ranked;
        assert_eq!(game_type.to_string(), "ranked");
    }

    #[tokio::test]
    async fn test_get_summoner() {
        let key = std::env::var("RIOT_API_KEY").expect("No API key found");
        let client = Client::new(&key);

        let summoner = client
            .summoner(SummonerRegion::NA1)
            .get_by_name("GhostJester")
            .send()
            .await
            .unwrap();
        assert_eq!(summoner.name, "GhostJester");
    }

    #[tokio::test]
    async fn test_get_match_ids() {
        let key = std::env::var("RIOT_API_KEY").expect("No API key found");
        let client = Client::new(&key);

        let matches = client
            .matches(MatchRegion::AMERICAS)
            .get_ids(
                "elIz6dJiPhipthToLKMHB8pdY_Z500BYDLpR_Yw3lWPWELbZm2lGDvSZvmenU6ZnuEQMQ-6HPLMJiA",
            )
            .count(30)
            .send()
            .await
            .unwrap();
        assert_eq!(matches.len(), 30);
    }

    #[tokio::test]
    async fn test_get_match_details() {
        let key = std::env::var("RIOT_API_KEY").expect("No API key found");
        let client = Client::new(&key);

        let details = client
            .matches(MatchRegion::AMERICAS)
            .get_details("NA1_4637629131")
            .send()
            .await
            .unwrap();
        assert_eq!(
            details
                .info
                .participants
                .iter()
                .find(|p| p.summoner_name == "GhostJester")
                .is_some(),
            true
        );
    }

    #[tokio::test]
    async fn test_get_match_by_summoner_name() {
        let key = std::env::var("RIOT_API_KEY").expect("No API key found");
        let client = Client::new(&key);

        let summoner = client
            .summoner(SummonerRegion::NA1)
            .get_by_name("rems")
            .send()
            .await
            .unwrap();
        let queue_type = Queue::RankedSolo5x5;

        let game_ids = client
            .matches(MatchRegion::AMERICAS)
            .get_ids(&summoner.puuid)
            .count(10)
            .queue(queue_type)
            .send()
            .await
            .unwrap();

        let game_ids_count = game_ids.len();

        let game_details_fut = game_ids
            .iter()
            .map(|game_id| {
                client
                    .matches(MatchRegion::AMERICAS)
                    .get_details(game_id)
                    .send()
            })
            .collect::<Vec<_>>();

        let game_details = futures::future::join_all(game_details_fut).await;

        let game_details = game_details
            .into_iter()
            .filter_map(|game| match game {
                Ok(game) => Some(game),
                Err(e) => {
                    println!("Error: {}", e);
                    None
                }
            })
            .collect::<Vec<_>>();

        let game_details_count = game_details.len();

        let won_games = game_details
            .iter()
            .filter(|game| {
                game.info
                    .participants
                    .iter()
                    .find(|p| p.summoner_id == summoner.id)
                    .unwrap()
                    .win
            })
            .count();

        let winrate = won_games as f32 / game_details_count as f32 * 100.0;

        println!("Winrate: {}", winrate);

        assert_eq!(game_ids_count, game_details_count);
    }
}
