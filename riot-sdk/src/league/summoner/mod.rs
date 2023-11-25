use self::league::GetLeagueDetailsRequestBuilder;
use self::name::GetByNameRequestBuilder;
use self::puuid::GetByPuuidRequestBuilder;
use crate::summoner::Region;
use crate::{Handle, ServiceUrl};

pub mod league;
pub mod name;
pub mod puuid;

pub struct SummonerClient {
    handle: std::sync::Arc<Handle>,
    region: Region,
}
impl SummonerClient {
    pub fn new(handle: std::sync::Arc<Handle>, region: Region) -> Self {
        Self { handle, region }
    }
    #[deprecated(since = "0.2.0", note = "Please use `get_by_puuid` instead")]
    pub fn get_by_name(&self, summoner_name: &str) -> GetByNameRequestBuilder {
        let url = format!(
            "{}/lol/summoner/v4/summoners/by-name/{}",
            self.region.base_url(),
            summoner_name
        );
        GetByNameRequestBuilder::new(self.handle.clone(), url)
    }

    pub fn get_by_puuid(&self, puuid: &str) -> GetByPuuidRequestBuilder {
        let url = format!(
            "{}/lol/summoner/v4/summoners/by-puuid/{}",
            self.region.base_url(),
            puuid
        );
        GetByPuuidRequestBuilder::new(self.handle.clone(), url)
    }

    /// Get league entries in all queues for a given summoner ID
    pub fn get_league_details(&self, summoner_id: &str) -> GetLeagueDetailsRequestBuilder {
        let url = format!(
            "{}/lol/league/v4/entries/by-summoner/{}",
            self.region.base_url(),
            summoner_id
        );
        GetLeagueDetailsRequestBuilder::new(self.handle.clone(), url)
    }
}
