use super::LpItem;
use crate::error::Result;
use crate::{ident, GameType, Handle};
use aws_sdk_dynamodb::types::AttributeValue;
use serde_dynamo::aws_sdk_dynamodb_0_25::from_item;

pub struct GetBuilder {
    handle: std::sync::Arc<Handle>,
    /// The encrypted summoner id
    id: Option<String>,
    timestamp: Option<i64>,
    game_type: GameType,
}

impl GetBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, game_type: GameType) -> Self {
        Self {
            handle,
            id: None,
            timestamp: None,
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
    pub async fn send(self) -> Result<Option<LpItem>> {
        let id = self.id.expect("id is required");
        let timestamp = self.timestamp.expect("timestamp is required");
        let ident = ident(self.game_type);
        let sk = format!("#{}{}", ident, timestamp);

        let res = self
            .handle
            .inner
            .get_item()
            .table_name(self.handle.table_name.as_str())
            .key("id", AttributeValue::S(id))
            .key("sk", AttributeValue::S(sk))
            .send()
            .await?;

        match res.item() {
            Some(item) => Ok(Some(from_item(item.clone()).unwrap())),
            None => Ok(None),
        }
    }
}
