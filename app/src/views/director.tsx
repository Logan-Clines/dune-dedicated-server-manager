import { Activity, Map, RefreshCw } from "lucide-react";
import { useMemo, useState } from "react";
import { EmptyState, StatusPill } from "../components/primitives";
import type { DirectorMapSummary, DirectorServerSummary } from "../types";

type DirectorViewProps = {
  directorMaps: DirectorMapSummary[];
  busy: boolean;
  onReload: () => void;
  onEditMap: (mapName: string) => void;
  onClearMapOverride: (mapName: string) => void;
};

export function DirectorView({
  directorMaps,
  busy,
  onReload,
  onEditMap,
  onClearMapOverride
}: DirectorViewProps) {
  const [selectedMapName, setSelectedMapName] = useState("");
  const selectedMap = useMemo(
    () => directorMaps.find((map) => map.name === selectedMapName) ?? directorMaps[0] ?? null,
    [directorMaps, selectedMapName]
  );

  return (
    <>
      <section className="panel">
        <div className="panel-title">
          <h2>Director Maps</h2>
          <div className="button-row">
            <button onClick={onReload} disabled={busy}>
              <RefreshCw size={16} />
              Reload
            </button>
            <Map size={19} />
          </div>
        </div>
        {directorMaps.length === 0 ? (
          <EmptyState text="No Director map data loaded." />
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Map</th>
                  <th>Kind</th>
                  <th>Players</th>
                  <th>Queue</th>
                  <th>Servers</th>
                  <th>Override</th>
                </tr>
              </thead>
              <tbody>
                {directorMaps.map((map) => (
                  <tr
                    key={`${map.kind}-${map.name}`}
                    className={map.name === selectedMap?.name ? "selected" : ""}
                    onClick={() => setSelectedMapName(map.name)}
                  >
                    <td>
                      <strong>{map.name}</strong>
                    </td>
                    <td>{map.kind}</td>
                    <td>{map.players}</td>
                    <td>{map.queued}</td>
                    <td>{map.servers.length}</td>
                    <td>
                      <div className="button-row">
                        <StatusPill value={map.hasOverride ? "Active" : "None"} />
                        <button onClick={() => onEditMap(map.name)} disabled={busy}>
                          Edit
                        </button>
                        <button onClick={() => onClearMapOverride(map.name)} disabled={busy || !map.hasOverride}>
                          Clear
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {directorMaps.length > 0 && (
        <section className="panel">
          <div className="panel-title">
            <h2>Server Runtime</h2>
            <Activity size={19} />
          </div>
          {selectedMap ? (
            <DirectorMapDetail map={selectedMap} onEditMap={onEditMap} onClearMapOverride={onClearMapOverride} busy={busy} />
          ) : (
            <EmptyState text="Select a map to inspect runtime detail." />
          )}
        </section>
      )}
    </>
  );
}

function DirectorMapDetail({
  map,
  busy,
  onEditMap,
  onClearMapOverride
}: {
  map: DirectorMapSummary;
  busy: boolean;
  onEditMap: (mapName: string) => void;
  onClearMapOverride: (mapName: string) => void;
}) {
  const statusCounts = map.servers.reduce<Record<string, number>>((counts, server) => {
    counts[server.status] = (counts[server.status] ?? 0) + 1;
    return counts;
  }, {});
  const staleServers = map.servers.filter(
    (server) => server.heartbeatSecondsAgo === null || server.heartbeatSecondsAgo === undefined || server.heartbeatSecondsAgo > 60
  ).length;
  const overrideServers = map.servers.filter((server) => server.hasOverride).length;

  return (
    <article className="runtime-map runtime-map-detail">
      <div className="mini-title">
        <strong>{map.name}</strong>
        <span>{map.kind}</span>
      </div>
      <div className="director-detail-metrics">
        <MetricBox label="Players" value={map.players} />
        <MetricBox label="Online" value={map.online} />
        <MetricBox label="Queued" value={map.queued} />
        <MetricBox label="Servers" value={map.servers.length} />
        <MetricBox label="Stale" value={staleServers} />
        <MetricBox label="Overrides" value={map.hasOverride ? Math.max(1, overrideServers) : overrideServers} />
      </div>
      <div className="director-status-row">
        {Object.entries(statusCounts).length === 0 ? (
          <span>No server status rows</span>
        ) : (
          Object.entries(statusCounts).map(([status, count]) => (
            <span key={status}>
              <StatusPill value={status} /> {count}
            </span>
          ))
        )}
      </div>
      <div className="button-row">
        <button onClick={() => onEditMap(map.name)} disabled={busy}>
          Edit Override
        </button>
        <button onClick={() => onClearMapOverride(map.name)} disabled={busy || !map.hasOverride}>
          Clear Override
        </button>
      </div>
      <div className="runtime-servers detailed">
        {map.servers.length === 0 ? (
          <EmptyState text="No server rows reported." />
        ) : (
          map.servers.map((server) => <DirectorServerRow server={server} mapName={map.name} key={serverKey(map.name, server)} />)
        )}
      </div>
    </article>
  );
}

function DirectorServerRow({ server, mapName }: { server: DirectorServerSummary; mapName: string }) {
  const heartbeat =
    server.heartbeatSecondsAgo === null || server.heartbeatSecondsAgo === undefined
      ? "No heartbeat"
      : `${server.heartbeatSecondsAgo}s ago`;
  return (
    <div>
      <div>
        <strong>{server.label || "Unnamed"}</strong>
        <span className="mono">{server.serverId || "No server id"}</span>
      </div>
      <StatusPill value={server.status} />
      <span>{server.players} players</span>
      <span>{server.online} online</span>
      <span>{server.queued ?? "N/A"} queued</span>
      <span>{heartbeat}</span>
      <span>{server.partitionId === null || server.partitionId === undefined ? "No partition" : `Partition ${server.partitionId}`}</span>
      <span>
        {server.dimensionIndex === null || server.dimensionIndex === undefined ? mapName : `Dimension ${server.dimensionIndex}`}
      </span>
      <StatusPill value={server.hasOverride ? "Override" : "Base"} />
    </div>
  );
}

function MetricBox({ label, value }: { label: string; value: number }) {
  return (
    <div>
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}

function serverKey(mapName: string, server: DirectorServerSummary) {
  return `${mapName}-${server.partitionId ?? "x"}-${server.dimensionIndex ?? "x"}-${server.serverId || server.label}`;
}
