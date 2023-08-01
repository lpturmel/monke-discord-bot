use crate::db::{get_league_details_from_cache, get_tft_details_from_cache};
use crate::discord::{
    DiscordPayload, DiscordResponse, GameType, InteractionResponse, ResponseType,
};
use crate::error::Result;
use crate::AppState;
use riot_sdk::league::summoner::league::league_type_str;
use riot_sdk::matches::Region as MatchesRegion;
use riot_sdk::summoner::Region as SummonerRegion;
use riot_sdk::Queue;
use std::fmt::Display;
use std::str::FromStr;

const WIN: &str = "✅";
const REMAKE: &str = "🔄";
const LOSS: &str = "❌";

#[derive(Debug)]
pub enum WinRateError {
    SummonerNotFound,
    MissingSummonerOption,
    MissingGameOption,
    MissingData,
    NoLeagueDetails,
    MissingOptions,
    MissingOptionValue,
    GetItemNoTableResults,
    GetItemNoResults,
    SummonerNotPartOfGame,
}

impl Display for WinRateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            WinRateError::SummonerNotFound => "Summoner not found",
            WinRateError::MissingSummonerOption => "Missing required summoner option",
            WinRateError::MissingGameOption => "Missing required game option",
            WinRateError::NoLeagueDetails => "No league details found (user not ranked)",
            WinRateError::MissingData => "Missing data (from discord)",
            WinRateError::MissingOptions => "Missing options (from discord)",
            WinRateError::MissingOptionValue => "Missing option value (from discord)",
            WinRateError::GetItemNoTableResults => "No table results from dynamodb",
            WinRateError::GetItemNoResults => "No results from dynamodb",
            WinRateError::SummonerNotPartOfGame => "Summoner not found in game participants",
        };
        write!(f, "{}", msg)
    }
}
pub async fn run(body: &DiscordPayload, state: &AppState) -> Result<DiscordResponse> {
    let data = body.data.as_ref().ok_or(WinRateError::MissingData)?;
    let option = data.options.as_ref().ok_or(WinRateError::MissingOptions)?;
    let summoner_name = option
        .iter()
        .find(|o| o.name == "summoner")
        .ok_or(WinRateError::MissingSummonerOption)?
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
    let summoner_name = summoner_name.as_str().unwrap();
    let game_type = GameType::from_str(game_type.as_str().unwrap())?;

    match game_type {
        GameType::League => run_league(summoner_name, state).await,
        GameType::Tft => run_tft(summoner_name, state).await,
    }
}

