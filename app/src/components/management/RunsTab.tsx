import { useCallback, useEffect, useState } from "react";
import { Badge, Box, Button, Flex, Table, Text } from "@radix-ui/themes";

import { managementApi } from "../../services/management";
import type { LogDto, RunDto } from "../../types/management";

const TASKS: Array<{ id: string; label: string }> = [
  { id: "backup", label: "Backup" },
  { id: "update-check", label: "Update check" },
  { id: "update-apply", label: "Update apply" },
  { id: "restart-notice", label: "Restart notice" },
  { id: "restart", label: "Restart" },
];

export type RunsTabProps = { tunnelId: string };

export default function RunsTab({ tunnelId }: RunsTabProps) {
  const [runs, setRuns] = useState<RunDto[]>([]);
  const [logs, setLogs] = useState<LogDto[]>([]);
  const [selectedRun, setSelectedRun] = useState<number | null>(null);
  const [busyTrigger, setBusyTrigger] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    try {
      const r = await managementApi.listRuns(tunnelId, 50);
      setRuns(r);
      if (selectedRun !== null) {
        const l = await managementApi.listLogs(tunnelId, 500, selectedRun);
        setLogs(l);
      } else {
        const l = await managementApi.listLogs(tunnelId, 100);
        setLogs(l);
      }
    } catch (err) {
      setError(String(err));
    }
  }, [tunnelId, selectedRun]);

  useEffect(() => {
    void reload();
    const handle = setInterval(reload, 5000);
    return () => clearInterval(handle);
  }, [reload]);

  const trigger = useCallback(
    async (task: string) => {
      setBusyTrigger(task);
      try {
        await managementApi.triggerRun(tunnelId, task);
        await reload();
      } catch (err) {
        alert(`Trigger ${task} failed: ${err}`);
      } finally {
        setBusyTrigger(null);
      }
    },
    [reload, tunnelId],
  );

  return (
    <Box mt="3">
      <Flex gap="2" wrap="wrap" mb="3">
        {TASKS.map((t) => (
          <Button
            key={t.id}
            size="1"
            variant="surface"
            disabled={busyTrigger === t.id}
            onClick={() => trigger(t.id)}
          >
            {busyTrigger === t.id ? `Running ${t.label}…` : `Run ${t.label}`}
          </Button>
        ))}
        <Button size="1" variant="ghost" onClick={reload}>
          Refresh
        </Button>
      </Flex>

      {error ? (
        <Text size="1" color="red">
          {error}
        </Text>
      ) : null}

      <Flex gap="3" align="stretch" wrap="wrap">
        <Box style={{ flex: "1 1 460px", minWidth: 0 }}>
          <Text size="2" weight="medium">
            Recent runs
          </Text>
          <Table.Root variant="surface" size="1" mt="1">
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeaderCell>ID</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>Task</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>Status</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>Started</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>Duration</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {runs.map((run) => (
                <Table.Row
                  key={run.id}
                  onClick={() => setSelectedRun(run.id === selectedRun ? null : run.id)}
                  style={{ cursor: "pointer", background: selectedRun === run.id ? "var(--accent-3)" : undefined }}
                >
                  <Table.Cell className="mono">{run.id}</Table.Cell>
                  <Table.Cell>{run.taskId}</Table.Cell>
                  <Table.Cell>
                    <Badge color={statusColor(run.status)}>{run.status}</Badge>
                  </Table.Cell>
                  <Table.Cell className="mono" style={{ fontSize: 11 }}>
                    {fmtTime(run.startedAt)}
                  </Table.Cell>
                  <Table.Cell className="mono">
                    {run.durationMs != null ? `${(run.durationMs / 1000).toFixed(1)}s` : "—"}
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
        </Box>
        <Box style={{ flex: "2 1 600px", minWidth: 0 }}>
          <Flex justify="between" align="baseline">
            <Text size="2" weight="medium">
              Logs {selectedRun !== null ? `(run ${selectedRun})` : "(recent)"}
            </Text>
            {selectedRun !== null ? (
              <Button size="1" variant="ghost" onClick={() => setSelectedRun(null)}>
                Show all
              </Button>
            ) : null}
          </Flex>
          <Box
            mt="1"
            className="mono"
            style={{
              maxHeight: 480,
              overflowY: "auto",
              fontSize: 11,
              padding: 8,
              background: "var(--color-panel-translucent)",
              whiteSpace: "pre-wrap",
            }}
          >
            {logs.length === 0 ? (
              <Text color="gray">No logs.</Text>
            ) : (
              logs.map((log) => (
                <div key={log.id}>
                  <span style={{ color: logColor(log.level) }}>{log.level.toUpperCase()}</span>{" "}
                  <span style={{ color: "var(--gray-9)" }}>{fmtTime(log.createdAt)}</span>{" "}
                  {log.message}
                </div>
              ))
            )}
          </Box>
        </Box>
      </Flex>
    </Box>
  );
}

function statusColor(s: string): "gray" | "green" | "red" | "amber" {
  if (s === "success") return "green";
  if (s === "failed") return "red";
  if (s === "running") return "amber";
  return "gray";
}

function logColor(level: string): string {
  if (level === "error") return "var(--red-10)";
  if (level === "warn") return "var(--amber-10)";
  return "var(--gray-11)";
}

function fmtTime(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toISOString().slice(11, 19);
}
