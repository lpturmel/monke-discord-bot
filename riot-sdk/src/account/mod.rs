use crate::{Error, Handle, Result, ServiceUrl};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

pub enum AccountRegion {
    AMERICAS,
    ASIA,
    EUROPE,
    ESPORTS,
}

impl Display for AccountRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let region = match self {
            AccountRegion::AMERICAS => "americas",
            AccountRegion::ASIA => "asia",
            AccountRegion::EUROPE => "europe",
            AccountRegion::ESPORTS => "sea",
        };
        write!(f, "{}", region)
    }
}
impl ServiceUrl for AccountRegion {}

pub struct AccountClient {
    handle: std::sync::Arc<Handle>,
    region: AccountRegion,
}
impl AccountClient {
    pub fn new(handle: std::sync::Arc<Handle>, region: AccountRegion) -> Self {
        Self { handle, region }
    }
    /// Get summoner PUUID by Riot ID (gameName + tagLine)
    pub fn get_by_riot_id(&self, game_name: &str, tag_line: &str) -> GetByRiotIdRequestBuilder {
        let url = format!(
            "{}/riot/account/v1/accounts/by-riot-id/{}/{}",
            self.region.base_url(),
            game_name,
            tag_line
        );
        GetByRiotIdRequestBuilder::new(self.handle.clone(), url)
    }
}

pub struct GetByRiotIdRequestBuilder {
    request: reqwest::Request,
    handle: std::sync::Arc<Handle>,
}

impl GetByRiotIdRequestBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, url: String) -> Self {
        Self {
            handle,
            request: reqwest::Request::new(
                reqwest::Method::GET,
                reqwest::Url::from_str(&url).unwrap(),
            ),
        }
    }
    pub async fn send(self) -> Result<AccountResponse> {
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
        let summoner: AccountResponse = res.json().await?;
        Ok(summoner)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}
