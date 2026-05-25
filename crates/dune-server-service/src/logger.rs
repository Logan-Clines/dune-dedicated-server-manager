use std::io::IsTerminal;
use std::sync::OnceLock;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static ACTIVE_TOKEN: OnceLock<String> = OnceLock::new();

pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,dune_server_service=debug"));

    // Colour escapes look fine in an interactive terminal but render as raw
    // `[2m[32m...[0m` noise when journalctl / a tail panel reads the log
    // file. Only emit ANSI when stdout is actually a TTY.
    let with_ansi = std::io::stdout().is_terminal();

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_ansi(with_ansi),
        )
        .init();
}

/// Register the active command-auth token so it can be scrubbed from log/error
/// strings. Callers must invoke `redact` before emitting any string that may
/// have come from a command-publish error path.
///
/// Only the first call wins; subsequent token loads (e.g. after a manual
/// refresh) are ignored. Token rotation requires a daemon restart.
pub fn register_token(token: &str) {
    let _ = ACTIVE_TOKEN.set(token.to_string());
}

/// Replace every occurrence of the registered token with `***` so accidental
/// inclusion in error/log strings does not leak it through journald.
pub fn redact(input: &str) -> std::borrow::Cow<'_, str> {
    match ACTIVE_TOKEN.get() {
        Some(token) if !token.is_empty() && input.contains(token) => {
            std::borrow::Cow::Owned(input.replace(token, "***"))
        }
        _ => std::borrow::Cow::Borrowed(input),
    }
}
