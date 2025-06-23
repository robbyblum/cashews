use std::{
    collections::{BTreeSet, HashSet},
    time::Duration,
};

use chron_db::{
    ChronDb,
    derived::{DbGame, DbGameSaveModel, GetGamesQuery},
    models::{EntityKind, EntityVersion},
};
use futures::StreamExt;
use serde::Deserialize;
use time::OffsetDateTime;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::models::{MmolbGame, MmolbGameEvent, MmolbTeam};

use super::{IntervalWorker, WorkerContext};

pub struct PollAllScheduledGames;
pub struct PollTodayGames;
pub struct PollLiveGames;
pub struct PollFinishedGamesFromFeed;

impl IntervalWorker for PollAllScheduledGames {
    fn interval() -> tokio::time::Interval {
        interval(Duration::from_secs(60 * 60))
    }

    async fn tick(&mut self, ctx: &mut super::WorkerContext) -> anyhow::Result<()> {
        fetch_all_schedules(ctx, 1).await
    }
}

#[derive(Deserialize)]
struct TodayGame {
    game_id: String,
}

impl IntervalWorker for PollTodayGames {
    fn interval() -> tokio::time::Interval {
        interval(Duration::from_secs(1 * 60))
    }

    async fn tick(&mut self, ctx: &mut WorkerContext) -> anyhow::Result<()> {
        let time = ctx.try_update_time().await?;
        match time.season_status.to_ascii_lowercase().as_str() {
            "preseason" | "holiday" => {
                info!("season status is {}, skipping", time.season_status);
                return Ok(());
            }
            _ => {}
        }

        let resp = ctx
            .client
            .fetch("https://mmolb.com/api/today-games")
            .await?;
        let today_games = resp.parse::<Vec<TodayGame>>()?;
        ctx.process_many_with_progress(
            today_games.into_iter().map(|x| x.game_id),
            5,
            "fetch new games",
            fetch_game_if_new,
        )
        .await;
        Ok(())
    }
}

impl IntervalWorker for PollLiveGames {
    fn interval() -> tokio::time::Interval {
        interval(Duration::from_secs(30))
    }

    async fn tick(&mut self, ctx: &mut WorkerContext) -> anyhow::Result<()> {
        let time = ctx.try_update_time().await?;

        let known_games_today = ctx
            .db
            .get_games(GetGamesQuery {
                count: 99999, // ignore pagination for now?
                season: Some(time.season_number),
                day: None,
                order: chron_db::queries::SortOrder::Asc,
                page: None,
                team: None,
            })
            .await?;
        let live_games = known_games_today
            .items
            .into_iter()
            .filter(|x| x.state != "Complete")
            .collect::<Vec<_>>();
        info!("found {} live games in db", live_games.len());

        ctx.process_many_with_progress(live_games, 5, "fetch live games", poll_live_game).await;

        Ok(())
    }
}

#[derive(Deserialize)]
// using different struct type here for resilience
struct TeamWithOnlyFeed {
    #[serde(rename = "Feed")]
    feed: Option<Vec<FeedItem>>,
}

#[derive(Deserialize)]
struct FeedItem {
    #[serde(rename = "type")]
    kind: String,
    text: String,
    links: Vec<FeedItemLink>,
}

#[derive(Deserialize)]
struct FeedItemLink {
    #[serde(rename = "type")]
    kind: String,
    id: String,
}

impl IntervalWorker for PollFinishedGamesFromFeed {
    fn interval() -> tokio::time::Interval {
        interval(Duration::from_secs(5 * 60))
    }

    async fn tick(&mut self, ctx: &mut WorkerContext) -> anyhow::Result<()> {
        let teams = ctx.db.get_all_latest(EntityKind::Team).await?;

        let mut game_ids = BTreeSet::new();
        // just eat every game id we can find
        for team in teams {
            let parsed = team.parse::<TeamWithOnlyFeed>()?;
            for item in parsed.feed.unwrap_or_default() {
                for link in item.links {
                    if link.kind == "game" {
                        game_ids.insert(link.id);
                    }
                }
            }
        }

        ctx.process_many_with_progress(game_ids, 5, "fetch finished games from feed", fetch_game_if_not_completed)
            .await;

        Ok(())
    }
}

