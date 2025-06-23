use std::sync::{Arc, RwLock};
use std::time::Duration;

use chron_base::ChronConfig;
use chron_db::models::IsoDateTime;
use chron_db::{ChronDb, models::EntityKind};
use futures::StreamExt;
use futures::stream;
use reqwest::IntoUrl;
use time::{OffsetDateTime, Time};
use tokio::time::Interval;
use tracing::{error, info};
use uuid::Uuid;

use crate::http::{ClientResponse, DataClient};
use crate::models::MmolbTime;

pub mod crunch;
pub mod games;
pub mod import;
pub mod league;
pub mod map;
pub mod matviews;
pub mod message;

#[derive(Clone)]
pub struct WorkerContext {
    pub _sim: Arc<RwLock<SimState>>,
    pub config: Arc<ChronConfig>,
    pub db: ChronDb,
    pub client: DataClient,
}

impl WorkerContext {
    // pub fn season_day(&self) -> (Uuid, i32) {
    //     let s = self.sim.read().expect("should never be poisoned");
    //     (s._season.clone(), s._day)
    // }

    // pub fn update_state(&self, new_state: SimState) {
    //     let mut s = self.sim.write().expect("should never be poisoned");
    //     *s = new_state;
    // }

    pub async fn try_update_time(&self) -> anyhow::Result<MmolbTime> {
        let latest_time = self
            .db
            .get_latest_observation(EntityKind::Time, "time")
            .await?;
        if let Some(latest) = latest_time {
            let buffer = Duration::from_secs(30);
            if latest.timestamp.0 + buffer > OffsetDateTime::now_utc() {
                // cache is valid
                return latest.parse();
            }
        }

        let res = self
            .fetch_and_save("https://mmolb.com/api/time", EntityKind::Time, "time")
            .await?;
        res.parse()
    }

    pub async fn fetch_and_save(
        &self,
        url: impl IntoUrl,
        kind: EntityKind,
        entity_id: impl Into<String>,
    ) -> anyhow::Result<ClientResponse> {
        let resp = self.client.fetch(url).await?;
        self.db
            .save(resp.to_chron(kind, &entity_id.into())?)
            .await?;
        Ok(resp)
    }

    pub async fn process_many<'a, T, F, Fut>(
        &'a self,
        values: impl IntoIterator<Item = T>,
        parallel: usize,
        function: F,
    ) where
        F: Fn(&'a WorkerContext, T) -> Fut,
        Fut: Future<Output = anyhow::Result<()>>,
    {
        stream::iter(values)
            .map(|x| function(&self, x))
            .buffer_unordered(parallel)
            .for_each(|res| async {
                if let Err(e) = res {
                    error!("error processing item: {:?}", e);
                }
            })
            .await
    }

    // will buffer `values`
    pub async fn process_many_with_progress<'a, T, F, Fut>(
        &'a self,
        values: impl IntoIterator<Item = T>,
        parallel: usize,
        name: &'static str,
        function: F,
    ) where
        F: Fn(&'a WorkerContext, T) -> Fut,
        Fut: Future<Output = anyhow::Result<()>>,
    {
        let values = values.into_iter().collect::<Vec<_>>();
        let count = values.len();
        let progress_interval = if count > 1000 { 10 } else { 1 };
        stream::iter(values)
            .map(|x| function(&self, x))
            .buffer_unordered(parallel)
            .enumerate()
            .for_each(|(i, res)| async move {
                if let Err(e) = res {
                    error!("error processing item: {:?}", e);
                } else if i % progress_interval == 0 {
                    info!("processed {} ({}/{})", name, i, count);
                }
            })
            .await
    }
}

pub trait IntervalWorker: Send + Sync {
    fn interval() -> Interval;

    fn tick(
        &mut self,
        ctx: &mut WorkerContext,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

pub struct SimState {
    pub _season: Uuid,
    pub _day: i32,
}
