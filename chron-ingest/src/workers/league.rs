use std::{collections::HashSet, time::Duration};

use chron_db::{
    derived::{DbLeagueSaveModel, DbTeamSaveModel},
    models::{EntityKind, NewObject},
};
use futures::TryStreamExt;
use tracing::info;

use crate::{
    http::ClientResponse,
    models::{MmolbLeague, MmolbState, MmolbTeam},
};

use super::{IntervalWorker, WorkerContext};

pub struct PollLeague;
pub struct PollNewPlayers;
pub struct PollAllPlayers;

impl IntervalWorker for PollLeague {
    fn interval() -> tokio::time::Interval {
        tokio::time::interval(Duration::from_secs(10 * 60))
    }

    async fn tick(&mut self, ctx: &mut WorkerContext) -> anyhow::Result<()> {
        poll_league(ctx).await?;
        Ok(())
    }
}

pub async fn poll_league(ctx: &WorkerContext) -> anyhow::Result<()> {
    let state_resp = ctx
        .fetch_and_save("https://mmolb.com/api/state", EntityKind::State, "state")
        .await?;

    ctx.fetch_and_save(
        "https://mmolb.com/api/spotlight",
        EntityKind::Spotlight,
        "spotlight",
    )
    .await?;

    let _time = ctx.try_update_time().await?;

    let state: MmolbState = state_resp.parse()?;

    let league_ids = get_league_ids(&state);
    info!("got {} league ids", league_ids.len());
    ctx.process_many_with_progress(league_ids, 3, "fetch leagues", fetch_league)
        .await;

    let team_ids = get_all_known_team_ids(ctx).await?;
    info!("got {} team ids", team_ids.len());
    ctx.process_many_with_progress(team_ids, 3, "fetch teams", fetch_team).await;
    Ok(())
}

impl IntervalWorker for PollNewPlayers {
    fn interval() -> tokio::time::Interval {
        tokio::time::interval(Duration::from_secs(60))
    }

    async fn tick(&mut self, ctx: &mut WorkerContext) -> anyhow::Result<()> {
        let player_ids = get_all_known_player_ids(ctx).await?;

        let player_object_ids = ctx.db.get_all_entity_ids(EntityKind::Player).await?;
        let player_object_ids = HashSet::from_iter(player_object_ids);

        let new_players = player_ids.difference(&player_object_ids).cloned();
        ctx.process_many_with_progress(new_players, 3, "fetch players", fetch_player).await;

        Ok(())
    }
}

impl IntervalWorker for PollAllPlayers {
    fn interval() -> tokio::time::Interval {
        tokio::time::interval(Duration::from_secs(60 * 30))
    }

    async fn tick(&mut self, ctx: &mut WorkerContext) -> anyhow::Result<()> {
        let player_ids = get_all_known_player_ids(ctx).await?;
        info!("got {} player ids", player_ids.len());

        // this one can go slowly, that's fine
        ctx.process_many_with_progress(player_ids, 5, "fetch all players", fetch_player).await;

        Ok(())
    }
}

async fn fetch_league(ctx: &WorkerContext, id: String) -> anyhow::Result<()> {
    let url = format!("https://mmolb.com/api/league/{}", id);
    let resp = ctx.fetch_and_save(url, EntityKind::League, &id).await?;

    let league_data = resp.parse::<MmolbLeague>()?;
    ctx.db
        .update_league(DbLeagueSaveModel {
            league_id: &id,
            league_type: &league_data.league_type,
            name: &league_data.name,
            color: &league_data.color,
            emoji: &league_data.emoji,
        })
        .await?;

    Ok(())
}

async fn fetch_team(ctx: &WorkerContext, id: String) -> anyhow::Result<()> {
    let url = format!("https://mmolb.com/api/team/{}", id);
    let resp = ctx.fetch_and_save(url, EntityKind::Team, &id).await?;

    write_team_lite(ctx, &id, &resp).await?;

    let team_data = resp.parse::<MmolbTeam>()?;
    ctx.db
        .update_team(DbTeamSaveModel {
            team_id: &id,
            league_id: &team_data.league,
            location: &team_data.location,
            name: &team_data.name,
            full_location: &team_data.full_location,
            emoji: &team_data.emoji,
            color: &team_data.color,
            abbreviation: &team_data.abbreviation,
        })
        .await?;

    Ok(())
}

async fn fetch_player(ctx: &WorkerContext, id: String) -> anyhow::Result<()> {
    let url = format!("https://mmolb.com/api/player/{}", id);
    let resp = ctx.fetch_and_save(url, EntityKind::Player, &id).await?;

    write_player_lite(ctx, &id, &resp).await?;
    Ok(())
}

fn get_league_ids(state: &MmolbState) -> Vec<String> {
    let mut out = Vec::new();
    out.extend(state.greater_leagues.iter().cloned());
    out.extend(state.lesser_leagues.iter().cloned());
    out
}

async fn get_all_known_team_ids(ctx: &WorkerContext) -> anyhow::Result<HashSet<String>> {
    let mut team_ids = HashSet::new();

    // get from DB leagues
    for league_obj in ctx.db.get_all_latest(EntityKind::League).await? {
        let league = league_obj.parse::<MmolbLeague>()?;
        team_ids.extend(league.teams);
        team_ids.extend(league.superstar_team);
    }

    // get from DB teams
    team_ids.extend(ctx.db.get_all_entity_ids(EntityKind::Team).await?);

    // get from stats obj?
    team_ids.extend(ctx.db.get_all_team_ids_from_stats().await?);

    Ok(team_ids)
}

