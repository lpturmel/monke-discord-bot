use crate::{ident, GameType, Handle, ItemModel};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod user;
/// Module responsible for the tracking state of the players.
///

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackingItem {
    #[serde(flatten)]
    /// The item partition key
    pub item: ItemModel,
    pub puuid: String,
    pub account_id: String,
    pub summoner_name: String,
    #[serde(skip)]
    pub game_type: GameType,
}
impl TrackingItem {
    pub fn new(
        game_type: GameType,
        summoner_id: &str,
        puuid: &str,
        account_id: &str,
        summoner_name: &str,
    ) -> Self {
        let ident = ident(game_type);
        Self {
            item: ItemModel {
                id: "TRACKING".to_string(),
                sk: format!("SUMMONER#{}{}", ident, summoner_id),
            },
            puuid: puuid.to_string(),
            account_id: account_id.to_string(),
            summoner_name: summoner_name.to_string(),
            game_type,
        }
    }
    pub fn summoner_id(&self) -> String {
        let splits: Vec<&str> = self.item.sk.split('#').collect();
        splits.last().unwrap_or(&"").to_string()
    }
}
pub struct TrackingClient {
    handle: Arc<Handle>,
    game_type: GameType,
}

impl TrackingClient {
    pub fn new(handle: std::sync::Arc<Handle>, game_type: GameType) -> Self {
        Self { handle, game_type }
    }
    pub fn track_user(&self) -> user::TrackUserBuilder {
        user::TrackUserBuilder::new(self.handle.clone(), self.game_type)
    }
    pub fn list(&self) -> self::user::ListUserBuilder {
        self::user::ListUserBuilder::new(self.handle.clone(), self.game_type)
    }
    pub fn untrack_user(&self) -> user::UntrackUserBuilder {
        user::UntrackUserBuilder::new(self.handle.clone(), self.game_type)
    }
}
