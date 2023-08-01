use self::league::GetLeagueDetailsRequestBuilder;
use self::name::GetByNameRequestBuilder;
use crate::summoner::Region;
use crate::{Handle, ServiceUrl};

pub mod league;
pub mod name;

pub struct SummonerClient {
    handle: std::sync::Arc<Handle>,
    region: Region,
}
impl SummonerClient {
    pub fn new(handle: std::sync::Arc<Handle>, region: Region) -> Self {
        Self { handle, region }
    }
    pub fn get_by_name(&self, summoner_name: &str) -> GetByNameRequestBuilder {
        let url = format!(
            "{}/tft/summoner/v1/summoners/by-name/{}",
            self.region.base_url(),
            summoner_name
        );
        GetByNameRequestBuilder::new(self.handle.clone(), url)
    }

    /// Get league entries in all queues for a given summoner ID
    pub fn get_league_details(&self, summoner_id: &str) -> GetLeagueDetailsRequestBuilder {
        let url = format!(
            "{}/tft/league/v1/entries/by-summoner/{}",
            self.region.base_url(),
            summoner_id
        );
        GetLeagueDetailsRequestBuilder::new(self.handle.clone(), url)
    }
}