async fn get_all_known_player_ids(ctx: &WorkerContext) -> anyhow::Result<HashSet<String>> {
    let mut team_ids = HashSet::new();

    // get from DB teams
    let get_all_latest =  ctx.db.get_all_latest(EntityKind::Team).await?;
    for team_obj in get_all_latest {
        let team = team_obj.parse::<MmolbTeam>()?;

        for player_slot in team.players {
            if player_slot.player_id != "#" {
                team_ids.insert(player_slot.player_id);
            }
        }
    }

    // get from DB players
    team_ids.extend(ctx.db.get_all_entity_ids(EntityKind::Player).await?);

    // get from stats obj?
    team_ids.extend(ctx.db.get_all_player_ids_from_stats().await?);

    Ok(team_ids)
}

pub async fn fetch_all_players(ctx: &WorkerContext) -> anyhow::Result<()> {
    let all_players = get_all_known_player_ids(ctx).await?;
    ctx.process_many_with_progress(all_players, 50, "fetch all players", fetch_player).await;
    Ok(())
}

async fn write_team_lite(
    ctx: &WorkerContext,
    id: &str,
    resp: &ClientResponse,
) -> anyhow::Result<()> {
    // write a "cleaned" version of the team object without the big Players[x].Stats objects
    let mut team_data = resp.parse::<serde_json::Value>()?;
    to_team_lite(&mut team_data);

    ctx.db
        .save(NewObject {
            kind: EntityKind::TeamLite,
            entity_id: id.to_string(),
            data: team_data,
            timestamp: resp.timestamp(),
            request_time: resp.request_time(),
        })
        .await?;
    Ok(())
}

async fn write_player_lite(
    ctx: &WorkerContext,
    id: &str,
    resp: &ClientResponse,
) -> anyhow::Result<()> {
    // write a "cleaned" version of the player object without the big stats object
    let mut player_data = resp.parse::<serde_json::Value>()?;
    to_player_lite(&mut player_data);
    ctx.db
        .save(NewObject {
            kind: EntityKind::PlayerLite,
            entity_id: id.to_string(),
            data: player_data,
            timestamp: resp.timestamp(),
            request_time: resp.request_time(),
        })
        .await?;

    Ok(())
}

fn to_team_lite(data: &mut serde_json::Value) {
    // can we make this code nicer?
    if let Some(o) = data.as_object_mut() {
        if let Some(players_value) = o.get_mut("Players") {
            if let Some(players) = players_value.as_array_mut() {
                for player_value in players {
                    if let Some(p) = player_value.as_object_mut() {
                        p.remove("Stats");
                    }
                }
            }
        }
        o.remove("Feed");
    }
}

fn to_player_lite(data: &mut serde_json::Value) {
    if let Some(o) = data.as_object_mut() {
        o.remove("Stats");
        o.remove("Feed");
    }
}

// todo: deduplicate these two?
pub async fn rebuild_team_lite(ctx: &WorkerContext) -> anyhow::Result<()> {
    sqlx::query("delete from observations where kind = $1")
        .bind(EntityKind::TeamLite)
        .execute(&ctx.db.pool)
        .await?;

    let mut stream = ctx
        .db
        .get_all_versions_stream(EntityKind::TeamLite)
        .await?
        .try_chunks(1000);

    let mut i = 0;
    while let Some(vers) = stream.try_next().await? {
        let mut chunk = Vec::with_capacity(1000);

        for ver in vers {
            let mut data = ver.parse()?;
            to_team_lite(&mut data);
            let hash = ctx.db.save_object(data).await?;
            chunk.push((
                EntityKind::TeamLite,
                ver.entity_id,
                ver.valid_from.0,
                0.0f64,
                hash,
            ));
        }

        ctx.db.insert_observations_raw_bulk(&chunk).await?;

        i += chunk.len();
        tracing::info!("rebuilt {} lite observations", i);
    }

    tracing::info!("rebuilding versions table for teamlite");
    ctx.db.rebuild_all(EntityKind::TeamLite).await?;

    tracing::info!("done!");
    Ok(())
}

pub async fn rebuild_player_lite(ctx: &WorkerContext) -> anyhow::Result<()> {
    sqlx::query("delete from observations where kind = $1")
        .bind(EntityKind::PlayerLite)
        .execute(&ctx.db.pool)
        .await?;

    let mut stream = ctx
        .db
        .get_all_versions_stream(EntityKind::PlayerLite)
        .await?
        .try_chunks(1000);

    let mut i = 0;
    while let Some(vers) = stream.try_next().await? {
        let mut chunk = Vec::with_capacity(1000);

        for ver in vers {
            let mut data = ver.parse()?;
            to_player_lite(&mut data);
            let hash = ctx.db.save_object(data).await?;
            chunk.push((
                EntityKind::PlayerLite,
                ver.entity_id,
                ver.valid_from.0,
                0.0f64,
                hash,
            ));
        }

        ctx.db.insert_observations_raw_bulk(&chunk).await?;

        i += chunk.len();
        tracing::info!("rebuilt {} lite observations", i);
    }

    tracing::info!("rebuilding versions table for playerlite");
    ctx.db.rebuild_all(EntityKind::PlayerLite).await?;

    tracing::info!("done!");
    Ok(())
}
