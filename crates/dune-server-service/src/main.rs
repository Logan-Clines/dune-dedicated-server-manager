use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context, Result};

use dune_server_service::admin::MqPublisher;
use dune_server_service::config::{resolve_command_auth_token, ServiceConfig};
use dune_server_service::http::{self, AppState};
use dune_server_service::kubectl::{BattlegroupCli, ClusterCache, KubectlClient, SteamCmd};
use dune_server_service::logger;
use dune_server_service::postgres::{PgClient, PgConfig};
use dune_server_service::scheduler::{Scheduler, TaskRunner};
use dune_server_service::store::Store;
use dune_server_service::tasks::TaskEnv;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PATH_EXTRAS: &str =
    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/home/dune/.local/bin";

fn main() -> ExitCode {
    // SAFETY: set_var requires no other threads to be running. We are still
    // single-threaded here (before the tokio runtime is built below). Inject a
    // sane PATH that covers common kubectl / battlegroup / steamcmd locations
    // so the daemon's subprocesses don't depend on the init system's PATH.
    unsafe {
        let merged = match std::env::var_os("PATH") {
            Some(existing) if !existing.is_empty() => {
                let mut v = std::ffi::OsString::from(DEFAULT_PATH_EXTRAS);
                v.push(":");
                v.push(existing);
                v
            }
            _ => DEFAULT_PATH_EXTRAS.into(),
        };
        std::env::set_var("PATH", merged);
    }

    logger::init();

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(err) => {
            tracing::error!(error = %err, "failed to build tokio runtime");
            return ExitCode::FAILURE;
        }
    };

    runtime.block_on(async {
        match run().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                tracing::error!(error = %err, "dune-server-service exiting with error");
                ExitCode::FAILURE
            }
        }
    })
}

async fn run() -> Result<()> {
    let cfg = ServiceConfig::from_env().context("loading config")?;
    let token = resolve_command_auth_token(&cfg.command_auth_token_file);
    logger::register_token(&token);

    tracing::info!(
        version = VERSION,
        bind = %format!("{}:{}", cfg.dashboard_host, cfg.dashboard_port),
        db_path = %cfg.db_path.display(),
        time_zone = %cfg.time_zone,
        "dune-server-service starting"
    );

    let store = Store::open(&cfg.db_path).context("opening sqlite store")?;

    let kubectl = KubectlClient::new(
        cfg.kubectl_use_sudo,
        cfg.namespace_override.clone(),
        cfg.mq_pod_override.clone(),
        cfg.db_pod_override.clone(),
    );
    let cluster = ClusterCache::new(kubectl.clone());

    let bg_cli = BattlegroupCli::new(&cfg.bin_dir);
    let download_path = cfg
        .steamcmd_download_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("/home/dune/.dune/download"));
    let steamcmd_bin = cfg
        .steamcmd_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("/home/dune/.local/bin/steamcmd"));
    let steamcmd = SteamCmd::new(steamcmd_bin, download_path.clone());
    if let Err(err) = steamcmd.ensure_wrapper() {
        tracing::warn!(error = %err, "could not ensure steamcmd wrapper; update-check will fail until resolved");
    }

    let mq = Arc::new(MqPublisher::new(kubectl.clone(), cluster.clone(), token.clone()));
    let pg = Arc::new(PgClient::new(
        kubectl.clone(),
        PgConfig {
            host_override: cfg.pg_host_override.clone(),
            user_override: cfg.pg_user_override.clone(),
            db_override: cfg.pg_db_override.clone(),
        },
    ));

    let env = Arc::new(TaskEnv {
        kubectl: kubectl.clone(),
        cluster: cluster.clone(),
        bg_cli,
        steamcmd,
        mq,
        pg,
        bin_dir: cfg.bin_dir.clone(),
        download_path,
        update_lead_secs: 30 * 60,
        restart_hour: 5,
        restart_minute: 0,
        restart_warning_frequency_secs: 600,
        restart_warning_duration_secs: 1800,
        restart_tz: cfg.time_zone,
    });

    let runner = Arc::new(TaskRunner::new(store.clone(), env.clone()));
    let mut scheduler = Scheduler::new(runner.clone(), cfg.time_zone);
    for task in dune_server_service::tasks::build_all(env.clone()) {
        scheduler.add(task);
    }
    let cancel = scheduler.cancel_token();
    scheduler.start();

    let state = AppState::new(store, env, runner);
    let server_cancel = cancel.clone();

    let http_handle = tokio::spawn(async move {
        if let Err(err) = http::serve(&cfg, state, server_cancel).await {
            tracing::error!(error = %err, "http server exited with error");
        }
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("ctrl-c received; shutting down");
        }
        _ = wait_sigterm() => {
            tracing::info!("SIGTERM received; shutting down");
        }
    }

    cancel.cancel();
    scheduler.shutdown().await;
    let _ = http_handle.await;
    tracing::info!("dune-server-service stopped");
    Ok(())
}

#[cfg(unix)]
async fn wait_sigterm() {
    use tokio::signal::unix::{signal, SignalKind};
    if let Ok(mut sig) = signal(SignalKind::terminate()) {
        sig.recv().await;
    } else {
        std::future::pending::<()>().await;
    }
}

#[cfg(not(unix))]
async fn wait_sigterm() {
    std::future::pending::<()>().await;
}
