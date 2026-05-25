use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid DUNE_DASHBOARD_PORT: {0}")]
    InvalidPort(String),
    #[error("invalid DUNE_SERVICE_TIME_ZONE: {0}")]
    InvalidTimeZone(String),
    #[error(
        "refusing to bind DUNE_DASHBOARD_HOST={0}; set DUNE_ALLOW_EXTERNAL_BIND=1 to override"
    )]
    NonLoopbackBindForbidden(String),
}
