use crate::error::Result;
use crate::riot::Queue;
use crate::riot::{Error, Handle};
use serde::{Deserialize, Serialize};

use std::str::FromStr;

pub struct GetLeagueDetailsRequestBuilder {
    request: reqwest::Request,
    handle: std::sync::Arc<Handle>,
}

impl GetLeagueDetailsRequestBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, url: String) -> Self {
        Self {
            handle,
            request: reqwest::Request::new(
                reqwest::Method::GET,
                reqwest::Url::from_str(&url).unwrap(),
            ),
        }
    }
    pub async fn send(self) -> Result<Vec<LeagueResponse>> {
        let res = self.handle.web.execute(self.request).await?;

        match res.status() {
            reqwest::StatusCode::OK => {}
            reqwest::StatusCode::NOT_FOUND => return Err(Error::SummonerNotFound)?,
            reqwest::StatusCode::TOO_MANY_REQUESTS => return Err(Error::TooManyRequests)?,
            reqwest::StatusCode::UNAUTHORIZED => return Err(Error::Unauthorized)?,
            reqwest::StatusCode::FORBIDDEN => return Err(Error::Forbidden)?,
            reqwest::StatusCode::BAD_REQUEST => return Err(Error::BadRequest)?,
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => return Err(Error::RiotError)?,
            _ => {}
        }
        let summoner: Vec<LeagueResponse> = res.json().await?;
        Ok(summoner)
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueResponse {
    pub league_id: String,
    pub queue_type: String,
    pub tier: String,
    pub rank: String,
    pub summoner_id: String,
    pub summoner_name: String,
    pub league_points: i64,
    pub wins: i64,
    pub losses: i64,
    pub hot_streak: bool,
}

/// Converts a queue type to a league specific api string representation
pub fn league_type_str(queue: &Queue) -> &str {
    match queue {
        Queue::RankedSolo5x5 => "RANKED_SOLO_5x5",
        Queue::RankedFlex5x5 => "RANKED_FLEX_SR",
    }
}
