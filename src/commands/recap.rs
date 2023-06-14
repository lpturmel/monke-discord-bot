use super::winrate::WinRateError;
use crate::db::get_game_details_from_cache;
use crate::discord::{DiscordPayload, DiscordResponse, InteractionResponse, ResponseType};
use crate::error::Result;
use crate::riot::matches::Region as MatchesRegion;
use crate::riot::summoner::league::league_type_str;
use crate::riot::summoner::Region as SummonerRegion;
use crate::riot::Queue;
use crate::AppState;
use chrono::{TimeZone, Utc};
use chrono_tz::US::Eastern;
use std::cmp::Ordering;

pub async fn run(body: &DiscordPayload, state: &AppState) -> Result<DiscordResponse> {
    let working_tz = Eastern;
    let data = body.data.as_ref().ok_or(WinRateError::MissingData)?;
    let option = data.options.as_ref().ok_or(WinRateError::MissingOptions)?;
    let summoner_name = option
        .iter()
        .find(|o| o.name == "summoner")
        .ok_or(WinRateError::MissingSummonerOption)?
        .value
        .as_ref()
        .ok_or(WinRateError::MissingOptionValue)?;
    let yesterday = option
        .iter()
        .find(|o| o.name == "yesterday")
        .and_then(|o| o.value.as_ref().map(|v| v.as_bool().unwrap_or(false)));
    let now = Utc::now().with_timezone(&working_tz);

    let date = match yesterday {
        Some(true) => now - chrono::Duration::days(1),
        _ => now,
    };

    let summoner_name = summoner_name.as_str().unwrap();

    let summoner_data = state
        .riot_client
        .summoner(SummonerRegion::NA1)
        .get_by_name(summoner_name)
        .send()
        .await?;

    let queue_type = Queue::RankedSolo5x5;

    let start_time = date
        .timezone()
        .from_local_datetime(&date.date_naive().and_hms_opt(0, 0, 0).unwrap())
        .unwrap()
        .timestamp();

    let end_time = date
        .timezone()
        .from_local_datetime(&date.date_naive().and_hms_opt(23, 59, 59).unwrap())
        .unwrap()
        .timestamp();

    let game_ids = state
        .riot_client
        .matches(MatchesRegion::AMERICAS)
        .get_ids(&summoner_data.puuid)
        .count(25)
        .start_time(start_time)
        .end_time(end_time)
        .queue(queue_type.clone())
        .send();
    let league_details = state
        .riot_client
        .summoner(SummonerRegion::NA1)
        .get_league_details(&summoner_data.id)
        .send();

    let (game_ids, league_details) = futures::join!(game_ids, league_details);

    let game_ids = game_ids?;
    let league_details = league_details?;

    let league_details = league_details
        .iter()
        .find(|l| l.queue_type == league_type_str(&queue_type));

    let game_details = get_game_details_from_cache(&game_ids, state).await?;

    let user_games = game_details.iter().map(|game| {
        game.info
            .participants
            .iter()
            .find(|p| p.summoner_id == summoner_data.id)
            .ok_or(WinRateError::SummonerNotPartOfGame)
    });
    let user_games = user_games.collect::<std::result::Result<Vec<_>, WinRateError>>()?;
    let user_games_no_remake = user_games
        .iter()
        .filter(|p| !p.game_ended_in_early_surrender);
    let user_games_no_remake = user_games_no_remake.collect::<Vec<_>>();

    let game_count = user_games_no_remake.len();

    let won_games = user_games_no_remake.iter().filter(|p| p.win).count();
    let lost_games = game_count - won_games;

    let winrate = won_games as f32 / game_count as f32 * 100.0;

    let winrate_line = match winrate.is_nan() {
        true => "No games played".to_string(),
        false => format!("{}/{} **{:.2}%** winrate", won_games, lost_games, winrate),
    };
    let league_banner = match league_details {
        Some(league) => format!(
            "**{} {}** {} LP",
            league.tier, league.rank, league.league_points
        ),
        None => "Unranked".to_string(),
    };
    let mut banner = format!(
        "**{}** {}\n\nRecap for **{}**\n\n{}",
        summoner_data.name,
        league_banner,
        date.format("%A, %B %e, %Y"),
        winrate_line,
    );

    match won_games.cmp(&lost_games) {
        Ordering::Greater => {
            banner.push_str("\n\n**📈**");
        }
        Ordering::Less => {
            banner.push_str("\n\n**📉**");
        }
        Ordering::Equal => {}
    }
    let res = InteractionResponse::new(ResponseType::ChannelMessageWithSource, banner);
    Ok(res)
}

#[cfg(test)]
// write tests for start and end time
mod test {
    use chrono::{Local, TimeZone, Utc};
    use chrono_tz::US::Eastern;

    #[test]
    fn test_start_time() {
        // get now in local timezone
        let now = Local::now();
        let start_time = now
            .timezone()
            .from_local_datetime(&now.date_naive().and_hms_opt(0, 0, 0).unwrap())
            .unwrap();

        let start_time = start_time.timestamp();
        println!("start_time: {}", start_time);
        assert_eq!(start_time, 1686542400);
    }
    use crate::riot::matches::Region as MatchRegion;
    use crate::riot::Client;
    #[tokio::test]
    async fn get_games_by_date() {
        let key = std::env::var("RIOT_API_KEY").expect("No API key found");
        let client = Client::new(&key);

        let puuid =
            "tXgGun0-atXvYXFckcs4x_Y62WUykoSXSO04ew3BIPL0_MWG1Sx84CMLzoW1DggTyhha7Lo6devGLg";

        let now = Utc::now().with_timezone(&Eastern);

        let start_time = now
            .timezone()
            .from_local_datetime(&now.date_naive().and_hms_opt(0, 0, 0).unwrap())
            .unwrap();

        let end_time = now
            .timezone()
            .from_local_datetime(&now.date_naive().and_hms_opt(23, 59, 59).unwrap())
            .unwrap();

        // create a fixed timezone for the East
        let matches = client
            .matches(MatchRegion::AMERICAS)
            .get_ids(puuid)
            .start_time(start_time.timestamp())
            .end_time(end_time.timestamp())
            .count(25)
            .send()
            .await
            .unwrap();

        assert_eq!(matches.len(), 14);
    }
}
