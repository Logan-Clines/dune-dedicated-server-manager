use std::sync::Arc;

use anyhow::Result;

use crate::kubectl::ClusterCache;
use crate::postgres::{search_players as pg_search_players, PgClient, Player};

/// Thin wrapper that resolves the current namespace from the cluster cache and
/// delegates to the tokio-postgres query.
pub async fn search_players(
    pg: &Arc<PgClient>,
    cluster: &ClusterCache,
    query: &str,
    limit: u32,
) -> Result<Vec<Player>> {
    let cluster = cluster.get().await?;
    pg_search_players(pg, &cluster.namespace, query, limit).await
}
