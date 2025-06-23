use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use chron_base::load_config;
use chron_db::ChronDb;
use http::DataClient;
use tracing::{error, info};
use uuid::Uuid;
use workers::{
    IntervalWorker, SimState, WorkerContext, crunch,
    games::{self},
    import,
    league::{self, PollLeague, PollNewPlayers},
    matviews::RefreshMatviews,
};

use crate::workers::{
    games::{PollFinishedGamesFromFeed, PollLiveGames, PollTodayGames},
    league::PollAllPlayers,
    map::LookupMapLocations,
    message::PollMessage,
};

mod http;
mod models;
mod workers;

fn spawn<T: IntervalWorker + 'static>(mut ctx: WorkerContext, mut w: T) {
    tokio::spawn(async move {
        let mut interval = T::interval();
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let type_name = std::any::type_name::<T>().split("::").last().unwrap();

        // add some jitter to prevent hammering the server on ingest startup
        if ctx.config.jitter {
            let jitter_amount = rand::random_range(0.1..=1.0);
            let sleep_duration = interval.period().mul_f64(jitter_amount);
            info!("{}: sleeping for {:?}", type_name, sleep_duration);
            tokio::time::sleep(sleep_duration).await;
            interval.reset_immediately();
        }

        loop {
            interval.tick().await;

            info!("running: {}", type_name);
            w.tick(&mut ctx).await.unwrap_or_else(|e| {
                let type_name = std::any::type_name::<T>().split("::").last().unwrap();
                error!("error executing worker {}: {:?}", type_name, e);
            });
            info!("done: {}", type_name);
        }
    });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(load_config()?);

    let args = std::env::args().collect::<Vec<_>>();
    // todo: this is stupid
    let db = if args.len() > 1 && args[1] == "migrate" {
        ChronDb::new_from_scratch(&config).await?
    } else {
        ChronDb::new(&config).await?
    };
    let client = DataClient::new()?;
    let ctx = WorkerContext {
        client,
        db,
        config: config,
        _sim: Arc::new(RwLock::new(SimState {
            _season: Uuid::default(),
            _day: -1,
        })),
    };

    if args.len() > 1 {
        if let Err(e) = handle_fn(&ctx, &args[1], &args[2..]).await {
            error!("error running cli: {:?}", e);
        }
        Ok(())
    } else {
        spawn(ctx.clone(), PollLeague);
        spawn(ctx.clone(), PollNewPlayers);
        spawn(ctx.clone(), RefreshMatviews);
        spawn(ctx.clone(), PollMessage);
        spawn(ctx.clone(), PollTodayGames);
        spawn(ctx.clone(), PollLiveGames);
        spawn(ctx.clone(), PollAllPlayers);
        spawn(ctx.clone(), PollFinishedGamesFromFeed);
        spawn(ctx.clone(), LookupMapLocations);

        // retiring this one for now, server's slow
        // spawn(ctx.clone(), PollAllScheduledGames);

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

async fn handle_fn(ctx: &WorkerContext, name: &str, args: &[String]) -> anyhow::Result<()> {
    match name {
        "rebuild-games" => games::rebuild_games(ctx).await?,
        "rebuild-games-slow" => games::rebuild_games_slow(ctx).await?,
        "fetch-league" => league::poll_league(ctx).await?,
        "fetch-all-games" => games::fetch_all_games(ctx).await?,
        "fetch-all-schedules" => games::fetch_all_schedules(ctx, 10).await?,
        "fetch-all-players" => league::fetch_all_players(ctx).await?,
        "rebuild-team-lite" => league::rebuild_team_lite(ctx).await?,
        "rebuild-player-lite" => league::rebuild_player_lite(ctx).await?,
        "import-db" => import::import(ctx, &args[0]).await?,
        "crunch" => crunch::crunch(ctx).await?,
        "migrate" => ctx.db.migrate().await?,
        _ => panic!("unknown function: {}", name),
    }

    Ok(())
}
