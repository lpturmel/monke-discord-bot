pub mod error;

use aws_config::SdkConfig;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod builders;

#[derive(Debug)]
pub struct Handle {
    pub inner: DynamoDbClient,
    pub table_name: String,
}
/// Database client for the LP service
///
/// LP: League Points
pub struct Client {
    handle: Arc<Handle>,
}

impl Client {
    pub fn new(table_name: &str, sdk_config: &SdkConfig) -> Self {
        let handle = Arc::new(Handle {
            inner: DynamoDbClient::new(sdk_config),
            table_name: table_name.to_string(),
        });
        Self { handle }
    }

    pub fn tracking(&self, game_type: GameType) -> builders::tracking::TrackingClient {
        builders::tracking::TrackingClient::new(self.handle.clone(), game_type)
    }
    pub fn league_points(&self, game_type: GameType) -> builders::league_points::LeaguePointClient {
        builders::league_points::LeaguePointClient::new(self.handle.clone(), game_type)
    }
}

fn ident(game_type: GameType) -> &'static str {
    match game_type {
        GameType::League => "LEAGUE#",
        GameType::Tft => "TFT#",
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub enum GameType {
    #[default]
    League,
    Tft,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemModel {
    /// PK
    pub id: String,
    /// SK
    pub sk: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackingModel {
    #[serde(flatten)]
    pub item: ItemModel,

    pub ids: Vec<RiotAccountDetails>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotAccountDetails {
    id: String,
    account_id: String,
    puuid: String,
    summoner_name: String,
}
