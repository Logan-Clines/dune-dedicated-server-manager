use anyhow::{Context, Result};
use serde::Serialize;

use super::conn::PgClient;

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    #[serde(rename = "flsId")]
    pub fls_id: String,
    pub name: String,
    pub online: String,
    #[serde(rename = "lastSeen")]
    pub last_seen: String,
}

const PLAYERS_SQL: &str = "
WITH matches AS (
    SELECT DISTINCT
        COALESCE(acct.\"user\"::text, '') AS fls_id,
        COALESCE(ps.character_name, '')   AS character_name,
        COALESCE(ps.online_status::text, '') AS online_status,
        COALESCE(
            to_char(ps.last_avatar_activity AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS'),
            ''
        ) AS last_seen
    FROM dune.player_state ps
    LEFT JOIN dune.accounts acct           ON acct.id = ps.account_id
    LEFT JOIN dune.encrypted_accounts enc  ON enc.id  = ps.account_id
    WHERE lower(ps.character_name) LIKE lower($1)
       OR lower(convert_from(enc.encrypted_funcom_id, 'UTF8')) LIKE lower($1)
)
SELECT fls_id, character_name, online_status, last_seen
FROM matches
WHERE fls_id <> ''
ORDER BY
    CASE WHEN lower(online_status) = 'online' THEN 0 ELSE 1 END,
    last_seen DESC,
    character_name ASC
LIMIT $2;
";

pub async fn search_players(
    pg: &PgClient,
    namespace: &str,
    query: &str,
    limit: u32,
) -> Result<Vec<Player>> {
    let safe_limit = limit.clamp(1, 200) as i64;
    let pattern = format!("%{}%", query);

    let state = pg.client(namespace).await?;
    let rows = state
        .client()
        .query(PLAYERS_SQL, &[&pattern, &safe_limit])
        .await
        .context("running player search query")?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(Player {
            fls_id: row.get::<_, String>(0),
            name: row.get::<_, String>(1),
            online: row.get::<_, String>(2),
            last_seen: row.get::<_, String>(3),
        });
    }
    Ok(out)
}