async fn poll_schedule_for_team_for_new_games(
    ctx: &WorkerContext,
    id: String,
) -> anyhow::Result<()> {
    let url = format!("https://mmolb.com/api/team-schedule/{}", &id);
    let resp = ctx.fetch_and_save(url, EntityKind::Schedule, &id).await?;

    let sched = resp.parse::<TeamSchedule>()?;
    for ele in sched.games {
        if let Some(game_id) = ele.game_id {
            let knows_about_game = ctx
                .db
                .get_latest(EntityKind::Game, &game_id)
                .await?
                .is_some();
            if !knows_about_game {
                poll_game_by_id(ctx, game_id).await?;
            }
        }
    }
    Ok(())
}

async fn fetch_game_if_new(ctx: &WorkerContext, game_id: String) -> anyhow::Result<()> {
    let knows_about_game = ctx
        .db
        .get_latest(EntityKind::Game, &game_id)
        .await?
        .is_some();
    if !knows_about_game {
        poll_game_by_id(ctx, game_id).await?;
    }

    Ok(())
}

async fn fetch_game_if_not_completed(ctx: &WorkerContext, game_id: String) -> anyhow::Result<()> {
    let known_game = ctx.db.get_latest(EntityKind::Game, &game_id).await?;
    let should_poll = if let Some(game) = known_game {
        let game: MmolbGame = game.parse()?;
        game.state != "Complete"
    } else {
        true
    };

    if should_poll {
        poll_game_by_id(ctx, game_id).await?;
    }

    Ok(())
}

async fn poll_schedule_for_team_for_all_games(
    ctx: &WorkerContext,
    id: String,
) -> anyhow::Result<()> {
    let url = format!("https://mmolb.com/api/team-schedule/{}", &id);
    let resp = ctx.fetch_and_save(url, EntityKind::Schedule, &id).await?;

    let sched = resp.parse::<TeamSchedule>()?;
    for ele in sched.games {
        if let Some(game_id) = ele.game_id {
            poll_game_by_id(ctx, game_id).await?;
        }
    }
    Ok(())
}

async fn try_get_team(
    db: &ChronDb,
    team_id: &str,
    timestamp: &OffsetDateTime,
) -> anyhow::Result<Option<MmolbTeam>> {
    Ok(db
        .get_entity_at(EntityKind::Team, &team_id, timestamp)
        .await?
        .map(|x| x.parse())
        .transpose()?)
}

fn try_find_player_by_name(
    team: &MmolbTeam,
    player_name: &str,
    position_type: &str,
) -> Option<String> {
    let mut result: Option<&str> = None;
    for slot in &team.players {
        // todo: remove alloc?
        let full_name = format!("{} {}", slot.first_name, slot.last_name);
        if full_name == player_name && slot.position_type.as_deref().unwrap_or("") == position_type
        {
            if result.is_some() {
                // we found two valid players, abort
                return None;
            }
            result = Some(&slot.player_id);
        }
    }
    result.map(|x| x.to_string())
}

// todo: this is nasty
struct GenericGame<'a> {
    game_id: &'a str,
    season: i32,
    day: i32,
    home_team_id: &'a str,
    away_team_id: &'a str,
}
struct EnrichedGameEvent {
    pitcher_id: Option<String>,
    batter_id: Option<String>,
}

