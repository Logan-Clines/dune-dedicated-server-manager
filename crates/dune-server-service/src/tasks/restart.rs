use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;

use crate::kubectl::battlegroup as bg;
use crate::kubectl::battlegroup_cli;
use crate::scheduler::{Schedule, Task, TaskCtx, TaskOutcome};

/// Replaces `scripts/daily-battlegroup-restart`. Stops the BattleGroup, waits
/// for clean shutdown, restarts it, and waits for full readiness. Schedule
/// fires at the configured wall-clock hour:minute in the IANA timezone (default
/// 05:00 Europe/Amsterdam).
pub struct RestartTask;

#[async_trait]
impl Task for RestartTask {
    fn id(&self) -> &'static str {
        "restart"
    }

    fn schedule(&self) -> Schedule {
        // Resolved against the env's tz at scheduler tick time.
        Schedule::daily(5, 0)
    }

    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutcome> {
        let cluster = ctx.env.cluster.get().await?;
        let bg_name = bg::bg_name(&ctx.env.kubectl, &cluster.namespace).await?;
        ctx.log_info(&format!(
            "stopping battlegroup bg={bg_name} ns={}",
            cluster.namespace
        ))?;

        if ctx.dry_run {
            ctx.log_info("[dry-run] would stop and start battlegroup")?;
            return Ok(TaskOutcome::Done);
        }

        ctx.env.bg_cli.stop().await?;
        battlegroup_cli::wait_until_stopped(
            &ctx.env.kubectl,
            &cluster.namespace,
            &bg_name,
            Duration::from_secs(900),
        )
        .await?;

        ctx.log_info(&format!("starting battlegroup bg={bg_name}"))?;
        ctx.env.bg_cli.start().await?;
        let summary = battlegroup_cli::wait_until_running(
            &ctx.env.kubectl,
            &cluster.namespace,
            &bg_name,
            Duration::from_secs(1200),
        )
        .await?;
        ctx.log_info(&format!(
            "battlegroup restart complete phase={} serverGroupPhase={} ready={}/{}",
            summary.phase, summary.server_group_phase, summary.ready, summary.size
        ))?;
        Ok(TaskOutcome::Done)
    }
}
