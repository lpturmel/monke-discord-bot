use super::LpItem;
use crate::error::Result;
use crate::{ident, GameType, Handle};
use aws_sdk_dynamodb::types::AttributeValue;
use serde_dynamo::aws_sdk_dynamodb_0_25::from_items;

pub struct GetBetweenBuilder {
    handle: std::sync::Arc<Handle>,
    /// The encrypted summoner id
    id: Option<String>,
    /// The starting timestamp of the query
    start_time: Option<i64>,
    /// The ending timestamp of the query
    end_time: Option<i64>,
    game_type: GameType,
}

impl GetBetweenBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, game_type: GameType) -> Self {
        Self {
            handle,
            id: None,
            start_time: None,
            end_time: None,
            game_type,
        }
    }
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    pub fn start_time(mut self, timestamp: i64) -> Self {
        self.start_time = Some(timestamp);
        self
    }
    pub fn end_time(mut self, timestamp: i64) -> Self {
        self.end_time = Some(timestamp);
        self
    }
    pub async fn send(self) -> Result<Option<Vec<LpItem>>> {
        let id = self.id.expect("id is required");
        let start_time = self.start_time.expect("start_time is required");
        let end_time = self.end_time.expect("end_time is required");

        let ident = ident(self.game_type);
        let res = self
            .handle
            .inner
            .query()
            .table_name(self.handle.table_name.as_str())
            .key_condition_expression("id = :id AND sk BETWEEN :start_time AND :end_time")
            .expression_attribute_values(":id", AttributeValue::S(id))
            .expression_attribute_values(
                ":start_time",
                AttributeValue::S(format!("#{}{}", ident, start_time)),
            )
            .expression_attribute_values(
                ":end_time",
                AttributeValue::S(format!("#{}{}", ident, end_time)),
            )
            .send()
            .await?;

        match res.items() {
            Some(items) => Ok(Some(from_items(items.to_vec()).unwrap())),
            None => Ok(None),
        }
    }
}
