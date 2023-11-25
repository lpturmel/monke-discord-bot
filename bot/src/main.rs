use self::discord::{
    verify_sig, DiscordPayload, DiscordResponse, InteractionResponse, ResponseType,
};
use self::error::Error as AppError;
use crate::discord::Command;
use aws_sdk_dynamodb::Client as DynamoClient;
use lambda_http::{run, service_fn, Body, IntoResponse, Request, Response};
use lambda_runtime::Error;
use std::future::Future;
use std::pin::Pin;

pub mod commands;
pub mod db;
pub mod discord;
pub mod error;

pub type ResponseFuture = Pin<Box<dyn Future<Output = Response<Body>> + Send>>;

pub struct AppState {
    account_client: riot_sdk::AccountClient,
    league_client: riot_sdk::LeagueClient,
    tft_client: riot_sdk::TftClient,
    db_client: DynamoClient,
    lp_db_client: lp_db::Client,
}
async fn wrapper_fn(event: Request, state: &AppState) -> Result<Response<Body>, Error> {
    let res = function_handler(event, state).await;
    match res {
        Ok(res) => Ok(res.into_response().await),
        Err(err) => {
            tracing::error!("Error: {:?}", err);
            Ok(err.into_response().await)
        }
    }
}
async fn function_handler(event: Request, state: &AppState) -> Result<DiscordResponse, AppError> {
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
            let command = Command::parse_from_str(&int_data.id).ok_or(AppError::BadCommand)?;

            match command {
                Command::Winrate => commands::winrate::run(&body, state).await?,
                Command::Recap => commands::recap::run(&body, state).await?,
                Command::Track => commands::track::run(&body, state).await?,
                Command::Untrack => commands::untrack::run(&body, state).await?,
                Command::List => commands::list::run(&body, state).await?,
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

    let config = aws_config::load_from_env().await;
    let client = DynamoClient::new(&config);

    let state = AppState {
        account_client: riot_sdk::AccountClient::new(
            &std::env::var("RIOT_API_KEY").expect("RIOT_API_KEY not set"),
        ),
        league_client: riot_sdk::LeagueClient::new(
            &std::env::var("RIOT_API_KEY").expect("RIOT_API_KEY not set"),
        ),
        tft_client: riot_sdk::TftClient::new(
            &std::env::var("TFT_RIOT_API_KEY").expect("TFT_RIOT_API_KEY not set"),
        ),
        lp_db_client: lp_db::Client::new(
            &std::env::var("LP_DB_TABLE_NAME").expect("LP_DB_TABLE_NAME not set"),
            &config,
        ),
        db_client: client,
    };

    run(service_fn(|event: Request| async {
        wrapper_fn(event, &state).await
    }))
    .await
}
