use std::{str::FromStr, sync::Arc};

use anyhow::anyhow;
use chron_base::ChronConfig;
use dashmap::DashSet;
use futures::{StreamExt, stream};
use models::{EntityKind, NewObject};
use sea_query::Iden;
use siphasher::sip128::{Hasher128, SipHasher};
use sqlx::{
    Executor, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use time::{Duration, OffsetDateTime};
use tracing::{error, info};
use util::HashingWriter;
use uuid::Uuid;

pub mod derived;
pub mod models;
pub mod queries;
pub mod util;

#[derive(Iden)]
pub enum Idens {
    AwayTeamId,
    Data,
    Day,
    EntityId,
    Event,
    Events,
    GameId,
    GamePlayerStats,
    Games,
    Hash,
    HomeTeamId,
    Kind,
    Objects,
    Payload,
    PlayerId,
    Raw,
    Season,
    TeamId,
    Timestamp,
    ValidFrom,
    ValidTo,
    Versions,
}

#[derive(Clone)]
pub struct ChronDb {
    pub pool: PgPool,
    pub saved_objects: Arc<DashSet<Uuid>>,
}

impl ChronDb {
    pub async fn new_from_scratch(config: &ChronConfig) -> anyhow::Result<ChronDb> {
        let pool_opts = PgPoolOptions::new().max_connections(50);
        let conn_opts = PgConnectOptions::from_str(&config.database_uri)?;
        let pool = pool_opts.connect_with(conn_opts).await?;

        Ok(ChronDb {
            pool,
            saved_objects: Arc::new(DashSet::new()),
        })
    }

    pub async fn new(config: &ChronConfig) -> anyhow::Result<ChronDb> {
        let pool_opts = PgPoolOptions::new().max_connections(50);
        let conn_opts = PgConnectOptions::from_str(&config.database_uri)?;
        let pool = pool_opts.connect_with(conn_opts).await?;

        if let Err(_) = pool.execute("select * from _sqlx_migrations").await {
            return Err(anyhow::anyhow!(
                "database not initialized, run `chron-ingest migrate`?"
            ))?;
        }

        Ok(ChronDb {
            pool,
            saved_objects: Arc::new(DashSet::new()),
        })
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        info!("migrating...");
        sqlx::migrate!("./migrations").run(&self.pool).await?;

        info!("updating functions...");

        let mut tx = self.pool.acquire().await?;
        tx.execute(include_str!("../migrations/functions.sql"))
            .await?;

        info!("updating views...");
        tx.execute(include_str!("../migrations/views.sql")).await?;

        info!("done!");
        Ok(())
    }

    pub async fn rebuild(&self, kind: EntityKind, entity_id: String) -> anyhow::Result<()> {
        sqlx::query("select rebuild_entity($1::smallint, $2::text)")
            .bind(kind)
            .bind(entity_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn rebuild_all(&self, kind: EntityKind) -> anyhow::Result<()> {
        // sqlx::query("select rebuild_entity($1::smallint, id) from latest_versions where kind = $1")
        //     .bind(kind)
        //     .execute(&self.pool)
        //     .await?;

        let ids = self.get_all_entity_ids_slow(kind).await?;
        stream::iter(ids)
            .map(|x| self.rebuild(kind, x))
            .buffer_unordered(10)
            .for_each(|x| async {
                if let Err(e) = x {
                    error!("error rebuilding entity: {:?}", e);
                }
            })
            .await;

        Ok(())
    }

    pub async fn save(&self, obj: NewObject) -> anyhow::Result<()> {
        let hash = self.save_object(obj.data).await?;
        self.add_version(
            obj.kind,
            &obj.entity_id,
            hash,
            obj.timestamp,
            obj.request_time,
        )
        .await?;

        Ok(())
    }

    pub async fn save_raw(&self, obj: NewObject) -> anyhow::Result<()> {
        let hash = self.save_object(obj.data).await?;
        self.insert_observation_raw(
            obj.kind,
            &obj.entity_id,
            obj.timestamp,
            obj.request_time.as_seconds_f64(),
            hash,
        )
        .await?;
        Ok(())
    }

    pub async fn insert_observation_raw(
        &self,
        kind: EntityKind,
        entity_id: &str,
        timestamp: OffsetDateTime,
        request_time: f64,
        hash: Uuid,
    ) -> anyhow::Result<()> {
        sqlx::query("insert into observations (kind, entity_id, timestamp, request_time, hash) values ($1, $2, $3, $4, $5)")
            .bind(kind)
            .bind(entity_id)
            .bind(timestamp)
            .bind(request_time)
            .bind(hash)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn insert_observations_raw_bulk(
        &self,
        observations: &[(EntityKind, std::string::String, OffsetDateTime, f64, Uuid)],
    ) -> anyhow::Result<()> {
        let kinds = observations.iter().map(|x| x.0).collect::<Vec<_>>();
        let ids = observations.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let timestamps = observations.iter().map(|x| x.2).collect::<Vec<_>>();
        let times = observations.iter().map(|x| x.3).collect::<Vec<_>>();
        let hashes = observations.iter().map(|x| x.4).collect::<Vec<_>>();
        sqlx::query("insert into observations (kind, entity_id, timestamp, request_time, hash) select unnest($1), unnest($2), unnest($3), unnest($4), unnest($5)")
            .bind(kinds)
            .bind(ids)
            .bind(timestamps)
            .bind(times)
            .bind(hashes)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn save_object(&self, data: serde_json::Value) -> anyhow::Result<Uuid> {
        let (hash, data) = tokio::task::spawn_blocking(|| json_hash(data)).await??;

        // ok if we save double here
        if !self.saved_objects.contains(&hash) {
            sqlx::query("insert into objects (hash, data) values ($1, $2) on conflict do nothing")
                .bind(hash)
                .bind(data)
                .execute(&self.pool)
                .await?;
            self.saved_objects.insert(hash);
        }

        Ok(hash)
    }

    async fn add_version(
        &self,
        kind: EntityKind,
        entity_id: &str,
        hash: Uuid,
        timestamp: OffsetDateTime,
        request_time: Duration,
    ) -> anyhow::Result<()> {
        sqlx::query("select add_version($1, $2, $3, $4, $5)")
            .bind(kind)
            .bind(entity_id)
            .bind(hash)
            .bind(timestamp)
            .bind(request_time.as_seconds_f32())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

fn json_hash(mut value: serde_json::Value) -> anyhow::Result<(Uuid, serde_json::Value)> {
    value.sort_all_objects();

    let mut hasher = SipHasher::new();

    let hw = HashingWriter::new(&mut hasher);
    serde_json::to_writer(hw, &value).map_err(|_| anyhow!("error serializing json"))?;

    let hash = Uuid::from_u128(hasher.finish128().as_u128());
    Ok((hash, value))
}
