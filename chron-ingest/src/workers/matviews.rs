use std::time::{Duration, Instant};

use tracing::info;

use super::IntervalWorker;

pub struct RefreshMatviews;

impl IntervalWorker for RefreshMatviews {
    fn interval() -> tokio::time::Interval {
        tokio::time::interval(Duration::from_secs(60))
    }

    async fn tick(&mut self, ctx: &mut super::WorkerContext) -> anyhow::Result<()> {
        let matviews = [
            "game_player_stats_league_aggregate",
            "game_player_stats_global_aggregate",
            "pitches",
        ];

        for matview in matviews {
            info!("refreshing matview {}...", matview);

            let time_before = Instant::now();
            sqlx::query(&format!(
                "refresh materialized view concurrently {}",
                matview
            ))
            .execute(&ctx.db.pool)
            .await?;
            let time_after = Instant::now();

            let delta = time_after.duration_since(time_before);
            info!(
                "refreshed matview {} (took {}s)",
                matview,
                delta.as_secs_f32()
            );
        }

        Ok(())
    }
}
