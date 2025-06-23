use std::{collections::BTreeMap, fmt::Display, str::FromStr, sync::Arc};

use axum::{
    Json,
    extract::{Query, State},
};
use chron_base::normalize_location;
use chron_db::{
    derived::{AverageStats, DbGame, DbGamePlayerStats, DbLeague, DbTeam},
    models::PageToken,
    queries::{PaginatedResult, SortOrder},
};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay, serde_as};
use sqlx::FromRow;
use tracing::info;

use crate::{AppError, AppState};

#[derive(Deserialize, Debug)]
pub struct GetGamesQuery {
    pub season: i32,
    pub day: Option<i32>,
    pub team: Option<String>,

    #[serde(default)]
    order: SortOrder,
    #[serde(default)]
    count: Option<u64>,
    page: Option<PageToken>,
}

pub async fn get_games(
    State(ctx): State<AppState>,
    Query(q): Query<GetGamesQuery>,
) -> Result<Json<PaginatedResult<DbGame>>, AppError> {
    let games = ctx
        .db
        .get_games(chron_db::derived::GetGamesQuery {
            season: Some(q.season),
            day: q.day,
            team: q.team,

            order: q.order,
            count: q.count.unwrap_or(1000),
            page: q.page,
        })
        .await?;

    Ok(Json(games))
}

#[derive(Deserialize, Debug)]
pub struct GetTeamsQuery {}

pub async fn get_teams(
    State(ctx): State<AppState>,
    Query(_q): Query<GetTeamsQuery>,
) -> Result<Json<PaginatedResult<DbTeam>>, AppError> {
    let teams = ctx.db.get_teams().await?;

    Ok(Json(fake_paginate(teams)))
}

#[derive(Deserialize, Debug)]
pub struct GetLeaguesQuery {}

pub async fn get_leagues(
    State(ctx): State<AppState>,
    Query(_q): Query<GetTeamsQuery>,
) -> Result<Json<PaginatedResult<DbLeague>>, AppError> {
    let teams = ctx.db.get_leagues().await?;

    Ok(Json(fake_paginate(teams)))
}

#[derive(Deserialize, Debug)]
pub struct GetPlayerStatsQuery {
    pub start: Option<SeasonDay>,
    pub end: Option<SeasonDay>,

    pub player: Option<String>,
    pub team: Option<String>,
}

pub async fn get_player_stats(
    State(ctx): State<AppState>,
    Query(q): Query<GetPlayerStatsQuery>,
) -> Result<Json<Vec<ApiPlayerStats>>, AppError> {
    if q.player.is_none() && q.team.is_none() {
        return Err(anyhow::anyhow!("must include either player or team id").into());
    }

    let stats = ctx
        .db
        .get_player_stats(chron_db::derived::GetPlayerStatsQuery {
            start: q.start.map(Into::into),
            end: q.end.map(Into::into),
            player: q.player,
            team: q.team,
        })
        .await?;

    Ok(Json(aggregate_player_stats(&stats)))
}

#[derive(Deserialize, Debug)]
pub struct LeagueAggregateQuery {
    pub season: i32,
}

pub async fn league_aggregate(
    State(ctx): State<AppState>,
    Query(q): Query<LeagueAggregateQuery>,
) -> Result<Json<LeagueAggregateResponse>, AppError> {
    let data = ctx.percentile_cache.get(()).await?;

    if q.season < 0 {
        return Err(AppError(anyhow::anyhow!("invalid season")));
    }

    if let Some(data) = data.get(q.season as usize) {
        Ok(Json(data.clone()))
    } else {
        Err(AppError(anyhow::anyhow!("invalid season")))
    }
}

#[derive(Deserialize)]
pub struct LeagueAveragesQuery {
    pub season: i16,
}

pub async fn league_averages(
    State(ctx): State<AppState>,
    Query(q): Query<LeagueAveragesQuery>,
) -> Result<Json<Vec<AverageStats>>, AppError> {
    if q.season < 0 {
        return Err(AppError(anyhow::anyhow!("invalid season")));
    }

    let data = ctx.db.get_league_averages(q.season).await?;

    Ok(Json(data))
}

