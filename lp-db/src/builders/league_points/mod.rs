use crate::{ident, GameType, Handle, ItemModel};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod add;
pub mod between;
pub mod get;
/// Module responsible for handling the league points entries per summoner
///

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LpItem {
    #[serde(flatten)]
    /// The item partition key
    pub item: ItemModel,
    pub tier: String,
    pub rank: String,
    pub league_points: i32,
    pub wins: i64,
    pub losses: i64,
}
impl LpItem {
    pub fn new(
        game_type: GameType,
        summoner_id: &str,
        timestamp: i64,
        tier: &str,
        rank: &str,
        league_points: i32,
        wins: i64,
        losses: i64,
    ) -> Self {
        let ident = ident(game_type);
        Self {
            item: ItemModel {
                id: summoner_id.to_string(),
                sk: format!("#{}{}", ident, timestamp),
            },
            tier: tier.to_string(),
            rank: rank.to_string(),
            league_points,
            wins,
            losses,
        }
    }
}
pub struct LeaguePointClient {
    handle: Arc<Handle>,
    game_type: GameType,
}

impl LeaguePointClient {
    pub fn new(handle: std::sync::Arc<Handle>, game_type: GameType) -> Self {
        Self { handle, game_type }
    }
    pub fn add(&self) -> add::AddBuilder {
        add::AddBuilder::new(self.handle.clone(), self.game_type)
    }
    pub fn get(&self) -> self::get::GetBuilder {
        self::get::GetBuilder::new(self.handle.clone(), self.game_type)
    }
    pub fn get_between(&self) -> self::between::GetBetweenBuilder {
        self::between::GetBetweenBuilder::new(self.handle.clone(), self.game_type)
    }
}
