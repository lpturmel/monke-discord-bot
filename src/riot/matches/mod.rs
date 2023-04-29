use self::ids::IdsRequestBuilder;
use crate::riot::Handle;
use std::fmt::Display;

pub mod details;
pub mod ids;

pub struct MatchClient {
    handle: std::sync::Arc<Handle>,
    region: Region,
}
impl MatchClient {
    pub fn new(handle: std::sync::Arc<Handle>, region: Region) -> Self {
        Self { handle, region }
    }
    pub fn get_details(&self, match_id: &str) -> details::DetailsRequestBuilder {
        let url = format!(
            "{}/lol/match/v5/matches/{}",
            self.region.base_url(),
            match_id
        );
        details::DetailsRequestBuilder::new(self.handle.clone(), url)
    }
    pub fn get_ids(&self, summoner_puuid: &str) -> IdsRequestBuilder {
        let url = format!(
            "{}/lol/match/v5/matches/by-puuid/{}/ids",
            self.region.base_url(),
            summoner_puuid
        );
        IdsRequestBuilder::new(self.handle.clone(), url)
    }
}

pub enum Region {
    AMERICAS,
    ASIA,
    EUROPE,
    SEA,
}

impl Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let region = match self {
            Region::AMERICAS => "americas",
            Region::ASIA => "asia",
            Region::EUROPE => "europe",
            Region::SEA => "sea",
        };
        write!(f, "{}", region)
    }
}
impl Region {
    /// Returns the base URL of the API for the region
    fn base_url(&self) -> String {
        format!("https://{}.api.riotgames.com", self)
    }
}