async fn run_league(summoner_name: &str, state: &AppState) -> Result<DiscordResponse> {
    let summoner_data = state
        .league_client
        .summoner(SummonerRegion::NA1)
        .get_by_name(summoner_name)
        .send()
        .await?;

    let queue_type = Queue::RankedSolo5x5;

    let game_ids = state
        .league_client
        .matches(MatchesRegion::AMERICAS)
        .get_ids(&summoner_data.puuid)
        .count(10)
        .queue(queue_type.clone())
        .send();
    let league_details = state
        .league_client
        .summoner(SummonerRegion::NA1)
        .get_league_details(&summoner_data.id)
        .send();

    let (game_ids, league_details) = futures::join!(game_ids, league_details);

    let game_ids = game_ids?;
    let league_details = league_details?;

    let league_details = league_details
        .iter()
        .find(|l| l.queue_type == league_type_str(&queue_type));

    let league_banner = match league_details {
        Some(l) => {
            let season_winrate = l.wins as f32 / (l.wins + l.losses) as f32 * 100.0;
            let mut league_banner = format!(
                "[**{} {}**] {} LP {}/{} ({:.2}%)",
                l.tier.as_ref().unwrap(),
                l.rank.as_ref().unwrap(),
                l.league_points,
                l.wins,
                l.losses,
                season_winrate
            );

            if l.hot_streak {
                league_banner.push_str(" 🔥");
            }
            league_banner
        }
        None => "Unranked".to_string(),
    };

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

    let game_lines = user_games
        .iter()
        .map(|p| match p.game_ended_in_early_surrender {
            true => print_game_line(true, &p.champion_name, 0, 0, 0, false),
            false => print_game_line(false, &p.champion_name, p.kills, p.deaths, p.assists, p.win),
        })
        .collect::<String>();

    let winrate = won_games as f32 / game_count as f32 * 100.0;

    let res = InteractionResponse::new(
        ResponseType::ChannelMessageWithSource,
        format!(
            "** --- League --- **\n\n**{}** {}\n\n[{}]: {:.2}% in last {} game(s)\n{}",
            summoner_data.name,
            league_banner,
            queue_type.friendly_name(),
            winrate,
            game_count,
            game_lines
        ),
    );
    Ok(res)
}
async fn run_tft(summoner_name: &str, state: &AppState) -> Result<DiscordResponse> {
    let summoner_data = state
        .tft_client
        .summoner(SummonerRegion::NA1)
        .get_by_name(summoner_name)
        .send()
        .await?;

    let queue_type = Queue::TFTRanked;

    let game_ids = state
        .tft_client
        .matches(MatchesRegion::AMERICAS)
        .get_ids(&summoner_data.puuid)
        .count(10)
        .send();
    let league_details = state
        .tft_client
        .summoner(SummonerRegion::NA1)
        .get_league_details(&summoner_data.id)
        .send();

    let (game_ids, league_details) = futures::join!(game_ids, league_details);

    let game_ids = game_ids?;
    let league_details = league_details?;

    let league_details = league_details
        .iter()
        .find(|l| l.queue_type == league_type_str(&queue_type));

    let league_banner = match league_details {
        Some(l) => {
            let season_winrate = l.wins as f32 / (l.wins + l.losses) as f32 * 100.0;
            let tier = l.tier.as_ref().unwrap();
            let rank = l.rank.as_ref().unwrap();
            let lp = l.league_points.as_ref().unwrap();
            let mut league_banner = format!(
                "[**{} {}**] {} LP {}/{} ({:.2}%)",
                tier, rank, lp, l.wins, l.losses, season_winrate
            );

            if l.hot_streak.unwrap() {
                league_banner.push_str(" 🔥");
            }
            league_banner
        }
        None => "Unranked".to_string(),
    };

    let game_details = get_tft_details_from_cache(&game_ids, state).await?;

    let game_count = game_details.len();

    let won_games = game_details
        .iter()
        .filter(|g| {
            g.info
                .participants
                .iter()
                .find(|p| p.puuid == summoner_data.puuid)
                .map(|p| p.placement <= 4)
                .unwrap_or(false)
        })
        .count();

    let game_lines = game_details
        .iter()
        .map(|g| {
            let p = g
                .info
                .participants
                .iter()
                .find(|p| p.puuid == summoner_data.puuid)
                .unwrap();
            let placement = match p.placement {
                1 => "🥇".to_string(),
                2 => "🥈".to_string(),
                3 => "🥉".to_string(),
                _ => format!("#{}", p.placement),
            };
            format!(
                "\n{}\t[{}]\n",
                placement,
                Queue::from(g.info.queue_id).friendly_name()
            )
        })
        .collect::<String>();

    let winrate = won_games as f32 / game_count as f32 * 100.0;

    let res = InteractionResponse::new(
        ResponseType::ChannelMessageWithSource,
        format!(
            "** --- TFT --- **\n\n**{}** {}\n\n{:.2}% in last {} game(s)\n{}\n\n",
            summoner_data.name, league_banner, winrate, game_count, game_lines
        ),
    );
    Ok(res)
}
pub fn print_game_line(
    is_remake: bool,
    champion_name: &str,
    kills: i64,
    deaths: i64,
    assists: i64,
    win: bool,
) -> String {
    let win_str = if is_remake {
        REMAKE
    } else if win {
        WIN
    } else {
        LOSS
    };
    let kda_num = get_numeric_kda(kills, deaths, assists);
    let kda_str = match kda_num {
        x if x == f32::INFINITY => "Perfect".to_string(),
        _ => format!("{:.2}", kda_num),
    };
    format!(
        "\n{} - {} {}/{}/{} **{}** KDA\n",
        win_str, champion_name, kills, deaths, assists, kda_str
    )
}

fn get_numeric_kda(kills: i64, deaths: i64, assists: i64) -> f32 {
    if deaths == 0 {
        f32::INFINITY
    } else {
        (kills + assists) as f32 / deaths as f32
    }
}
