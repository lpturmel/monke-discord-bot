use super::winrate::WinRateError;
use crate::db::{get_league_details_from_cache, get_tft_details_from_cache};
use crate::discord::{
    DiscordPayload, DiscordResponse, GameType, InteractionResponse, ResponseType,
};
use crate::error::Result;
use crate::AppState;
use chrono::{TimeZone, Utc};
use chrono_tz::US::Eastern;
use lp_db::GameType as DbGameType;
use riot_sdk::account::AccountRegion;
use riot_sdk::league::summoner::league::league_type_str;
use riot_sdk::matches::Region as MatchesRegion;
use riot_sdk::summoner::Region as SummonerRegion;
use riot_sdk::{PlayerRank, Queue};
use std::cmp::Ordering;
use std::str::FromStr;

const WORKING_TZ: chrono_tz::Tz = Eastern;
pub async fn run(body: &DiscordPayload, state: &AppState) -> Result<DiscordResponse> {
    let data = body.data.as_ref().ok_or(WinRateError::MissingData)?;
    let option = data.options.as_ref().ok_or(WinRateError::MissingOptions)?;
    let game_name = option
        .iter()
        .find(|o| o.name == "game_name")
        .ok_or(WinRateError::MissingSummonerOption)?
        .value
        .as_ref()
        .ok_or(WinRateError::MissingOptionValue)?;
    let tag_line = option
        .iter()
        .find(|o| o.name == "tag_line")
        .ok_or(WinRateError::MissingTagLineOption)?
        .value
        .as_ref()
        .ok_or(WinRateError::MissingOptionValue)?;
    let game_type = option
        .iter()
        .find(|o| o.name == "game")
        .ok_or(WinRateError::MissingGameOption)?
        .value
        .as_ref()
        .ok_or(WinRateError::MissingOptionValue)?;
    let yesterday = option
        .iter()
        .find(|o| o.name == "yesterday")
        .and_then(|o| o.value.as_ref().map(|v| v.as_bool().unwrap_or(false)));

    let tag_line = tag_line.as_str().unwrap();
    let game_name = game_name.as_str().unwrap();
    let game_type = GameType::from_str(game_type.as_str().unwrap())?;

    let riot_id_data = state
        .account_client
        .account(AccountRegion::AMERICAS)
        .get_by_riot_id(game_name, tag_line)
        .send()
        .await
        .map_err(|_| WinRateError::RiotIdNotFound)?;
    let riot_id = format!("{}#{}", riot_id_data.game_name, riot_id_data.tag_line);

    match game_type {
        GameType::League => run_league(&riot_id, &riot_id_data.puuid, yesterday, state).await,
        GameType::Tft => run_tft(&riot_id, &riot_id_data.puuid, yesterday, state).await,
    }
}