async fn save_game_events(
    ctx: &WorkerContext,
    timestamp: OffsetDateTime,
    game: &GenericGame<'_>,
    raw_events: &[serde_json::Value],
    start_idx: i32,
) -> anyhow::Result<()> {
    let away_team = try_get_team(&ctx.db, &game.away_team_id, &timestamp).await?;
    let home_team = try_get_team(&ctx.db, &game.home_team_id, &timestamp).await?;

    let mut indexes = Vec::new();
    let mut datas = Vec::new();
    let mut pitchers = Vec::new();
    let mut batters = Vec::new();
    for (idx, evt) in raw_events.iter().enumerate() {
        let absolute_idx = idx as i32 + start_idx;

        let enriched: Option<EnrichedGameEvent> = match MmolbGameEvent::deserialize(evt) {
            Ok(parsed_event) => {
                let (pitching_team, batting_team) = if parsed_event.inning_side == 0 {
                    (home_team.as_ref(), away_team.as_ref())
                } else {
                    (away_team.as_ref(), home_team.as_ref())
                };

                let pitcher_id = pitching_team
                    .zip(parsed_event.pitcher.as_ref())
                    .and_then(|(t, name)| try_find_player_by_name(t, name, "Pitcher"));
                let batter_id = batting_team
                    .zip(parsed_event.batter.as_ref())
                    .and_then(|(t, name)| try_find_player_by_name(t, name, "Batter"));
                Some(EnrichedGameEvent {
                    pitcher_id,
                    batter_id,
                })
            }
            Err(e) => {
                let s = serde_json::to_string(evt);
                warn!(
                    "couldn't parse game event {}/{} ({:?}): {:?}",
                    game.game_id, absolute_idx, s, e
                );
                None
            }
        };

        indexes.push(absolute_idx);
        datas.push(evt);
        pitchers.push(enriched.as_ref().and_then(|x| x.pitcher_id.clone()));
        batters.push(enriched.as_ref().and_then(|x| x.batter_id.clone()));
    }

    ctx.db
        .update_game_events(
            &game.game_id,
            game.season,
            game.day,
            &timestamp,
            &indexes,
            &datas,
            &pitchers,
            &batters,
        )
        .await?;
    Ok(())
}