#[derive(Serialize)]
pub struct TeamLocation {
    team: DbTeam,
    location: Option<MapsLocation>,
}

#[derive(Serialize, Clone)]
pub struct MapsLocation {
    lat: f64,
    long: f64,
}

pub async fn locations(State(ctx): State<AppState>) -> Result<Json<Vec<TeamLocation>>, AppError> {
    Ok(Json(locations_inner(ctx).await?))
}

#[derive(Deserialize)]
struct MapsLocationRaw {
    location: MapsLocationLatLong,
    #[serde(rename = "formattedAddress")]
    formatted_address: String,
    // #[serde(rename = "shortFormattedAddress")]
    // short_formatted_address: String,
}

#[derive(Deserialize)]
struct MapsLocationLatLong {
    latitude: f64,
    longitude: f64,
}

pub async fn locations_inner(ctx: AppState) -> anyhow::Result<Vec<TeamLocation>> {
    let data: Vec<(String, serde_json::Value)> = sqlx::query_as("select loc, data from locations")
        .fetch_all(&ctx.db.pool)
        .await?;

    let mut locations_map = BTreeMap::new();
    for (location_str, location_json) in data {
        if let Ok(loc) = MapsLocationRaw::deserialize(&location_json) {
            locations_map.insert(
                normalize_location(&loc.formatted_address),
                MapsLocation {
                    lat: loc.location.latitude,
                    long: loc.location.longitude,
                },
            );
            locations_map.insert(
                normalize_location(&location_str),
                MapsLocation {
                    lat: loc.location.latitude,
                    long: loc.location.longitude,
                },
            );
        }
    }

    let teams = ctx.db.get_teams().await?;
    let teams_augmented = teams
        .into_iter()
        .map(|team| TeamLocation {
            location: locations_map
                .get(&normalize_location(&team.full_location))
                .cloned(),
            team: team,
        })
        .filter(|x| x.location.is_some())
        .collect();
    Ok(teams_augmented)
}

#[derive(FromRow, Serialize)]
pub struct ScorigamiEntry {
    min: i32,
    max: i32,
    count: i32,
    first: String,
}

pub async fn scorigami(State(ctx): State<AppState>) -> Result<Json<Vec<ScorigamiEntry>>, AppError> {
    let r = fetch_scorigami(&ctx).await?;
    Ok(Json(r))
}

async fn fetch_scorigami(ctx: &AppState) -> anyhow::Result<Vec<ScorigamiEntry>> {
    // inline sql here is a bit nasty but we ball
    let r = sqlx::query_as(r"with games2 as (select least((last_update->>'home_score')::int, (last_update->>'away_score')::int) as min, greatest((last_update->>'home_score')::int, (last_update->>'away_score')::int) as max, game_id from games where state = 'Complete') select min(game_id) as first, min, max, count(*)::int as count from games2 group by (min, max);")
    .fetch_all(&ctx.db.pool)
    .await?;
    Ok(r)
}

#[derive(Clone, Serialize)]
pub struct LeagueAggregateResponse {
    leagues: BTreeMap<String, LeagueAggregateLeague>,
}

#[derive(Default, Serialize, Clone)]
pub struct LeagueAggregateLeague {
    pub ba: LeagueAggregateStat,
    pub obp: LeagueAggregateStat,
    pub slg: LeagueAggregateStat,
    pub ops: LeagueAggregateStat,
    pub sb_success: LeagueAggregateStat,
    pub era: LeagueAggregateStat,
    pub whip: LeagueAggregateStat,
    pub fip_base: LeagueAggregateStat,
    pub fip_const: LeagueAggregateStat,
    pub h9: LeagueAggregateStat,
    pub k9: LeagueAggregateStat,
    pub bb9: LeagueAggregateStat,
    pub hr9: LeagueAggregateStat,
}