async fn run_league(
    riot_id: &str,
    puuid: &str,
    yesterday: Option<bool>,
    state: &AppState,
) -> Result<DiscordResponse> {
    let now = Utc::now().with_timezone(&WORKING_TZ);
    let date = match yesterday {
        Some(true) => now - chrono::Duration::days(1),
        _ => now,
    };
    let summoner_data = state
        .league_client
        .summoner(SummonerRegion::NA1)
        .get_by_puuid(puuid)
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
        .league_client
        .matches(MatchesRegion::AMERICAS)
        .get_ids(&summoner_data.puuid)
        .count(25)
        .start_time(start_time)
        .end_time(end_time)
        .queue(queue_type.clone())
        .send();
    let league_details = state
        .league_client
        .summoner(SummonerRegion::NA1)
        .get_league_details(&summoner_data.id)
        .send();
    let daily_lp = state
        .lp_db_client
        .league_points(DbGameType::League)
        .get_between()
        .id(&summoner_data.id)
        .start_time(start_time)
        .end_time(end_time)
        .send();

    let (game_ids, league_details, daily_lp) = futures::join!(game_ids, league_details, daily_lp);

    let game_ids = game_ids?;
    let league_details = league_details?;
    let daily_lp = daily_lp?;

    let league_details = league_details
        .iter()
        .find(|l| l.queue_type == league_type_str(&queue_type));

    let game_details = get_league_details_from_cache(&game_ids, state).await?;

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
        Some(league) => {
            let rank = PlayerRank::parse_str(
                league.tier.as_ref().unwrap(),
                league.rank.as_ref().unwrap(),
                (league.league_points).try_into().unwrap(),
            );
            rank.formatted_rank()
        }
        None => "Unranked".to_string(),
    };
    let mut banner = format!(
        "** --- League ---**\n\n**{}** {}\n\nRecap for **{}**\n\n{}",
        riot_id,
        league_banner,
        date.format("%A, %B %e, %Y"),
        winrate_line,
    );

    if let Some(league_details) = league_details {
        if let Some(daily_lp) = daily_lp {
            let mut daily_lp_iter = daily_lp.iter();
            let morning_lp_snapshot = daily_lp_iter.next();
            if let Some(morning_lp_snapshot) = morning_lp_snapshot {
                let evening_lp_snapshot = daily_lp_iter.next();

                let start_rank = PlayerRank::parse_str(
                    &morning_lp_snapshot.tier,
                    &morning_lp_snapshot.rank,
                    morning_lp_snapshot.league_points,
                );

                match evening_lp_snapshot {
                    Some(evening_lp_snapshot) => {
                        let current_rank = match yesterday {
                            Some(true) => PlayerRank::parse_str(
                                &evening_lp_snapshot.tier,
                                &evening_lp_snapshot.rank,
                                evening_lp_snapshot.league_points,
                            ),
                            _ => PlayerRank::parse_str(
                                league_details.tier.as_ref().unwrap(),
                                league_details.rank.as_ref().unwrap(),
                                league_details.league_points as i32,
                            ),
                        };

                        let rank_change = start_rank.points_difference(&current_rank);

                        let rank_change_str = match rank_change.cmp(&0) {
                            Ordering::Greater => format!("**+{}**", rank_change),
                            Ordering::Less => format!("**{}**", rank_change),
                            Ordering::Equal => format!("**{}**", rank_change),
                        };
                        banner.push_str(&format!(
                            "\n\n`LP DAILY RECAP`\n\nstart\t{}\nend\t  {}\nGain {}",
                            start_rank.formatted_rank(),
                            current_rank.formatted_rank(),
                            rank_change_str
                        ));
                    }
                    None => {
                        // banner.push_str("\n\n*No evening info, LP info will be available tomorrow*")
                        let current_rank = PlayerRank::parse_str(
                            league_details.tier.as_ref().unwrap(),
                            league_details.rank.as_ref().unwrap(),
                            league_details.league_points as i32,
                        );

                        let rank_change = start_rank.points_difference(&current_rank);

                        let rank_change_str = match rank_change.cmp(&0) {
                            Ordering::Greater => format!("**+{}**", rank_change),
                            Ordering::Less => format!("**{}**", rank_change),
                            Ordering::Equal => format!("**{}**", rank_change),
                        };
                        banner.push_str(&format!(
                            "\n\n`LP RECAP as of {}`\n\nstart\t{}\nend\t  {}\nGain {}",
                            date.format("%A, %B %e, %Y %H:%M:%S"),
                            start_rank.formatted_rank(),
                            current_rank.formatted_rank(),
                            rank_change_str
                        ));
                    }
                }
            }
        }
    }

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
async fn run_tft(
    riot_id: &str,
    puuid: &str,
    yesterday: Option<bool>,
    state: &AppState,
) -> Result<DiscordResponse> {
    let now = Utc::now().with_timezone(&WORKING_TZ);
    let date = match yesterday {
        Some(true) => now - chrono::Duration::days(1),
        _ => now,
    };
    let summoner_data = state
        .league_client
        .summoner(SummonerRegion::NA1)
        .get_by_puuid(puuid)
        .send()
        .await?;

    let queue_type = Queue::TFTRanked;

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
        .tft_client
        .matches(MatchesRegion::AMERICAS)
        .get_ids(&summoner_data.puuid)
        .count(25)
        .start_time(start_time)
        .end_time(end_time)
        .send();
    let league_details = state
        .tft_client
        .summoner(SummonerRegion::NA1)
        .get_league_details(&summoner_data.id)
        .send();
    let daily_lp = state
        .lp_db_client
        .league_points(DbGameType::Tft)
        .get_between()
        .id(&summoner_data.id)
        .start_time(start_time)
        .end_time(end_time)
        .send();

    let (game_ids, league_details, daily_lp) = futures::join!(game_ids, league_details, daily_lp);

    let game_ids = game_ids?;
    let league_details = league_details?;
    let daily_lp = daily_lp?;

    let league_details = league_details
        .iter()
        .find(|l| l.queue_type == league_type_str(&queue_type));

    let game_details = get_tft_details_from_cache(&game_ids, state).await?;

    let user_games = game_details.iter().map(|game| {
        game.info
            .participants
            .iter()
            .find(|p| p.puuid == summoner_data.puuid)
            .ok_or(WinRateError::SummonerNotPartOfGame)
    });
    let user_games = user_games.collect::<std::result::Result<Vec<_>, WinRateError>>()?;

    let game_count = user_games.len();

    let won_games = user_games.iter().filter(|p| p.placement <= 4).count();
    let lost_games = game_count - won_games;

    let winrate = won_games as f32 / game_count as f32 * 100.0;

    let winrate_line = match winrate.is_nan() {
        true => "No games played".to_string(),
        false => format!("{}/{} **{:.2}%** winrate", won_games, lost_games, winrate),
    };
    let league_banner = match league_details {
        Some(league) => {
            let rank = PlayerRank::parse_str(
                league.tier.as_ref().unwrap(),
                league.rank.as_ref().unwrap(),
                (*league.league_points.as_ref().unwrap())
                    .try_into()
                    .unwrap(),
            );
            rank.formatted_rank()
        }
        None => "Unranked".to_string(),
    };
    let mut banner = format!(
        "** --- TFT --- **\n\n**{}** {}\n\nRecap for **{}**\n\n{}",
        riot_id,
        league_banner,
        date.format("%A, %B %e, %Y"),
        winrate_line,
    );

    if let Some(league_details) = league_details {
        if let Some(daily_lp) = daily_lp {
            let mut daily_lp_iter = daily_lp.iter();
            let morning_lp_snapshot = daily_lp_iter.next();
            if let Some(morning_lp_snapshot) = morning_lp_snapshot {
                let evening_lp_snapshot = daily_lp_iter.next();

                let start_rank = PlayerRank::parse_str(
                    &morning_lp_snapshot.tier,
                    &morning_lp_snapshot.rank,
                    morning_lp_snapshot.league_points,
                );
                let current_rank = match yesterday {
                    Some(true) => PlayerRank::parse_str(
                        &evening_lp_snapshot.unwrap().tier,
                        &evening_lp_snapshot.unwrap().rank,
                        evening_lp_snapshot.unwrap().league_points,
                    ),
                    _ => PlayerRank::parse_str(
                        league_details.tier.as_ref().unwrap(),
                        league_details.rank.as_ref().unwrap(),
                        (*league_details.league_points.as_ref().unwrap()) as i32,
                    ),
                };

                let rank_change = start_rank.points_difference(&current_rank);

                let rank_change_str = match rank_change.cmp(&0) {
                    Ordering::Greater => format!("**+{}**", rank_change),
                    Ordering::Less => format!("**{}**", rank_change),
                    Ordering::Equal => format!("**{}**", rank_change),
                };
                banner.push_str(&format!(
                    "\n\n`LP INFO`\n\nstart\t{}\nend\t  {}\nGain {}",
                    start_rank.formatted_rank(),
                    current_rank.formatted_rank(),
                    rank_change_str
                ));
            }
        }
    }

    match won_games.cmp(&lost_games) {
        Ordering::Greater => {
            banner.push_str("\n\n**📈**");
        }
        Ordering::Less => {
            banner.push_str("\n\n**📉**");
        }
        Ordering::Equal => {}
    }

    banner.push_str("\n\n*ℹ️  Recap includes normal games in TFT because of API limitations*");
    let res = InteractionResponse::new(ResponseType::ChannelMessageWithSource, banner);
    Ok(res)
}
