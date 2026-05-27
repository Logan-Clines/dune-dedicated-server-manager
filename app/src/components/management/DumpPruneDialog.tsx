import { useCallback, useEffect, useState } from "react";
import {
  Badge,
  Box,
  Button,
  Checkbox,
  Dialog,
  Flex,
  Separator,
  Text,
} from "@radix-ui/themes";

import { managementApi } from "../../services/management";
import type { DumpPruneItem, DumpPruneResult } from "../../types/management";

type LoadState =
  | { status: "loading" }
  | { status: "error"; message: string }
  | { status: "confirming"; items: DumpPruneItem[]; selected: Set<string> }
  | { status: "deleting"; items: DumpPruneItem[]; selected: Set<string> }
  | { status: "done"; result: DumpPruneResult };

function itemKey(item: { namespace: string; name: string }): string {
  return `${item.namespace}/${item.name}`;
}

export type DumpPruneDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  tunnelId: string;
};

export default function DumpPruneDialog({
  open,
  onOpenChange,
  tunnelId,
}: DumpPruneDialogProps) {
  const [state, setState] = useState<LoadState>({ status: "loading" });

  const load = useCallback(async () => {
    setState({ status: "loading" });
    try {
      const items = await managementApi.dumpPrunePreview(tunnelId);
      const selected = new Set(items.map(itemKey));
      // Drop straight into confirming — every row pre-selected; the operator
      // ticks off anything they want to keep before clicking Delete.
      setState({ status: "confirming", items, selected });
    } catch (err) {
      setState({ status: "error", message: String(err) });
    }
  }, [tunnelId]);

  useEffect(() => {
    if (open) void load();
  }, [open, load]);

  const toggle = (key: string) => {
    setState((prev) => {
      if (prev.status !== "confirming") return prev;
      const next = new Set(prev.selected);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return { ...prev, selected: next };
    });
  };

  const selectAll = (checked: boolean) => {
    setState((prev) => {
      if (prev.status !== "confirming") return prev;
      const next = checked ? new Set(prev.items.map(itemKey)) : new Set<string>();
      return { ...prev, selected: next };
    });
  };

  const runDelete = async () => {
    if (state.status !== "confirming") return;
    const targets = state.items
      .filter((item) => state.selected.has(itemKey(item)))
      .map((item) => ({ namespace: item.namespace, name: item.name }));
    if (targets.length === 0) return;
    setState({ status: "deleting", items: state.items, selected: state.selected });
    try {
      const result = await managementApi.dumpPruneExecute(tunnelId, targets);
      setState({ status: "done", result });
    } catch (err) {
      setState({ status: "error", message: String(err) });
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content maxWidth="720px">
        <Dialog.Title>Clean up database operations</Dialog.Title>
        <Dialog.Description size="2" color="gray" mb="3">
          Removes terminal <Text className="mono">DatabaseOperation</Text> resources from the
          cluster — both <Badge color="green">Succeeded</Badge> (artifact on disk, CR is just
          bookkeeping) and <Badge color="red">Failed</Badge> (no artifact produced, pure
          clutter). Covers both <Text className="mono">dump</Text> and{" "}
          <Text className="mono">import</Text> actions. The pod attached to each is
          garbage-collected by Funcom&apos;s operator. The{" "}
          <Text className="mono">.backup</Text> file on disk is{" "}
          <Text weight="bold">never</Text> touched. In-progress operations are not listed.
        </Dialog.Description>

        <Separator size="4" my="2" />

        <Body state={state} onToggle={toggle} onSelectAll={selectAll} />

        <Flex gap="2" mt="4" justify="end">
          <Dialog.Close>
            <Button variant="soft" color="gray">
              {state.status === "done" ? "Close" : "Cancel"}
            </Button>
          </Dialog.Close>
          {state.status === "confirming" ? (
            <Button
              color="red"
              disabled={state.selected.size === 0}
              onClick={runDelete}
            >
              Delete {state.selected.size} selected
            </Button>
          ) : null}
          {state.status === "done" ? (
            <Button onClick={() => void load()}>Refresh</Button>
          ) : null}
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

function Body({
  state,
  onToggle,
  onSelectAll,
}: {
  state: LoadState;
  onToggle: (key: string) => void;
  onSelectAll: (checked: boolean) => void;
}) {
  if (state.status === "loading") {
    return (
      <Text size="2" color="gray">
        Listing eligible dump operations…
      </Text>
    );
  }
  if (state.status === "error") {
    return (
      <Text size="2" color="red">
        {state.message}
      </Text>
    );
  }
  if (state.status === "deleting") {
    return (
      <Text size="2" color="gray">
        Deleting {state.selected.size} operation(s)…
      </Text>
    );
  }
  if (state.status === "done") {
    return (
      <Box>
        <Text size="2" weight="medium">
          Deleted: {state.result.deleted.length}
        </Text>
        {state.result.deleted.length > 0 ? (
          <Box mt="1" mb="2">
            {state.result.deleted.map((name) => (
              <Text key={name} size="1" className="mono" color="green" as="div">
                {name}
              </Text>
            ))}
          </Box>
        ) : null}
        {state.result.skipped.length > 0 ? (
          <Box mt="2">
            <Text size="2" weight="medium" color="amber">
              Skipped: {state.result.skipped.length}
            </Text>
            {state.result.skipped.map((row) => (
              <Text
                key={itemKey(row)}
                size="1"
                className="mono"
                color="amber"
                as="div"
              >
                {row.namespace}/{row.name} — {row.reason}
              </Text>
            ))}
          </Box>
        ) : null}
      </Box>
    );
  }
  // confirming
  if (state.items.length === 0) {
    return (
      <Text size="2" color="gray">
        Nothing to clean up — no terminal dump operations found.
      </Text>
    );
  }
  const allSelected = state.selected.size === state.items.length;
  const succeeded = state.items.filter((i) => i.phase === "Succeeded").length;
  const failed = state.items.filter((i) => i.phase === "Failed").length;
  return (
    <Box>
      <Flex align="center" gap="3" mb="2" wrap="wrap">
        <Checkbox
          checked={allSelected}
          onCheckedChange={(checked) => onSelectAll(Boolean(checked))}
        />
        <Text size="2">
          {allSelected ? "Deselect all" : "Select all"} ({state.items.length} total)
        </Text>
        <Flex gap="2" ml="auto">
          <Badge color="green">{succeeded} succeeded</Badge>
          <Badge color="red">{failed} failed</Badge>
        </Flex>
      </Flex>
      <Box className="dump-prune-list">
        {state.items.map((item) => {
          const key = itemKey(item);
          const checked = state.selected.has(key);
          const badgeColor: "green" | "red" | "gray" =
            item.phase === "Succeeded" ? "green" : item.phase === "Failed" ? "red" : "gray";
          return (
            <Flex key={key} gap="2" align="center" py="1">
              <Checkbox checked={checked} onCheckedChange={() => onToggle(key)} />
              <Badge color={badgeColor}>{item.phase}</Badge>
              <Box style={{ flex: 1, minWidth: 0 }}>
                <Text size="2" className="mono" as="div">
                  {item.name}
                </Text>
                <Text size="1" color="gray" as="div">
                  ns={item.namespace} · action={item.action} · age={item.ageDays}d ·{" "}
                  {item.backup ? `backup=${item.backup}` : "no backup recorded"}
                </Text>
              </Box>
            </Flex>
          );
        })}
      </Box>
    </Box>
  );
}
