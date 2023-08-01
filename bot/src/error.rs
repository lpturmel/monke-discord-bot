use crate::{
    commands::winrate::WinRateError,
    discord::{InteractionResponse, ResponseType},
    ResponseFuture,
};
use aws_smithy_http::result::CreateUnhandledError;
use ed25519_dalek::SignatureError;
use hex::FromHexError;
use lambda_http::{
    http::{header::CONTENT_TYPE, StatusCode},
    Body, IntoResponse, Response,
};
use std::fmt::Display;
use std::string::FromUtf8Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingSignature,
    MissingTimestamp,
    BadSignature,
    BadBody,
    SerializeError(serde_json::Error),
    BadCommand,
    BadOption,
    HttpError(reqwest::Error),
    WinrateCommandError(WinRateError),
    RiotApiError(riot_sdk::Error),
    AwsSdk(String),
    Validation(String),
    LeaguePointsServiceError(lp_db::error::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::SerializeError(e) => Some(e),
            _ => None,
        }
    }
}
impl<T: std::error::Error + 'static + Send + Sync + CreateUnhandledError>
    From<aws_sdk_dynamodb::error::SdkError<T>> for Error
{
    fn from(err: aws_sdk_dynamodb::error::SdkError<T>) -> Self {
        let msg = format!("{:?}", err);
        Self::AwsSdk(msg)
    }
}
impl From<lp_db::error::Error> for Error {
    fn from(e: lp_db::error::Error) -> Self {
        Error::LeaguePointsServiceError(e)
    }
}
impl From<serde_dynamo::Error> for Error {
    fn from(err: serde_dynamo::Error) -> Self {
        Self::Validation(err.to_string())
    }
}
impl From<riot_sdk::Error> for Error {
    fn from(e: riot_sdk::Error) -> Self {
        Error::RiotApiError(e)
    }
}
impl From<WinRateError> for Error {
    fn from(e: WinRateError) -> Self {
        Error::WinrateCommandError(e)
    }
}
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::HttpError(e)
    }
}

impl From<Error> for Body {
    fn from(e: Error) -> Self {
        Body::from(format!("{}", e))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::MissingSignature => "Missing signature",
            Error::MissingTimestamp => "Missing timestamp",
            Error::BadSignature => "Bad signature",
            Error::BadBody => "Bad body",
            Error::BadOption => "Bad option",
            Error::LeaguePointsServiceError(e) => return e.fmt(f),
            Error::SerializeError(e) => return e.fmt(f),
            Error::BadCommand => "Bad command",
            Error::Validation(e) => return e.fmt(f),
            Error::HttpError(e) => return e.fmt(f),
            Error::RiotApiError(e) => return e.fmt(f),
            Error::WinrateCommandError(e) => return e.fmt(f),
            Error::AwsSdk(e) => {
                println!("AwsSdk error: {}", e);
                "Aws sdk error"
            }
        };
        write!(f, "{}", msg)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> ResponseFuture {
        let (status, body) = match self {
            Error::MissingSignature => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::MissingTimestamp => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::BadSignature => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::BadBody => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::SerializeError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Error::HttpError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Error::BadCommand => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::AwsSdk(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Error::Validation(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Error::LeaguePointsServiceError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&InteractionResponse::new(
                    ResponseType::ChannelMessageWithSource,
                    e.to_string(),
                ))
                .unwrap(),
            ),
            Error::BadOption => (
                StatusCode::OK,
                serde_json::to_string(&InteractionResponse::new(
                    ResponseType::ChannelMessageWithSource,
                    self.to_string(),
                ))
                .unwrap(),
            ),
            Error::RiotApiError(e) => (
                StatusCode::OK,
                serde_json::to_string(&InteractionResponse::new(
                    ResponseType::ChannelMessageWithSource,
                    e.to_string(),
                ))
                .unwrap(),
            ),
            // Send an interaction response (this is an application level error but not a http error)
            Error::WinrateCommandError(e) => (
                StatusCode::OK,
                serde_json::to_string(&InteractionResponse::new(
                    ResponseType::ChannelMessageWithSource,
                    e.to_string(),
                ))
                .unwrap(),
            ),
        };
        Box::pin(async move {
            Response::builder()
                .header(CONTENT_TYPE, "application/json")
                .status(status)
                .body(Body::from(body))
                .expect("unable to build http::Response")
        })
    }
}
impl From<FromHexError> for Error {
    fn from(_: FromHexError) -> Self {
        Error::BadSignature
    }
}

impl From<SignatureError> for Error {
    fn from(_: SignatureError) -> Self {
        Error::BadSignature
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerializeError(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Self {
        Error::BadBody
    }
}
