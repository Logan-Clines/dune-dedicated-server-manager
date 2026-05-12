use anyhow::{anyhow, Context, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{AttachParams, ListParams},
    Api,
};
use tokio::io::AsyncReadExt;

use crate::{models::DatabaseWorldPartition, state::AppState};

const WORLD_PARTITIONS_QUERY: &str = "select coalesce(json_agg(t), '[]'::json) from (select partition_id, server_id, map, partition_definition::text as partition_definition, dimension_index, blocked, label from world_partition order by map, partition_id) t";

pub async fn list_world_partitions(state: &AppState) -> Result<Vec<DatabaseWorldPartition>> {
    let pod_name = find_database_pod(state).await?;
    let mut attached = Api::<Pod>::namespaced(state.client.clone(), &state.namespace)
        .exec(
            &pod_name,
            vec![
                "psql",
                "-h",
                "127.0.0.1",
                "-p",
                "15432",
                "-U",
                "dune",
                "-d",
                "dune",
                "-t",
                "-A",
                "-c",
                WORLD_PARTITIONS_QUERY,
            ],
            &AttachParams::default().stderr(true),
        )
        .await
        .with_context(|| format!("failed to exec psql in database pod {pod_name}"))?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut reader) = attached.stdout() {
        reader
            .read_to_string(&mut stdout)
            .await
            .context("failed to read psql stdout")?;
    }
    if let Some(mut reader) = attached.stderr() {
        reader
            .read_to_string(&mut stderr)
            .await
            .context("failed to read psql stderr")?;
    }
    attached
        .join()
        .await
        .with_context(|| format!("psql exited with an error: {}", stderr.trim()))?;

    serde_json::from_str(stdout.trim())
        .with_context(|| "failed to parse world_partition query output".to_string())
}

async fn find_database_pod(state: &AppState) -> Result<String> {
    let pods: Api<Pod> = Api::namespaced(state.client.clone(), &state.namespace);
    let list = pods
        .list(&ListParams::default())
        .await
        .context("failed to list pods while locating database")?;
    list.items
        .into_iter()
        .filter_map(|pod| pod.metadata.name)
        .find(|name| name.contains("-db-dbdepl-sts-0"))
        .ok_or_else(|| anyhow!("database pod was not found"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_world_partition_rows() {
        let json = r#"[{"partition_id":8,"server_id":"server-a","map":"DeepDesert_0","partition_definition":"{}","dimension_index":0,"blocked":false,"label":"DeepDesert_0"}]"#;

        let rows: Vec<DatabaseWorldPartition> = serde_json::from_str(json).unwrap();

        assert_eq!(rows[0].partition_id, 8);
        assert_eq!(rows[0].server_id.as_deref(), Some("server-a"));
        assert_eq!(rows[0].map, "DeepDesert_0");
    }
}