#[derive(Default, Serialize, Clone)]
pub struct LeagueAggregateStat {
    percentiles: Vec<(f32, f32)>,
}

pub async fn refresh_league_aggregate(
    ctx: AppState,
) -> anyhow::Result<Vec<LeagueAggregateResponse>> {
    info!("refreshing league aggregates");
    let mut percentiles = Vec::with_capacity(101);
    for i in 0..=100 {
        percentiles.push((i as f32) / 100.0);
    }

    // todo: don't hardcode season
    let mut seasons = Vec::new();
    for season in [0, 1] {
        let res = ctx.db.get_league_percentiles(&percentiles, season).await?;

        // we should really just "transpose" this logic all the way through...
        let mut leagues = BTreeMap::new();
        for entry in res {
            let league: &mut LeagueAggregateLeague = leagues.entry(entry.league_id).or_default();

            for (stat, val) in [
                (&mut league.ba, entry.ba),
                (&mut league.obp, entry.obp),
                (&mut league.slg, entry.slg),
                (&mut league.ops, entry.ops),
                (&mut league.sb_success, entry.sb_success),
                (&mut league.era, entry.era),
                (&mut league.whip, entry.whip),
                (&mut league.fip_base, entry.fip_base),
                (&mut league.fip_const, entry.fip_const),
                (&mut league.h9, entry.h9),
                (&mut league.k9, entry.k9),
                (&mut league.bb9, entry.bb9),
                (&mut league.hr9, entry.hr9),
            ] {
                stat.percentiles.push((entry.percentile, val));
            }
        }

        seasons.push(LeagueAggregateResponse { leagues });
    }

    Ok(seasons)
}

#[derive(Serialize, Debug)]
pub struct ApiPlayerStats {
    player_id: String,
    team_id: String,
    stats: BTreeMap<String, i32>,
}

fn aggregate_player_stats(source: &[DbGamePlayerStats]) -> Vec<ApiPlayerStats> {
    // team -> player -> stat -> i32
    let mut keys = BTreeMap::<String, BTreeMap<String, BTreeMap<String, i32>>>::new();

    for row in source {
        let team_map = if let Some(team_map) = keys.get_mut(&row.team_id) {
            team_map
        } else {
            keys.entry(row.team_id.clone()).or_default()
        };

        let player_map = if let Some(player_map) = team_map.get_mut(&row.player_id) {
            player_map
        } else {
            team_map.entry(row.player_id.clone()).or_default()
        };

        if let Some(data) = row.data.as_object() {
            for (key, value) in data {
                let entry = if let Some(entry) = player_map.get_mut(key.as_str()) {
                    entry
                } else {
                    player_map.entry(key.clone()).or_insert(0)
                };

                if let Some(val) = value.as_i64() {
                    *entry += val as i32;
                }
            }
        }
    }

    let mut output = Vec::new();
    for (team_id, team_map) in keys {
        for (player_id, player_map) in team_map {
            output.push(ApiPlayerStats {
                player_id,
                team_id: team_id.clone(),
                stats: player_map,
            })
        }
    }

    output
}

fn fake_paginate<T>(data: Vec<T>) -> PaginatedResult<T> {
    PaginatedResult {
        items: data,
        next_page: None,
    }
}

// todo: move into chron-base?
#[derive(SerializeDisplay, DeserializeFromStr, Debug, Clone, Copy)]
pub struct SeasonDay {
    season: i32,
    day: i32,
}

impl Into<(i32, i32)> for SeasonDay {
    fn into(self) -> (i32, i32) {
        (self.season, self.day)
    }
}

impl FromStr for SeasonDay {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((left_str, right_str)) = s.split_once(",") else {
            return Err(anyhow::anyhow!("season/day must contain a ,"));
        };

        let left = i32::from_str(left_str)?;
        let right = i32::from_str(right_str)?;
        return Ok(SeasonDay {
            season: left,
            day: right,
        });
    }
}

impl Display for SeasonDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.season, self.day)
    }
}
