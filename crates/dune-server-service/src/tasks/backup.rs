use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;

use crate::kubectl::battlegroup as bg;
use crate::scheduler::{Schedule, Task, TaskCtx, TaskOutcome};

/// Replaces `scripts/cron-battlegroup-backup`. Runs the vendor backup helper,
/// emits a per-run log line referencing the dump path, and lets the operator
/// handle stale dump cleanup out-of-band (we do not invoke `sudo find -delete`
/// from the daemon — too easy to widen the blast radius).
pub struct BackupTask;

#[async_trait]
impl Task for BackupTask {
    fn id(&self) -> &'static str {
        "backup"
    }

    fn schedule(&self) -> Schedule {
        Schedule::interval_secs(2 * 60 * 60)
    }

    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutcome> {
        let cluster = ctx.env.cluster.get().await?;
        let bg_name = bg::bg_name(&ctx.env.kubectl, &cluster.namespace).await?;
        let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_name = format!("{}-{}.backup", bg_name, stamp);

        if ctx.dry_run {
            ctx.log_info(&format!(
                "[dry-run] would invoke battlegroup backup name={backup_name}"
            ))?;
            return Ok(TaskOutcome::Done);
        }

        ctx.log_info(&format!(
            "starting backup bg={bg_name} ns={} name={backup_name}",
            cluster.namespace
        ))?;
        ctx.env.bg_cli.backup(&backup_name).await?;
        ctx.log_info(&format!(
            "backup complete path=/funcom/artifacts/database-dumps/{bg_name}/{backup_name}"
        ))?;
        Ok(TaskOutcome::Done)
    }
}
