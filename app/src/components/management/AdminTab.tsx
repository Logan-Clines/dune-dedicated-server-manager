import { useCallback, useEffect, useMemo, useState } from "react";
import { Badge, Box, Button, Flex, Select, Table, Text, TextArea, TextField } from "@radix-ui/themes";

import { managementApi } from "../../services/management";
import type {
  CommandSpec,
  FieldSpec,
  HistoryDto,
  ItemDto,
  PlayerDto,
  PublishResultDto,
  VehicleDto,
} from "../../types/management";

export type AdminTabProps = { tunnelId: string };

export default function AdminTab({ tunnelId }: AdminTabProps) {
  const [commands, setCommands] = useState<CommandSpec[]>([]);
  const [selected, setSelected] = useState<CommandSpec | null>(null);
  const [values, setValues] = useState<Record<string, unknown>>({});
  const [history, setHistory] = useState<HistoryDto[]>([]);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<PublishResultDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    managementApi
      .listCommands(tunnelId)
      .then(setCommands)
      .catch((err) => setError(String(err)));
    void refreshHistory();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tunnelId]);

  const refreshHistory = useCallback(async () => {
    try {
      const list = await managementApi.history(tunnelId, 30);
      setHistory(list);
    } catch (err) {
      setError(String(err));
    }
  }, [tunnelId]);

  useEffect(() => {
    if (!selected) return;
    const initial: Record<string, unknown> = {};
    for (const field of selected.fields) {
      if (field.default !== undefined) initial[field.key] = field.default;
    }
    setValues(initial);
    setResult(null);
  }, [selected]);

  const grouped = useMemo(() => groupByCategory(commands), [commands]);

  const publish = useCallback(async () => {
    if (!selected) return;
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      const out = await managementApi.publish(tunnelId, selected.id, values);
      setResult(out);
      await refreshHistory();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }, [selected, tunnelId, values, refreshHistory]);

  return (
    <Flex mt="3" gap="3" align="stretch" wrap="wrap">
      <Box style={{ flex: "0 0 240px", minWidth: 0 }}>
        <Text size="2" weight="medium">
          Commands
        </Text>
        {Object.entries(grouped).map(([category, specs]) => (
          <Box key={category} mt="2">
            <Text size="1" color="gray" style={{ textTransform: "uppercase", letterSpacing: 0.5 }}>
              {category}
            </Text>
            <Flex direction="column" gap="1" mt="1">
              {specs.map((spec) => (
                <Button
                  key={spec.id}
                  size="1"
                  variant={selected?.id === spec.id ? "solid" : "surface"}
                  color={spec.destructive ? "red" : undefined}
                  onClick={() => setSelected(spec)}
                  style={{ justifyContent: "flex-start" }}
                >
                  {spec.label}
                </Button>
              ))}
            </Flex>
          </Box>
        ))}
      </Box>
      <Box style={{ flex: "1 1 400px", minWidth: 0 }}>
        {selected ? (
          <Box>
            <Flex justify="between" align="baseline" wrap="wrap" gap="2">
              <Text size="3" weight="medium">
                {selected.label}
              </Text>
              {selected.destructive ? <Badge color="red">destructive</Badge> : null}
            </Flex>
            <Text size="1" color="gray">
              {selected.describe}
            </Text>
            <Flex direction="column" gap="3" mt="3">
              {selected.fields.map((field) => (
                <FieldInput
                  key={field.key}
                  field={field}
                  value={values[field.key]}
                  onChange={(v) => setValues((prev) => ({ ...prev, [field.key]: v }))}
                  tunnelId={tunnelId}
                />
              ))}
            </Flex>
            <Flex mt="3" gap="2" align="center">
              <Button onClick={publish} disabled={busy}>
                {busy ? "Publishing…" : "Publish"}
              </Button>
              {result ? (
                <Badge color={result.ok ? "green" : "red"}>{result.ok ? "ok" : "failed"}</Badge>
              ) : null}
            </Flex>
            {result && !result.ok && result.error ? (
              <Text size="1" color="red" mt="2">
                {result.error}
              </Text>
            ) : null}
            {result?.output ? (
              <Box
                mt="2"
                className="mono"
                style={{ fontSize: 11, padding: 6, background: "var(--color-panel-translucent)", whiteSpace: "pre-wrap" }}
              >
                {result.output}
              </Box>
            ) : null}
            {error ? (
              <Text size="1" color="red" mt="2">
                {error}
              </Text>
            ) : null}
          </Box>
        ) : (
          <Text color="gray">Select a command on the left.</Text>
        )}
      </Box>
      <Box style={{ flex: "1 1 320px", minWidth: 0 }}>
        <Text size="2" weight="medium">
          Recent publishes
        </Text>
        <Table.Root variant="surface" size="1" mt="1">
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeaderCell>Cmd</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>OK</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>When</Table.ColumnHeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {history.map((h) => (
              <Table.Row key={h.id}>
                <Table.Cell className="mono" style={{ fontSize: 11 }}>
                  {h.command}
                </Table.Cell>
                <Table.Cell>
                  <Badge color={h.ok ? "green" : "red"}>{h.ok ? "ok" : "fail"}</Badge>
                </Table.Cell>
                <Table.Cell className="mono" style={{ fontSize: 11 }}>
                  {fmtTime(h.createdAt)}
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table.Root>
      </Box>
    </Flex>
  );
}

function groupByCategory(specs: CommandSpec[]): Record<string, CommandSpec[]> {
  const out: Record<string, CommandSpec[]> = {};
  for (const spec of specs) {
    if (!out[spec.category]) out[spec.category] = [];
    out[spec.category].push(spec);
  }
  return out;
}

function fmtTime(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toISOString().slice(11, 19);
}

function FieldInput({
  field,
  value,
  onChange,
  tunnelId,
}: {
  field: FieldSpec;
  value: unknown;
  onChange: (v: unknown) => void;
  tunnelId: string;
}) {
  const showLookup =
    (field.key === "ItemName" && true) ||
    (field.key === "ClassName" && true) ||
    (field.key === "PlayerId" && true);
  return (
    <Box>
      <Flex justify="between" align="baseline">
        <Text size="2" weight="medium">
          {field.label}
          {field.required ? " *" : ""}
        </Text>
        {field.helper ? (
          <Text size="1" color="gray">
            {field.helper}
          </Text>
        ) : null}
      </Flex>
      {renderInput(field, value, onChange)}
      {showLookup ? <Lookup field={field} onPick={onChange} tunnelId={tunnelId} /> : null}
    </Box>
  );
}

function renderInput(field: FieldSpec, value: unknown, onChange: (v: unknown) => void) {
  const strValue = value === undefined || value === null ? "" : String(value);
  if (field.kind === "select" && field.options) {
    return (
      <Select.Root value={strValue || field.options[0].value} onValueChange={onChange}>
        <Select.Trigger />
        <Select.Content>
          {field.options.map((opt) => (
            <Select.Item key={opt.value} value={opt.value}>
              {opt.label}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
    );
  }
  if (field.kind === "text") {
    return <TextArea value={strValue} onChange={(e) => onChange(e.target.value)} rows={3} />;
  }
  return (
    <TextField.Root
      value={strValue}
      onChange={(e) => {
        const raw = e.target.value;
        if (field.kind === "int" || field.kind === "float") {
          onChange(raw === "" ? "" : Number(raw));
        } else {
          onChange(raw);
        }
      }}
    />
  );
}

function Lookup({
  field,
  onPick,
  tunnelId,
}: {
  field: FieldSpec;
  onPick: (v: unknown) => void;
  tunnelId: string;
}) {
  const [q, setQ] = useState("");
  const [items, setItems] = useState<ItemDto[]>([]);
  const [vehicles, setVehicles] = useState<VehicleDto[]>([]);
  const [players, setPlayers] = useState<PlayerDto[]>([]);
  const kind: "items" | "vehicles" | "players" =
    field.key === "ItemName" ? "items" : field.key === "ClassName" ? "vehicles" : "players";

  useEffect(() => {
    let cancelled = false;
    const handle = setTimeout(async () => {
      try {
        if (kind === "items") {
          const r = await managementApi.searchItems(tunnelId, q, 20);
          if (!cancelled) setItems(r);
        } else if (kind === "vehicles") {
          const r = await managementApi.searchVehicles(tunnelId, q, 20);
          if (!cancelled) setVehicles(r);
        } else {
          if (!q.trim()) {
            if (!cancelled) setPlayers([]);
            return;
          }
          const r = await managementApi.searchPlayers(tunnelId, q, 20);
          if (!cancelled) setPlayers(r);
        }
      } catch {
        // swallow — lookups are advisory
      }
    }, 250);
    return () => {
      cancelled = true;
      clearTimeout(handle);
    };
  }, [kind, q, tunnelId]);

  return (
    <Box mt="1">
      <TextField.Root
        size="1"
        placeholder={`Search ${kind}…`}
        value={q}
        onChange={(e) => setQ(e.target.value)}
      />
      <Box style={{ maxHeight: 140, overflowY: "auto" }}>
        {kind === "items" &&
          items.map((it) => (
            <div key={it.id}>
              <Button size="1" variant="ghost" onClick={() => onPick(it.id)} style={{ justifyContent: "flex-start" }}>
                <span className="mono">{it.id}</span> — {it.name}
              </Button>
            </div>
          ))}
        {kind === "vehicles" &&
          vehicles.map((v) => (
            <div key={v.id}>
              <Button size="1" variant="ghost" onClick={() => onPick(v.actor_class)} style={{ justifyContent: "flex-start" }}>
                <span className="mono">{v.id}</span>
              </Button>
            </div>
          ))}
        {kind === "players" &&
          players.map((p) => (
            <div key={p.flsId}>
              <Button size="1" variant="ghost" onClick={() => onPick(p.flsId)} style={{ justifyContent: "flex-start" }}>
                <span className="mono">{p.flsId}</span> — {p.name} ({p.online})
              </Button>
            </div>
          ))}
      </Box>
    </Box>
  );
}
