use super::LpItem;
use crate::error::Result;
use crate::{GameType, Handle};
use serde_dynamo::aws_sdk_dynamodb_0_25::to_item;

pub struct AddBuilder {
    handle: std::sync::Arc<Handle>,
    /// The encrypted summoner id
    id: Option<String>,
    timestamp: Option<i64>,
    tier: Option<String>,
    rank: Option<String>,
    league_points: Option<i32>,
    wins: Option<i64>,
    losses: Option<i64>,
    game_type: GameType,
}

impl AddBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, game_type: GameType) -> Self {
        Self {
            handle,
            id: None,
            timestamp: None,
            tier: None,
            rank: None,
            league_points: None,
            wins: None,
            losses: None,
            game_type,
        }
    }
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
    pub fn tier(mut self, tier: &str) -> Self {
        self.tier = Some(tier.to_string());
        self
    }
    pub fn rank(mut self, rank: &str) -> Self {
        self.rank = Some(rank.to_string());
        self
    }
    pub fn league_points(mut self, league_points: i32) -> Self {
        self.league_points = Some(league_points);
        self
    }
    pub fn wins(mut self, wins: i64) -> Self {
        self.wins = Some(wins);
        self
    }
    pub fn losses(mut self, losses: i64) -> Self {
        self.losses = Some(losses);
        self
    }
    pub async fn send(self) -> Result<()> {
        let item = LpItem::new(
            self.game_type,
            &self.id.expect("id is required"),
            self.timestamp.expect("timestamp is required"),
            &self.tier.expect("tier is required"),
            &self.rank.expect("rank is required"),
            self.league_points.expect("league_points is required"),
            self.wins.expect("wins is required"),
            self.losses.expect("losses is required"),
        );
        self.handle
            .inner
            .put_item()
            .table_name(self.handle.table_name.as_str())
            .set_item(Some(to_item(&item).unwrap()))
            .send()
            .await?;
        Ok(())
    }
}
