use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;

use crate::admin::ShutdownType;
use crate::scheduler::{schedule::Schedule, timezone, Task, TaskCtx, TaskOutcome};

/// Replaces `scripts/daily-battlegroup-restart-notice`. Fires at the configured
/// wall-clock hour:minute (default 04:30 in the configured tz). Computes the
/// target timestamp for the actual restart and publishes a single ServerShutdown
/// broadcast — the server uses the frequency/duration fields to render its own
/// repeating countdown.
pub struct RestartNoticeTask;

#[async_trait]
impl Task for RestartNoticeTask {
    fn id(&self) -> &'static str {
        "restart-notice"
    }

    fn schedule(&self) -> Schedule {
        Schedule::daily(4, 30)
    }

    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutcome> {
        let target_utc = timezone::next_daily_at(
            ctx.env.restart_tz,
            ctx.env.restart_hour,
            ctx.env.restart_minute,
            Utc::now(),
        );
        let target_ts = target_utc.timestamp();

        ctx.log_info(&format!(
            "scheduling restart warning target_ts={target_ts} frequency={}s duration={}s tz={}",
            ctx.env.restart_warning_frequency_secs,
            ctx.env.restart_warning_duration_secs,
            ctx.env.restart_tz.name(),
        ))?;

        if ctx.dry_run {
            ctx.log_info("[dry-run] would publish ServerShutdown broadcast")?;
            return Ok(TaskOutcome::Done);
        }

        let result = ctx
            .env
            .mq
            .publish_server_shutdown(
                ShutdownType::Restart,
                target_ts,
                ctx.env.restart_warning_frequency_secs,
                ctx.env.restart_warning_duration_secs,
            )
            .await?;
        ctx.log_info(&format!(
            "publish ok={} output={}",
            result.ok,
            result.output.trim()
        ))?;
        Ok(TaskOutcome::Done)
    }
}
