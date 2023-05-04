use serde::{Deserialize, Serialize};

use crate::riot::matches::details::{Info, MatchDetails};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameItem {
    /// This is the partition key
    pub id: String,
    /// This is the sort key
    pub sk: String,
    #[serde(flatten)]
    pub info: Info,
}

impl GameItem {
    pub fn from_match_details(details: &MatchDetails) -> Self {
        let sk = "#".to_string();

        let info = details.info.clone();

        Self {
            id: details.metadata.match_id.clone(),
            sk,
            info,
        }
    }
}
impl From<MatchDetails> for GameItem {
    fn from(details: MatchDetails) -> Self {
        let id = details.metadata.match_id;
        let sk = "#".to_string();

        Self {
            id,
            sk,
            info: details.info,
        }
    }
}
