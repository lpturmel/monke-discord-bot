use self::ids::IdsRequestBuilder;
use crate::matches::Region;
use crate::{Handle, ServiceUrl};

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
            "{}/tft/match/v1/matches/{}",
            self.region.base_url(),
            match_id
        );
        details::DetailsRequestBuilder::new(self.handle.clone(), url)
    }
    pub fn get_ids(&self, summoner_puuid: &str) -> IdsRequestBuilder {
        let url = format!(
            "{}/tft/match/v1/matches/by-puuid/{}/ids",
            self.region.base_url(),
            summoner_puuid
        );
        IdsRequestBuilder::new(self.handle.clone(), url)
    }
}
