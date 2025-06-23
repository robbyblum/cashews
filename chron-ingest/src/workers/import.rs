use std::{
    collections::{BTreeMap, BTreeSet},
    io::Cursor,
};

use chron_db::models::EntityKind;
use futures::{StreamExt, TryStreamExt, stream};
use sqlx::{SqlitePool, prelude::FromRow};
use time::OffsetDateTime;
use tracing::{error, info};
use uuid::Uuid;

use super::WorkerContext;

#[derive(FromRow)]
struct Observation {
    #[sqlx(rename = "type")]
    kind: String,
    id: String,
    timestamp: i64,
    hash: String,
}

#[derive(FromRow)]
struct Object {
    hash: String,
    data: Vec<u8>,
}

pub async fn import(ctx: &WorkerContext, filename: &str) -> anyhow::Result<()> {
    let sqlite = SqlitePool::connect(filename).await?;

    info!("getting objects...");
    let objects: Vec<Object> = sqlx::query_as("select hash, data from objects")
        .fetch_all(&sqlite)
        .await?;
    info!("loaded {} objects", objects.len());
    let objects_map = BTreeMap::from_iter(objects.into_iter().map(|row| (row.hash, row.data)));

    let existing_objects: Vec<Uuid> = sqlx::query_scalar("select hash from objects")
        .fetch_all(&ctx.db.pool)
        .await?;
    let existing_objects = BTreeSet::from_iter(existing_objects);
    for ele in existing_objects {
        ctx.db.saved_objects.insert(ele);
    }

    let hash_map = stream::iter(objects_map)
        .map(|(old_hash, value)| insert_object(ctx, old_hash, value))
        .buffer_unordered(100)
        .enumerate()
        .map(|(i, x)| {
            if i % 1000 == 0 {
                info!("{} in", i);
            }
            x
        })
        .try_collect::<BTreeMap<_, _>>()
        .await?;

    info!("inserted objects");

    info!("getting observations...");
    let observations =
        sqlx::query_as::<_, Observation>("select type, id, timestamp, hash from observations")
            .fetch(&sqlite);
    observations
        .map(|obs| do_import(ctx, obs, &hash_map))
        .buffer_unordered(1000)
        .chunks(10000)
        .enumerate()
        .for_each_concurrent(10, |(i, x)| async move {
            if let Err(e) = submit(ctx, x).await {
                error!("aaaa: {:?}", e);
            } else {
                info!("{} in", i);
            }
        })
        .await;

    info!("deduplicating");
    sqlx::query("create table new_observations as select distinct c.* from observations c;")
        .execute(&ctx.db.pool)
        .await?;
    sqlx::query("truncate table observations;")
        .execute(&ctx.db.pool)
        .await?;
    sqlx::query("insert into observations select * from new_observations;")
        .execute(&ctx.db.pool)
        .await?;
    sqlx::query("drop table new_observations;")
        .execute(&ctx.db.pool)
        .await?;

    info!("rebuilding");
    for kind in [
        EntityKind::Time,
        EntityKind::State,
        EntityKind::League,
        EntityKind::Team,
        EntityKind::Game,
        EntityKind::News,
        EntityKind::Spotlight,
        EntityKind::Player,
        EntityKind::Nouns,
        EntityKind::Adjectives,
        EntityKind::Election,
        EntityKind::GamesEndpoint,
        EntityKind::PostseasonBracket,
    ] {
        info!("rebuilding: {:?}", kind);

        let ids = ctx.db.get_all_entity_ids_slow(kind).await?;
        stream::iter(ids)
            .map(|x| ctx.db.rebuild(kind, x))
            .buffer_unordered(100)
            .for_each(|x| async {
                if let Err(e) = x {
                    error!("aaaa: {:?}", e);
                }
            })
            .await;
    }

    info!("done");

    Ok(())
}

async fn insert_object(
    ctx: &WorkerContext,
    old_hash: String,
    value: Vec<u8>,
) -> anyhow::Result<(String, Uuid)> {
    let value = tokio::task::spawn_blocking(move || decode(&value)).await??;
    let our_hash = ctx.db.save_object(value).await?;
    Ok((old_hash, our_hash))
}

async fn do_import(
    _ctx: &WorkerContext,
    obs: sqlx::Result<Observation>,
    hash_map: &BTreeMap<String, Uuid>,
) -> anyhow::Result<(EntityKind, String, OffsetDateTime, f64, Uuid)> {
    let obs = obs?;

    let our_hash = hash_map.get(&obs.hash).unwrap().clone();

    let kind = match obs.kind.as_str() {
        "time" => EntityKind::Time,
        "state" => EntityKind::State,
        "league" => EntityKind::League,
        "team" => EntityKind::Team,
        "game-by-team" => return Err(anyhow::anyhow!("nope")),
        "game" => EntityKind::Game,
        "news" => EntityKind::News,
        "spotlight" => EntityKind::Spotlight,
        "player" => EntityKind::Player,
        "nouns" => EntityKind::Nouns,
        "adjectives" => EntityKind::Adjectives,
        "election" => EntityKind::Election,
        "games-endpoint" => EntityKind::GamesEndpoint,
        "postseason-bracket" => EntityKind::PostseasonBracket,
        _ => panic!("idk: {}", obs.kind),
    };

    let timestamp = OffsetDateTime::from_unix_timestamp_nanos((obs.timestamp as i128) * 1_000_000)?;
    Ok((kind, obs.id, timestamp, 0.0, our_hash))
}

fn decode(data: &[u8]) -> anyhow::Result<serde_json::Value> {
    let decoded = zstd::decode_all(Cursor::new(data))?;
    let value = serde_json::from_slice::<serde_json::Value>(&decoded)?;
    Ok(value)
}

async fn submit(
    ctx: &WorkerContext,
    foo: Vec<anyhow::Result<(EntityKind, std::string::String, OffsetDateTime, f64, Uuid)>>,
) -> anyhow::Result<()> {
    ctx.db
        .insert_observations_raw_bulk(&foo.into_iter().filter_map(|x| x.ok()).collect::<Vec<_>>())
        .await?;
    Ok(())
}
