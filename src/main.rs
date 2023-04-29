use lambda_http::{run, service_fn, Body, IntoResponse, Request, Response};
use lambda_runtime::Error;
use std::future::Future;
use std::pin::Pin;

use crate::discord::Command;

use self::discord::{
    verify_sig, DiscordPayload, DiscordResponse, InteractionResponse, ResponseType,
};
use self::error::Error as AppError;

pub mod commands;
pub mod discord;
pub mod error;
pub mod riot;

pub type ResponseFuture = Pin<Box<dyn Future<Output = Response<Body>> + Send>>;

async fn wrapper_fn(event: Request) -> Result<Response<Body>, Error> {
    let res = function_handler(event).await;
    match res {
        Ok(res) => Ok(res.into_response().await),
        Err(err) => Ok(err.into_response().await),
    }
}
async fn function_handler(event: Request) -> Result<DiscordResponse, AppError> {
    let signature = event
        .headers()
        .get("x-signature-ed25519")
        .ok_or(AppError::MissingSignature)?;

    let timestamp = event
        .headers()
        .get("x-signature-timestamp")
        .ok_or(AppError::MissingTimestamp)?;

    // convert body to string
    let body = event.body().as_ref().to_vec();
    let body_str = String::from_utf8(body)?;

    let valid_req = verify_sig(
        &body_str,
        signature.to_str().unwrap(),
        timestamp.to_str().unwrap(),
        "a5d643032b33dc867656fffd3adce590c07e9759e15ef5655a0f46c991dc78b6",
    )?;

    if !valid_req {
        return Err(AppError::BadSignature);
    }

    let body = serde_json::from_str::<DiscordPayload>(&body_str)?;

    let res = match body.r#type {
        1 => commands::ping::run(&body).await?,
        2 => {
            let int_data = &body.data.as_ref().ok_or(AppError::BadCommand)?;
            let command = Command::from_str(&int_data.id).ok_or(AppError::BadCommand)?;

            let key = std::env::var("RIOT_API_KEY").expect("RIOT_API_KEY not set");
            let riot_client = riot::Client::new(&key);
            match command {
                Command::Winrate => commands::winrate::run(&body, &riot_client).await?,
            }
        }
        _ => InteractionResponse::new(ResponseType::Pong, "Bad request type"),
    };

    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(wrapper_fn)).await
}
