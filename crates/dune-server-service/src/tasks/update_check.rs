use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;

use crate::kubectl::battlegroup as bg;
use crate::kubectl::steam;
use crate::scheduler::{Schedule, Task, TaskCtx, TaskOutcome};
use crate::store::PendingUpdateRecord;

/// Replaces `scripts/cron-battlegroup-update-check`. Polls Steam for the
/// public-branch buildid, compares to local + live versions, and on a real
/// delta downloads the new payload and writes a `pending_update` row that
/// `UpdateApplyTask` reads on its next tick.
pub struct UpdateCheckTask;

#[async_trait]
impl Task for UpdateCheckTask {
    fn id(&self) -> &'static str {
        "update-check"
    }

    fn schedule(&self) -> Schedule {
        Schedule::interval_secs(15 * 60)
    }

    async fn run(&self, ctx: &TaskCtx) -> Result<TaskOutcome> {
        // If a pending update already exists, nothing to do.
        if ctx.store.load_pending_update()?.is_some() {
            ctx.log_info("pending update already scheduled; skipping check")?;
            return Ok(TaskOutcome::Noop);
        }

        let cluster = ctx.env.cluster.get().await?;
        let bg_name = bg::bg_name(&ctx.env.kubectl, &cluster.namespace).await?;
        let bg_doc = bg::bg_json(&ctx.env.kubectl, &cluster.namespace, &bg_name).await?;
        let live_version = steam::extract_live_version(&bg_doc);

        let latest = ctx.env.steamcmd.latest_public_build().await?;
        let local = ctx.env.steamcmd.local_build().await?;

        ctx.log_info(&format!(
            "update check latest_build={} local_build={} live_version={}",
            latest.buildid,
            local.as_deref().unwrap_or("unknown"),
            live_version.as_deref().unwrap_or("unknown"),
        ))?;

        if let Some(local_build) = local.as_deref() {
            if local_build == latest.buildid {
                ctx.log_info("no Steam update available")?;
                return Ok(TaskOutcome::Noop);
            }
        }

        if ctx.dry_run {
            ctx.log_info("[dry-run] would download Steam update + schedule pending row")?;
            return Ok(TaskOutcome::Done);
        }

        ctx.log_info("Steam update detected; downloading before scheduling")?;
        ctx.env.steamcmd.download_update().await?;

        let new_local = ctx.env.steamcmd.local_build().await?;
        let downloaded_version = ctx
            .env
            .steamcmd
            .downloaded_version()
            .await?
            .ok_or_else(|| anyhow::anyhow!("downloaded version file missing or empty"))?;

        if let Some(live) = live_version.as_deref() {
            if downloaded_version == live {
                ctx.log_info(&format!(
                    "downloaded BG version equals live version ({downloaded_version}); no update scheduled"
                ))?;
                return Ok(TaskOutcome::Noop);
            }
        }

        let due_ts = Utc::now().timestamp() + ctx.env.update_lead_secs;
        ctx.store.upsert_pending_update(&PendingUpdateRecord {
            battlegroup: bg_name.clone(),
            namespace: cluster.namespace.clone(),
            latest_steam_build: Some(latest.buildid.clone()),
            local_steam_build: new_local,
            live_version,
            downloaded_version: downloaded_version.clone(),
            due_ts,
            created_ts: 0,
        })?;
        ctx.log_info(&format!(
            "scheduled update bg={bg_name} downloaded_version={downloaded_version} due_ts={due_ts}"
        ))?;

        if let Err(err) = ctx
            .env
            .mq
            .publish_service_broadcast(
                "Server update",
                "A server update is ready and will be applied in 30 minutes. The server will restart.",
                60,
            )
            .await
        {
            ctx.log_warn(&format!("warning broadcast failed: {err:#}"))?;
        }
        Ok(TaskOutcome::Done)
    }
}
