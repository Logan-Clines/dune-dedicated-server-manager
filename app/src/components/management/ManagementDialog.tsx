import { useEffect, useState } from "react";
import { Dialog, Flex, Tabs, Text } from "@radix-ui/themes";
import { invoke } from "@tauri-apps/api/core";

import type { RemoteServerRecord } from "../../types/server";
import type { ServerTunnelStatus } from "../../types/tunnel";
import { serverTunnelKey } from "../../utils/remote-server";

import RunsTab from "./RunsTab";
import AdminTab from "./AdminTab";

export type ManagementDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  server: RemoteServerRecord;
};

export default function ManagementDialog({ open, onOpenChange, server }: ManagementDialogProps) {
  const [tunnel, setTunnel] = useState<ServerTunnelStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const tunnelId = serverTunnelKey(server.id, "managementApi");

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setError(null);
    invoke<ServerTunnelStatus>("start_server_tunnel", {
      request: {
        tunnelId,
        serverKind: server.type,
        service: "managementApi",
        host: server.host,
        user: server.user,
        keyPath: server.keyPath,
        port: server.port,
        namespace: server.namespace || "",
      },
    })
      .then((status) => {
        if (!cancelled) setTunnel(status);
      })
      .catch((err) => {
        if (!cancelled) setError(String(err));
      });
    return () => {
      cancelled = true;
    };
  }, [open, tunnelId, server.host, server.keyPath, server.namespace, server.port, server.type, server.user]);

  useEffect(() => {
    if (open || !tunnel) return;
    invoke("stop_server_tunnel", { request: { tunnelId } }).catch(() => {});
  }, [open, tunnel, tunnelId]);

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content maxWidth="1100px" style={{ minHeight: "70vh" }}>
        <Dialog.Title>Management — {server.name || server.host}</Dialog.Title>
        <Dialog.Description size="2" color="gray">
          Tunneled through SSH to <Text className="mono">127.0.0.1:8787</Text> on the remote host.
        </Dialog.Description>

        {error ? (
          <Flex mt="3" direction="column" gap="2">
            <Text color="red">Could not open tunnel: {error}</Text>
          </Flex>
        ) : !tunnel ? (
          <Flex mt="4" justify="center">
            <Text color="gray">Connecting…</Text>
          </Flex>
        ) : (
          <Tabs.Root defaultValue="runs">
            <Tabs.List>
              <Tabs.Trigger value="runs">Runs &amp; Logs</Tabs.Trigger>
              <Tabs.Trigger value="admin">Admin Commands</Tabs.Trigger>
            </Tabs.List>
            <Tabs.Content value="runs">
              <RunsTab tunnelId={tunnelId} />
            </Tabs.Content>
            <Tabs.Content value="admin">
              <AdminTab tunnelId={tunnelId} />
            </Tabs.Content>
          </Tabs.Root>
        )}
      </Dialog.Content>
    </Dialog.Root>
  );
}
