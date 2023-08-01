use crate::{Error, Handle, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub struct DetailsRequestBuilder {
    request: reqwest::Request,
    handle: std::sync::Arc<Handle>,
}

impl DetailsRequestBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, url: String) -> Self {
        Self {
            handle,
            request: reqwest::Request::new(
                reqwest::Method::GET,
                reqwest::Url::from_str(&url).unwrap(),
            ),
        }
    }
    pub async fn send(self) -> Result<TftMatchDetails> {
        let res = self.handle.web.execute(self.request).await?;
        match res.status() {
            reqwest::StatusCode::NOT_FOUND => return Err(Error::SummonerNotFound)?,
            reqwest::StatusCode::TOO_MANY_REQUESTS => return Err(Error::TooManyRequests)?,
            reqwest::StatusCode::UNAUTHORIZED => return Err(Error::Unauthorized)?,
            reqwest::StatusCode::FORBIDDEN => return Err(Error::Forbidden)?,
            reqwest::StatusCode::BAD_REQUEST => return Err(Error::BadRequest)?,
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => return Err(Error::RiotError)?,
            _ => {}
        }
        let match_ids: TftMatchDetails = res.json().await?;
        Ok(match_ids)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TftMatchDetails {
    pub metadata: Metadata,
    pub info: Info,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(rename = "data_version")]
    pub data_version: String,
    #[serde(rename = "match_id")]
    pub match_id: String,
    pub participants: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    #[serde(rename = "game_datetime")]
    pub game_datetime: i64,
    #[serde(rename = "game_length")]
    pub game_length: f64,
    #[serde(rename = "game_version")]
    pub game_version: String,
    pub participants: Vec<Participant>,
    #[serde(rename = "queue_id")]
    pub queue_id: i64,
    #[serde(rename = "tft_game_type")]
    pub tft_game_type: String,
    #[serde(rename = "tft_set_core_name")]
    pub tft_set_core_name: String,
    #[serde(rename = "tft_set_number")]
    pub tft_set_number: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Participant {
    pub augments: Vec<String>,
    pub companion: Companion,
    #[serde(rename = "gold_left")]
    pub gold_left: i64,
    #[serde(rename = "last_round")]
    pub last_round: i64,
    pub level: i64,
    pub placement: i64,
    #[serde(rename = "players_eliminated")]
    pub players_eliminated: i64,
    pub puuid: String,
    #[serde(rename = "time_eliminated")]
    pub time_eliminated: f64,
    #[serde(rename = "total_damage_to_players")]
    pub total_damage_to_players: i64,
    pub traits: Vec<Trait>,
    pub units: Vec<Unit>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Companion {
    #[serde(rename = "content_ID")]
    pub content_id: String,
    #[serde(rename = "item_ID")]
    pub item_id: i64,
    #[serde(rename = "skin_ID")]
    pub skin_id: i64,
    pub species: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trait {
    pub name: String,
    #[serde(rename = "num_units")]
    pub num_units: i64,
    pub style: i64,
    #[serde(rename = "tier_current")]
    pub tier_current: i64,
    #[serde(rename = "tier_total")]
    pub tier_total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    #[serde(rename = "character_id")]
    pub character_id: String,
    pub item_names: Vec<String>,
    pub name: String,
    pub rarity: i64,
    pub tier: i64,
}
