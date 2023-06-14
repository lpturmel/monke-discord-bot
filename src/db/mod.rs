use crate::commands::winrate::WinRateError;
use crate::error::Result;
use crate::riot::matches::details::{Info, MatchDetails};
use crate::riot::matches::Region as MatchesRegion;
use crate::AppState;
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes, PutRequest, WriteRequest};
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_0_25::{from_items, to_item};
use std::collections::HashMap;
use std::env;

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

/// Get game details from DynamoDB and fetch the missing ones from source (Riot API)
///
/// When a game is not found in the cache, it is then added through a batch write to DynamoDB for
/// future requests.
///
/// Results are sorted by game creation time.
pub async fn get_game_details_from_cache(
    game_ids: &[String],
    state: &AppState,
) -> Result<Vec<GameItem>> {
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME env var not set");

    if game_ids.is_empty() {
        return Ok(Vec::new());
    }
    let keys = game_ids
        .iter()
        .map(|game_id| {
            let mut key = HashMap::new();
            key.insert("id".to_string(), AttributeValue::S(game_id.to_string()));
            key.insert("sk".to_string(), AttributeValue::S("#".to_string()));
            key
        })
        .collect::<Vec<_>>();

    let items = KeysAndAttributes::builder().set_keys(Some(keys)).build();

    let batch_get_res = state
        .db_client
        .batch_get_item()
        .request_items(&table_name, items)
        .send()
        .await?;

    let table_res = batch_get_res
        .responses
        .ok_or(WinRateError::GetItemNoTableResults)?;
    let items = table_res
        .get(&table_name)
        .ok_or(WinRateError::GetItemNoResults)?;

    let items: Vec<GameItem> = from_items(items.clone())?;

    let mut game_details: Vec<GameItem> = Vec::new();
    let missing_game_ids = game_ids
        .iter()
        .filter(
            |game_id| match items.iter().find(|item| item.id == **game_id) {
                Some(i) => {
                    game_details.push(i.clone());
                    false
                }
                None => true,
            },
        )
        .collect::<Vec<_>>();

    let game_details_fut = missing_game_ids
        .iter()
        .map(|game_id| {
            state
                .riot_client
                .matches(MatchesRegion::AMERICAS)
                .get_details(game_id)
                .send()
        })
        .collect::<Vec<_>>();

    println!("Fetching {} games from Riot API", game_details_fut.len());
    let game_details_res = futures::future::join_all(game_details_fut).await;
    let game_details_res = game_details_res
        .iter()
        .filter_map(|res| res.as_ref().ok())
        .collect::<Vec<_>>();

    if !game_details_res.is_empty() {
        let put_items = game_details_res
            .iter()
            .map(|game| {
                WriteRequest::builder()
                    .put_request(
                        PutRequest::builder()
                            .set_item(to_item::<GameItem>((*game).clone().into()).ok())
                            .build(),
                    )
                    .build()
            })
            .collect::<Vec<_>>();

        state
            .db_client
            .batch_write_item()
            .request_items(&table_name, put_items)
            .send()
            .await?;
    }

    game_details.extend(
        game_details_res
            .into_iter()
            .map(GameItem::from_match_details),
    );

    game_details.sort_by(|a, b| b.info.game_creation.cmp(&a.info.game_creation));

    Ok(game_details)
}
