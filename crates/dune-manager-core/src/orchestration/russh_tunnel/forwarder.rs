//! Local SSH port forwarder backed by russh `direct-tcpip` channels.
//!
//! The local TCP listener stays up for the lifetime of the [`LocalForwarder`].
//! The SSH session itself is owned by [`SessionProvider`], which lazily
//! reconnects with backoff when the previous session dies — so a flaky upstream
//! turns into a brief blocking reconnect on the next request rather than a
//! permanent dead-tunnel state.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::net::TcpListener;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

use crate::errors::failure;
use crate::models::{CommandFailure, CommandResult};
use crate::orchestration::russh_runner::session::{
    close as close_session, connect_with_config, shared_runtime, tunnel_config, SessionHandle,
};
use crate::orchestration::russh_runner::RusshTarget;

use super::proxy::proxy_one_connection;

/// How long [`SessionProvider::get`] will block trying to re-establish a dead
/// session before giving up and returning the last error. Single client
/// requests will see this as their request latency on first contact after a
/// blip.
const RECONNECT_DEADLINE: Duration = Duration::from_secs(15);

/// Initial reconnect backoff. Doubles up to `RECONNECT_MAX_BACKOFF`.
const RECONNECT_INITIAL_BACKOFF: Duration = Duration::from_millis(500);
const RECONNECT_MAX_BACKOFF: Duration = Duration::from_secs(3);

/// Active local SSH port forwarder.
///
/// Dropping the forwarder requests shutdown and aborts the accept loop. To
/// shut down gracefully and wait for the loop to exit, call
/// [`LocalForwarder::stop`].
pub struct LocalForwarder {
    shutdown: watch::Sender<bool>,
    task: Option<JoinHandle<()>>,
    local_addr: SocketAddr,
    provider: Arc<SessionProvider>,
}

impl LocalForwarder {
    /// Connects to `target`, binds a TCP listener on `127.0.0.1:local_port`
    /// (or a randomly chosen port if `local_port` is `0`), and spawns the
    /// background accept loop. The initial SSH session is established eagerly
    /// so an unreachable host fails fast at start() rather than on first use.
    pub fn start(
        target: &RusshTarget,
        local_port: u16,
        remote_host: &str,
        remote_port: u16,
    ) -> CommandResult<Self> {
        target.validate()?;
        let target = target.clone();
        let remote_host = remote_host.to_string();
        let rt = shared_runtime();
        rt.block_on(async move {
            let provider = Arc::new(SessionProvider::new(target));
            // Eager initial connect to surface bad creds / unreachable host
            // before we hand back a forwarder handle.
            provider.get().await?;
            let listener = TcpListener::bind(("127.0.0.1", local_port))
                .await
                .map_err(|err| {
                    failure(format!(
                        "Failed to bind local tunnel port {local_port}: {err}"
                    ))
                })?;
            let local_addr = listener
                .local_addr()
                .map_err(|err| failure(format!("Failed to read local tunnel port: {err}")))?;
            let (shutdown_tx, shutdown_rx) = watch::channel(false);
            let task = tokio::spawn(accept_loop(
                listener,
                provider.clone(),
                remote_host,
                remote_port,
                shutdown_rx,
            ));
            Ok(LocalForwarder {
                shutdown: shutdown_tx,
                task: Some(task),
                local_addr,
                provider,
            })
        })
    }

    /// Returns the actual bound local TCP port.
    pub fn local_port(&self) -> u16 {
        self.local_addr.port()
    }

    /// Returns `true` only when the forwarder has been explicitly stopped — a
    /// transiently dead SSH session no longer flips this, since the provider
    /// will reconnect on demand.
    pub fn is_finished(&self) -> bool {
        self.task
            .as_ref()
            .map(JoinHandle::is_finished)
            .unwrap_or(true)
    }

    /// Signals shutdown, waits for the accept loop to exit, and closes the
    /// SSH session.
    pub fn stop(mut self) {
        let _ = self.shutdown.send(true);
        if let Some(task) = self.task.take() {
            let provider = self.provider.clone();
            shared_runtime().block_on(async move {
                let _ = tokio::time::timeout(Duration::from_secs(5), task).await;
                provider.close().await;
            });
        }
    }
}

impl Drop for LocalForwarder {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

/// Owns the current SSH session and reconnects on demand. Callers hand off a
/// clone of this and never touch raw [`SessionHandle`]s directly.
pub(crate) struct SessionProvider {
    target: RusshTarget,
    // A single Mutex (not RwLock) is fine: lookups are cheap, only one
    // reconnect should ever be in flight, and contention is only on the rare
    // session-died path.
    current: Mutex<Option<Arc<SessionHandle>>>,
}

impl SessionProvider {
    fn new(target: RusshTarget) -> Self {
        Self {
            target,
            current: Mutex::new(None),
        }
    }

    /// Returns a healthy SSH session, reconnecting with backoff if the
    /// previous one died. Blocks for at most `RECONNECT_DEADLINE` before
    /// returning the last reconnect error.
    pub(crate) async fn get(&self) -> CommandResult<Arc<SessionHandle>> {
        let mut guard = self.current.lock().await;
        if let Some(session) = guard.as_ref() {
            if !session.is_closed() {
                return Ok(session.clone());
            }
            // Drop the dead handle so we can take a fresh one.
            *guard = None;
        }

        let started = Instant::now();
        let mut backoff = RECONNECT_INITIAL_BACKOFF;
        let mut last_err: Option<CommandFailure> = None;
        loop {
            match connect_with_config(&self.target, tunnel_config()).await {
                Ok(session) => {
                    let session = Arc::new(session);
                    *guard = Some(session.clone());
                    if last_err.is_some() {
                        eprintln!(
                            "russh tunnel: ssh session re-established to {}",
                            self.target.destination()
                        );
                    }
                    return Ok(session);
                }
                Err(err) => {
                    eprintln!(
                        "russh tunnel: reconnect to {} failed: {}",
                        self.target.destination(),
                        err.message
                    );
                    last_err = Some(err);
                    if started.elapsed() >= RECONNECT_DEADLINE {
                        return Err(last_err.unwrap_or_else(|| {
                            failure(format!(
                                "ssh reconnect to {} timed out",
                                self.target.destination()
                            ))
                        }));
                    }
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(RECONNECT_MAX_BACKOFF);
                }
            }
        }
    }

    async fn close(&self) {
        let mut guard = self.current.lock().await;
        if let Some(session) = guard.take() {
            close_session(&session).await;
        }
    }
}

async fn accept_loop(
    listener: TcpListener,
    provider: Arc<SessionProvider>,
    remote_host: String,
    remote_port: u16,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            biased;
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            accept = listener.accept() => {
                let (stream, peer) = match accept {
                    Ok(pair) => pair,
                    Err(_) => continue,
                };
                let provider = provider.clone();
                let remote_host = remote_host.clone();
                let origin_ip = peer.ip().to_string();
                let origin_port = peer.port();
                tokio::spawn(async move {
                    let session = match provider.get().await {
                        Ok(s) => s,
                        Err(err) => {
                            eprintln!(
                                "russh tunnel: dropping incoming connection — no healthy session ({}:{}): {}",
                                remote_host, remote_port, err.message
                            );
                            // stream is dropped; client sees an immediate
                            // connection reset rather than a hung read.
                            return;
                        }
                    };
                    proxy_one_connection(
                        &session,
                        stream,
                        remote_host,
                        remote_port,
                        (origin_ip, origin_port),
                    )
                    .await;
                });
            }
        }
    }
}
