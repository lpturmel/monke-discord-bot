use chrono::{TimeZone, Utc};
use chrono_tz::US::Eastern;
use lambda_runtime::LambdaEvent;
use lp_db::GameType;
use riot_sdk::summoner;
use serde_json::Value;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    tracing_subscriber::fmt()
        // .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .init();

    let func = lambda_runtime::service_fn(handler);

    lambda_runtime::run(func).await?;

    Ok(())
}
async fn handler(_e: LambdaEvent<Value>) -> Result<(), lambda_runtime::Error> {
    let now = Utc::now().with_timezone(&Eastern);

    let start_time = now
        .timezone()
        .from_local_datetime(&now.date_naive().and_time(now.time()))
        .unwrap()
        .timestamp();
    let config = aws_config::load_from_env().await;
    let league_client =
        riot_sdk::LeagueClient::new(&std::env::var("RIOT_API_KEY").expect("RIOT_API_KEY not set"));
    let tft_client = riot_sdk::TftClient::new(
        &std::env::var("TFT_RIOT_API_KEY").expect("TFT_RIOT_API_KEY not set"),
    );
    let lp_db_client = lp_db::Client::new(
        &std::env::var("LP_DB_TABLE_NAME").expect("LP_DB_TABLE_NAME not set"),
        &config,
    );
    let league_summs = lp_db_client
        .tracking(GameType::League)
        .list()
        .send()
        .await
        .unwrap();
    let tft_summs = lp_db_client
        .tracking(GameType::Tft)
        .list()
        .send()
        .await
        .unwrap();

    let league_summ_details = league_summs
        .iter()
        .map(|s| {
            let client = league_client.clone();
            let id = s.summoner_id();
            tokio::spawn(async move {
                sleep(Duration::from_millis(100)).await;
                client
                    .summoner(summoner::Region::NA1)
                    .get_league_details(&id)
                    .send()
                    .await
            })
        })
        .collect::<Vec<_>>();
    let tft_summ_details = tft_summs
        .iter()
        .map(|s| {
            let client = tft_client.clone();
            let id = s.summoner_id();
            tokio::spawn(async move {
                sleep(Duration::from_millis(100)).await;
                client
                    .summoner(summoner::Region::NA1)
                    .get_league_details(&id)
                    .send()
                    .await
            })
        })
        .collect::<Vec<_>>();

    let league_results: Result<Vec<_>, _> =
        futures::future::try_join_all(league_summ_details).await;
    let tft_results: Result<Vec<_>, _> = futures::future::try_join_all(tft_summ_details).await;

    match league_results {
        Ok(res) => {
            for r in res {
                match r {
                    Ok(league) => {
                        let queue_details =
                            league.iter().find(|l| l.queue_type == "RANKED_SOLO_5x5");
                        if let Some(queue_details) = queue_details {
                            let _ = lp_db_client
                                .league_points(lp_db::GameType::League)
                                .add()
                                .id(&queue_details.summoner_id)
                                .tier(queue_details.tier.as_ref().unwrap())
                                .rank(queue_details.rank.as_ref().unwrap())
                                .timestamp(start_time)
                                .league_points(queue_details.league_points as i32)
                                .wins(queue_details.wins)
                                .losses(queue_details.losses)
                                .send()
                                .await;
                        } else {
                            tracing::error!("User is not ranked in solo queue, skipping...");
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error getting league details for League of Legends: {:?}",
                            e
                        );
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!(
                "Error getting league details for League of Legends: {:?}",
                e
            );
        }
    }
    match tft_results {
        Ok(res) => {
            for r in res {
                match r {
                    Ok(league) => {
                        let queue_details = league.iter().find(|l| l.queue_type == "RANKED_TFT");
                        if let Some(queue_details) = queue_details {
                            let _ = lp_db_client
                                .league_points(lp_db::GameType::Tft)
                                .add()
                                .id(&queue_details.summoner_id)
                                .tier(queue_details.tier.as_ref().unwrap())
                                .rank(queue_details.rank.as_ref().unwrap())
                                .timestamp(start_time)
                                .league_points(
                                    (*queue_details.league_points.as_ref().unwrap()) as i32,
                                )
                                .wins(queue_details.wins)
                                .losses(queue_details.losses)
                                .send()
                                .await;
                        } else {
                            tracing::error!("User is not ranked TFT, skipping...");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error getting league details for TFT: {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Error getting league details for TFT: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use aws_config::profile::ProfileFileCredentialsProvider;
    use chrono::{TimeZone, Utc};
    use chrono_tz::US::Eastern;
    use riot_sdk::summoner::Region;

    #[test]
    fn test_current_time() {
        let now = Utc::now().with_timezone(&Eastern);
        let start_time = now
            .timezone()
            .from_local_datetime(&now.date_naive().and_time(now.time()))
            .unwrap()
            .timestamp();
        println!("{}", start_time);
    }
    #[test]
    fn test_replace() {
        let str = "SUMMONER#TFT#pPYzaoqMQ9ebkwi0o1nPQ6_5mIdeX-ZG3IFN-QIiQUfS_bo";
        let ident = "TFT#";
        let new_str = str.replace(format!("SUMMONER#{}", ident).as_str(), "");
        assert_eq!(new_str, "pPYzaoqMQ9ebkwi0o1nPQ6_5mIdeX-ZG3IFN-QIiQUfS_bo");
    }
    #[tokio::test]
    async fn test_handler() {
        let profile_name = "lpturmel";

        // This credentials provider will load credentials from ~/.aws/credentials.
        let credentials_provider = ProfileFileCredentialsProvider::builder()
            .profile_name(profile_name)
            .build();

        // Load the credentials
        let config = aws_config::from_env()
            .credentials_provider(credentials_provider)
            .load()
            .await;

        let client = lp_db::Client::new("monke-league-point-service-table", &config);

        let tracking_items = client
            .tracking(lp_db::GameType::League)
            .list()
            .send()
            .await
            .unwrap();

        let tft_client = riot_sdk::LeagueClient::new(
            &std::env::var("RIOT_API_KEY").expect("RIOT_API_KEY not set"),
        );
        for item in tracking_items {
            println!("{:?}", item);
            println!("ID for {}: {}", item.summoner_name, item.summoner_id());
            let league_details = tft_client
                .summoner(Region::NA1)
                .get_league_details(&item.summoner_id())
                .send()
                .await
                .unwrap();
            println!("{:?}", league_details);
        }
    }
}
