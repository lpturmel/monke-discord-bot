use std::str::FromStr;

use crate::error::{Error, Result};
use crate::ResponseFuture;
use ed25519_dalek::{Signature, Verifier};
use lambda_http::{http::header::CONTENT_TYPE, IntoResponse, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Command {
    Winrate,
    Recap,
    Track,
    Untrack,
    List,
}

impl Command {
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s {
            "1101728526060765215" => Some(Command::Winrate),
            "1117931115672502322" => Some(Command::Recap),
            "1120493627685208104" => Some(Command::Track),
            "1120509247931818045" => Some(Command::List),
            "1120510262055796788" => Some(Command::Untrack),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum GameType {
    League,
    Tft,
}

impl FromStr for GameType {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "league" => Ok(GameType::League),
            "tft" => Ok(GameType::Tft),
            _ => Err(Error::BadOption),
        }
    }
}

#[derive(Debug)]
pub struct InteractionResponse;

impl InteractionResponse {
    pub fn new<S: Into<String>>(r#type: ResponseType, content: S) -> DiscordResponse {
        DiscordResponse {
            r#type: r#type.to_int(),
            data: DiscordResponseData {
                content: content.into(),
                flags: 0,
                tts: false,
                embeds: None,
            },
        }
    }
}
pub enum ResponseType {
    Pong,
    ChannelMessageWithSource,
    DeferredChannelMessageWithSource,
    DeferredUpdateMessage,
    UpdateMessage,
    ApplicationCommandAutocompleteResult,
    Modal,
}
impl ResponseType {
    fn to_int(&self) -> u64 {
        match self {
            ResponseType::Pong => 1,
            ResponseType::ChannelMessageWithSource => 4,
            ResponseType::DeferredChannelMessageWithSource => 5,
            ResponseType::DeferredUpdateMessage => 6,
            ResponseType::UpdateMessage => 7,
            ResponseType::ApplicationCommandAutocompleteResult => 8,
            ResponseType::Modal => 9,
        }
    }
}
/// Verify the signature of a discord interaction
///
/// The signature is in ed25519 format
pub fn verify_sig(body: &str, signature: &str, timestamp: &str, public_key: &str) -> Result<bool> {
    let sig_data = hex::decode(signature)?;
    let public_key_data = hex::decode(public_key)?;
    let signature = Signature::from_bytes(&sig_data)?;
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_data)?;
    let timestamp_data = timestamp.as_bytes();
    let body_data = body.as_bytes();
    let message = [timestamp_data, body_data].concat();
    Ok(public_key.verify(&message, &signature).is_ok())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordOption {
    pub name: String,
    pub r#type: u8,
    pub value: Option<serde_json::Value>,
    pub options: Option<Vec<DiscordOption>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordData {
    pub id: String,
    pub name: String,
    pub r#type: u64,
    pub options: Option<Vec<DiscordOption>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUser {
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
    pub discriminator: String,
    pub id: String,
    pub public_flags: u64,
    pub username: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordMember {
    pub roles: Vec<String>,
    pub user: DiscordUser,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordPayload {
    pub application_id: String,
    pub channel_id: Option<String>,
    /// The data of the incoming integration command
    pub data: Option<DiscordData>,
    pub guild_id: Option<String>,
    pub member: Option<DiscordMember>,
    pub r#type: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordResponseData {
    pub content: String,
    pub flags: u64,
    pub tts: bool,
    pub embeds: Option<Vec<serde_json::Value>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordResponse {
    pub r#type: u64,
    pub data: DiscordResponseData,
}

impl IntoResponse for DiscordResponse {
    fn into_response(self) -> ResponseFuture {
        Box::pin(async move {
            Response::builder()
                .header(CONTENT_TYPE, "application/json")
                .body(
                    serde_json::to_string(&self)
                        .expect("unable to serialize serde_json::Value")
                        .into(),
                )
                .expect("unable to build http::Response")
        })
    }
}
