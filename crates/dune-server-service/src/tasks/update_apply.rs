use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;

use crate::kubectl::battlegroup_cli;
use crate::scheduler::{Schedule, Task, TaskCtx, TaskOutcome};

/// Replaces `scripts/apply-pending-battlegroup-update`. Every minute, checks
/// whether the `pending_update` row's `due_ts` has arrived; when it has,
/// broadcasts, runs a pre-update backup, stops the BattleGroup, applies the
/// downloaded payload, restarts, and waits for readiness.
pub struct UpdateApplyTask;

#[async_trait]
impl Task for UpdateApplyTask {
    fn id(&self) -> &'static str {
        "update-apply"
    }

    fn schedule(&self) -> Schedule {
        Schedule::interval_secs(60)
    }

    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutcome> {
        let Some(pending) = ctx.store.load_pending_update()? else {
            return Ok(TaskOutcome::Noop);
        };
        if Utc::now().timestamp() < pending.due_ts {
            return Ok(TaskOutcome::Noop);
        }

        ctx.log_info(&format!(
            "applying pending update bg={} downloaded={} live={}",
            pending.battlegroup,
            pending.downloaded_version,
            pending.live_version.as_deref().unwrap_or("unknown")
        ))?;

        if ctx.dry_run {
            ctx.log_info("[dry-run] would stop / update-from-downloads / start")?;
            return Ok(TaskOutcome::Done);
        }

        if let Err(err) = ctx
            .env
            .mq
            .publish_service_broadcast(
                "Server update",
                "Server update is starting now. The server will restart.",
                60,
            )
            .await
        {
            ctx.log_warn(&format!("pre-update broadcast failed: {err:#}"))?;
        }

        ctx.log_info("taking pre-update database backup")?;
        if let Err(err) = backup_one(ctx, &pending.battlegroup).await {
            ctx.log_warn(&format!("pre-update backup failed: {err:#}"))?;
        }

        ctx.log_info("stopping battlegroup")?;
        ctx.env.bg_cli.stop().await?;
        battlegroup_cli::wait_until_stopped(
            &ctx.env.kubectl,
            &pending.namespace,
            &pending.battlegroup,
            Duration::from_secs(900),
        )
        .await?;

        ctx.log_info("applying update-from-downloads")?;
        ctx.env.bg_cli.update_from_downloads().await?;

        ctx.log_info("starting battlegroup")?;
        ctx.env.bg_cli.start().await?;
        let summary = battlegroup_cli::wait_until_running(
            &ctx.env.kubectl,
            &pending.namespace,
            &pending.battlegroup,
            Duration::from_secs(1200),
        )
        .await?;

        ctx.log_info(&format!(
            "update complete phase={} serverGroupPhase={} ready={}/{}",
            summary.phase, summary.server_group_phase, summary.ready, summary.size
        ))?;
        ctx.store.clear_pending_update()?;

        if let Err(err) = ctx
            .env
            .mq
            .publish_service_broadcast(
                "Server update",
                "Server update is complete and the server is back online.",
                60,
            )
            .await
        {
            ctx.log_warn(&format!("post-update broadcast failed: {err:#}"))?;
        }
        Ok(TaskOutcome::Done)
    }
}

async fn backup_one(ctx: &TaskCtx, bg_name: &str) -> Result<()> {
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_name = format!("{}-pre-update-{}.backup", bg_name, stamp);
    ctx.env.bg_cli.backup(&backup_name).await
}
