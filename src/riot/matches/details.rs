use crate::error::Result;
use crate::riot::Error;
use crate::riot::Handle;
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
    pub async fn send(self) -> Result<MatchDetails> {
        let res = self.handle.web.execute(self.request).await?;
        match res.status() {
            reqwest::StatusCode::NOT_FOUND => return Err(Error::SummonerNotFound)?,
            reqwest::StatusCode::TOO_MANY_REQUESTS => return Err(Error::TooManyRequests)?,
            reqwest::StatusCode::UNAUTHORIZED => return Err(Error::Unauthorized)?,
            reqwest::StatusCode::FORBIDDEN => return Err(Error::Forbidden)?,
            _ => {}
        }
        let match_ids: MatchDetails = res.json().await?;
        Ok(match_ids)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDetails {
    pub metadata: Metadata,
    pub info: Info,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub data_version: String,
    pub match_id: String,
    pub participants: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub game_creation: i64,
    pub game_duration: i64,
    pub game_end_timestamp: i64,
    pub game_id: i64,
    pub game_mode: String,
    pub game_name: String,
    pub game_start_timestamp: i64,
    pub game_type: String,
    pub game_version: String,
    pub map_id: i64,
    pub participants: Vec<Participant>,
    pub platform_id: String,
    pub queue_id: i64,
    pub teams: Vec<Team>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Participant {
    pub assists: i64,
    pub champion_id: i64,
    pub champion_name: String,
    pub command_pings: i64,
    pub deaths: i64,
    pub double_kills: i64,
    pub summoner_name: String,
    pub summoner_id: String,
    pub win: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub bans: Vec<Ban>,
    pub objectives: Objectives,
    pub team_id: i64,
    pub win: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ban {
    pub champion_id: i64,
    pub pick_turn: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Objectives {
    pub baron: Baron,
    pub champion: Champion,
    pub dragon: Dragon,
    pub inhibitor: Inhibitor,
    pub rift_herald: RiftHerald,
    pub tower: Tower,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Baron {
    pub first: bool,
    pub kills: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Champion {
    pub first: bool,
    pub kills: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dragon {
    pub first: bool,
    pub kills: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inhibitor {
    pub first: bool,
    pub kills: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiftHerald {
    pub first: bool,
    pub kills: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tower {
    pub first: bool,
    pub kills: i64,
}
