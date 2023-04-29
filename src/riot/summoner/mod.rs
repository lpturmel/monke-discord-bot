use self::name::GetByNameRequestBuilder;
use super::league::Handle;
use std::fmt::Display;

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
            "{}/lol/summoner/v4/summoners/by-name/{}",
            self.region.base_url(),
            summoner_name
        );
        GetByNameRequestBuilder::new(self.handle.clone(), url)
    }
}
pub enum Region {
    BR1,
    EUN1,
    EUW1,
    JP1,
    KR,
    LA1,
    LA2,
    NA1,
    OC1,
    PH2,
    RU,
    SG2,
    TH2,
    TR1,
    TW2,
    VN2,
}

impl Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let region = match self {
            Region::BR1 => "br1",
            Region::EUN1 => "eun1",
            Region::EUW1 => "euw1",
            Region::JP1 => "jp1",
            Region::KR => "kr",
            Region::LA1 => "la1",
            Region::LA2 => "la2",
            Region::NA1 => "na1",
            Region::OC1 => "oc1",
            Region::PH2 => "ph2",
            Region::RU => "ru",
            Region::SG2 => "sg2",
            Region::TH2 => "th2",
            Region::TR1 => "tr1",
            Region::TW2 => "tw2",
            Region::VN2 => "vn2",
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
