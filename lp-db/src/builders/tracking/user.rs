use super::{ident, TrackingItem};
use crate::error::Result;
use crate::{GameType, Handle};
use aws_sdk_dynamodb::types::AttributeValue;
use serde_dynamo::aws_sdk_dynamodb_0_25::{from_items, to_item};

pub struct TrackUserBuilder {
    handle: std::sync::Arc<Handle>,
    /// The encrypted summoner id
    id: Option<String>,
    /// The PUUID of the summoner
    puuid: Option<String>,
    /// The account id of the summoner
    account_id: Option<String>,
    /// The summoner name
    summoner_name: Option<String>,
    game_type: GameType,
}

impl TrackUserBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, game_type: GameType) -> Self {
        Self {
            handle,
            id: None,
            puuid: None,
            account_id: None,
            summoner_name: None,
            game_type,
        }
    }
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    pub fn puuid(mut self, puuid: &str) -> Self {
        self.puuid = Some(puuid.to_string());
        self
    }
    pub fn account_id(mut self, account_id: &str) -> Self {
        self.account_id = Some(account_id.to_string());
        self
    }
    /// This field might get outdated as users can change their summoner name
    pub fn summoner_name(mut self, summoner_name: &str) -> Self {
        self.summoner_name = Some(summoner_name.to_string());
        self
    }
    pub async fn send(self) -> Result<()> {
        let item = TrackingItem::new(
            self.game_type,
            &self.id.expect("id is required"),
            &self.puuid.expect("puuid is required"),
            &self.account_id.expect("account_id is required"),
            &self.summoner_name.expect("summoner_name is required"),
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

pub struct ListUserBuilder {
    handle: std::sync::Arc<Handle>,
    game_type: GameType,
}

impl ListUserBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, game_type: GameType) -> Self {
        Self { handle, game_type }
    }
    pub async fn send(self) -> Result<Vec<TrackingItem>> {
        let ident = ident(self.game_type);
        let res = self
            .handle
            .inner
            .query()
            .table_name(self.handle.table_name.as_str())
            .key_condition_expression("id = :id AND begins_with(sk, :sk)")
            .expression_attribute_values(":id", AttributeValue::S("TRACKING".to_string()))
            .expression_attribute_values(":sk", AttributeValue::S(format!("SUMMONER#{}", ident)))
            .send()
            .await?;

        match res.items {
            Some(items) => Ok(from_items(items).unwrap()),
            None => Ok(vec![]),
        }
    }
}

pub struct UntrackUserBuilder {
    handle: std::sync::Arc<Handle>,
    /// The encrypted summoner id
    id: Option<String>,
    game_type: GameType,
}

impl UntrackUserBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, game_type: GameType) -> Self {
        Self {
            handle,
            id: None,
            game_type,
        }
    }
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    pub async fn send(self) -> Result<()> {
        let id = &self.id.expect("id is required");
        let ident = ident(self.game_type);
        self.handle
            .inner
            .delete_item()
            .table_name(self.handle.table_name.as_str())
            .key("id", AttributeValue::S("TRACKING".to_string()))
            .key("sk", AttributeValue::S(format!("SUMMONER#{}{}", ident, id)))
            .send()
            .await?;
        Ok(())
    }
}
