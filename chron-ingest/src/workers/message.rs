use std::time::Duration;

use chron_db::models::EntityKind;

use crate::workers::IntervalWorker;

pub struct PollMessage;

impl IntervalWorker for PollMessage {
    fn interval() -> tokio::time::Interval {
        tokio::time::interval(Duration::from_secs(5))
    }

    async fn tick(&mut self, ctx: &mut super::WorkerContext) -> anyhow::Result<()> {
        ctx.fetch_and_save("https://mmolb.com/api/message", EntityKind::Message, "")
            .await?;

        Ok(())
    }
}