async fn poll_live_game(ctx: &WorkerContext, game: DbGame) -> anyhow::Result<()> {
    let current_count = game.event_count;

    let url = format!(
        "https://mmolb.com/api/game/{}/live?after={}",
        game.game_id, current_count
    );
    let resp = ctx.client.fetch(&url).await?;

    let events = resp.parse::<LiveResponse>()?;

    let generic_game = GenericGame {
        away_team_id: &game.away_team_id,
        home_team_id: &game.home_team_id,
        game_id: &game.game_id,
        season: game.season,
        day: game.day,
    };
    save_game_events(
        ctx,
        resp.timestamp(),
        &generic_game,
        &events.entries,
        current_count,
    )
    .await?;

    fn is_game_over_event(e: &serde_json::Value) -> bool {
        // oh no
        if let Some(obj) = e.as_object() {
            if let Some(event_val) = obj.get("event") {
                if let Some(event_str) = event_val.as_str() {
                    if event_str == "Recordkeeping" || event_str == "GameOver" {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    let new_state = if events.entries.iter().any(is_game_over_event) {
        "Complete".to_string()
    } else {
        game.state
    };

    if let Some(last_update) = events.entries.last() {
        ctx.db
            .update_game(DbGameSaveModel {
                game_id: &game.game_id,
                season: game.season,
                day: game.day,
                home_team_id: &game.home_team_id,
                away_team_id: &game.away_team_id,
                state: &new_state,
                event_count: current_count + events.entries.len() as i32,
                last_update: Some(last_update),
            })
            .await?;
    }

    if new_state == "Complete" {
        // if the game just finished, poll the whole thing properly, which should fill in stats and such
        poll_game_by_id(ctx, game.game_id).await?;
    }
    Ok(())
}

async fn get_all_known_game_ids(ctx: &WorkerContext) -> anyhow::Result<HashSet<String>> {
    let preset_game_ids = include_str!("./game_ids.txt");

    let mut game_ids: HashSet<String> = preset_game_ids
        .split("\n")
        .map(|x| x.to_string())
        .filter(|x| !x.is_empty())
        .collect();

    game_ids.extend(ctx.db.get_all_entity_ids(EntityKind::Game).await?);
    Ok(game_ids)
}

async fn poll_game_by_id(ctx: &WorkerContext, id: String) -> anyhow::Result<()> {
    let url = format!("https://mmolb.com/api/game/{}", id);
    let resp = ctx.fetch_and_save(url, EntityKind::Game, &id).await?;

    let game: MmolbGame = resp.parse()?;
    process_game_data(ctx, &id, &game, &resp.timestamp()).await?;

    Ok(())
}

async fn process_game_data(
    ctx: &WorkerContext,
    id: &str,
    game: &MmolbGame,
    timestamp: &OffsetDateTime,
) -> anyhow::Result<()> {
    ctx.db
        .update_game(DbGameSaveModel {
            game_id: &id,
            season: game.season,
            day: game.day,
            home_team_id: &game.home_team_id,
            away_team_id: &game.away_team_id,
            state: &game.state,
            event_count: game.event_log.len() as i32,
            last_update: game.event_log.last(),
        })
        .await?;

    let generic_game = GenericGame {
        away_team_id: &game.away_team_id,
        home_team_id: &game.home_team_id,
        game_id: &id,
        season: game.season,
        day: game.day,
    };
    save_game_events(ctx, *timestamp, &generic_game, &game.event_log, 0).await?;

    if let Some(game_stats) = &game.stats {
        let mut stats = Vec::new();
        for (team_id, team_stats) in game_stats {
            for (player_id, player_stats) in team_stats {
                stats.push((team_id.as_str(), player_id.as_str(), player_stats));
            }
        }
        ctx.db
            .update_game_player_stats(&id, game.season, game.day, &stats)
            .await?;
    }

    Ok(())
}

pub async fn rebuild_games(ctx: &WorkerContext) -> anyhow::Result<()> {
    // get game ids separately because "all game objects" is gonna be massive
    let all_game_ids = ctx.db.get_all_entity_ids(EntityKind::Game).await?;

    ctx.process_many(all_game_ids, 20, rebuild_game).await;
    Ok(())
}

pub async fn rebuild_games_slow(ctx: &WorkerContext) -> anyhow::Result<()> {
    let count = ctx.db.get_version_count(EntityKind::Game).await?;
    let stream = ctx.db.get_all_versions_stream(EntityKind::Game).await?;

    stream
        .map(|v| rebuild_games_slow_inner(ctx, v))
        .buffer_unordered(10)
        .enumerate()
        .for_each(async |(i, res)| {
            if i % 1000 == 0 {
                info!("rebuild games: at {}/{}", i, count);
            }
            if let Err(e) = res {
                error!("error rebuilding: {:?}", e);
            }
        })
        .await;

    Ok(())
}

async fn rebuild_games_slow_inner(
    ctx: &WorkerContext,
    version: sqlx::Result<EntityVersion>,
) -> anyhow::Result<()> {
    let version = version?;
    let parsed = version.parse::<MmolbGame>()?;
    process_game_data(ctx, &version.entity_id, &parsed, &version.valid_from.0).await?;
    Ok(())
}

async fn rebuild_game(ctx: &WorkerContext, game_id: String) -> anyhow::Result<()> {
    info!("rebuilding game {}", game_id);
    let game_data = ctx.db.get_latest(EntityKind::Game, &game_id).await?;
    if let Some(game_data) = game_data {
        let parsed = game_data.parse()?;
        process_game_data(ctx, &game_id, &parsed, &game_data.valid_from.0).await?;
    }

    Ok(())
}

pub async fn fetch_all_games(ctx: &WorkerContext) -> anyhow::Result<()> {
    let game_ids = get_all_known_game_ids(ctx).await?;
    ctx.process_many_with_progress(game_ids, 50, "fetch all games", poll_game_by_id).await;
    Ok(())
}

pub async fn fetch_all_schedules(ctx: &WorkerContext, parallel: usize) -> anyhow::Result<()> {
    let team_ids = ctx.db.get_all_entity_ids(EntityKind::Team).await?;
    ctx.process_many_with_progress(team_ids, parallel, "fetch all schedules", poll_schedule_for_team_for_all_games)
        .await;

    Ok(())
}

#[derive(Deserialize)]
pub struct TeamSchedule {
    games: Vec<TeamScheduleGame>,
}

#[derive(Deserialize)]
pub struct TeamScheduleGame {
    day: i32,
    state: String,
    game_id: Option<String>,
}

#[derive(Deserialize)]
pub struct LiveResponse {
    entries: Vec<serde_json::Value>,
}
