import { useCallback, useEffect, useState } from "react";
import { Badge, Box, Button, Card, Dialog, Flex, Text, TextField } from "@radix-ui/themes";

import type { RemoteServerRecord } from "../../types/server";
import { managementService } from "../../services/management";
import type { ManagementServiceStatus } from "../../types/management";

import ManagementDialog from "./ManagementDialog";

export type ManagementServiceCardProps = {
  server: RemoteServerRecord;
};

type StatusState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ok"; value: ManagementServiceStatus }
  | { kind: "error"; message: string };

export default function ManagementServiceCard({ server }: ManagementServiceCardProps) {
  const [status, setStatus] = useState<StatusState>({ kind: "idle" });
  const [installOpen, setInstallOpen] = useState(false);
  const [installBusy, setInstallBusy] = useState(false);
  const [installError, setInstallError] = useState<string | null>(null);
  const [token, setToken] = useState("");
  const [openManagement, setOpenManagement] = useState(false);

  const refresh = useCallback(async () => {
    setStatus({ kind: "loading" });
    try {
      const result = await managementService.status({
        host: server.host,
        user: server.user,
        keyPath: server.keyPath,
        port: server.port,
      });
      setStatus({ kind: "ok", value: result });
    } catch (err) {
      setStatus({ kind: "error", message: String(err) });
    }
  }, [server.host, server.keyPath, server.port, server.user]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleInstall = useCallback(async () => {
    setInstallBusy(true);
    setInstallError(null);
    try {
      await managementService.install({
        host: server.host,
        user: server.user,
        keyPath: server.keyPath,
        port: server.port,
        commandAuthToken: token.trim() || undefined,
      });
      setInstallOpen(false);
      setToken("");
      await refresh();
    } catch (err) {
      setInstallError(String(err));
    } finally {
      setInstallBusy(false);
    }
  }, [refresh, server, token]);

  const handleUninstall = useCallback(async () => {
    if (!confirm("Uninstall dune-server-service from this host?")) return;
    try {
      await managementService.uninstall({
        host: server.host,
        user: server.user,
        keyPath: server.keyPath,
        port: server.port,
      });
      await refresh();
    } catch (err) {
      alert(`Uninstall failed: ${err}`);
    }
  }, [refresh, server]);

  const installed = status.kind === "ok" ? status.value.installed : false;
  const active = status.kind === "ok" ? status.value.active : false;

  return (
    <Card mt="3">
      <Flex justify="between" align="start" gap="3" wrap="wrap">
        <Box>
          <Text size="3" weight="medium">
            Management service
          </Text>
          <Flex align="center" gap="2" mt="1" wrap="wrap">
            <Badge color={installed ? (active ? "green" : "amber") : "gray"}>
              {status.kind === "loading"
                ? "checking..."
                : installed
                  ? active
                    ? "active"
                    : "installed, not running"
                  : "not installed"}
            </Badge>
            {status.kind === "ok" && status.value.initSystem ? (
              <Badge color="gray" variant="surface">
                {status.value.initSystem}
              </Badge>
            ) : null}
            {status.kind === "error" ? (
              <Text size="1" color="red">
                {status.message}
              </Text>
            ) : null}
          </Flex>
        </Box>
        <Flex gap="2" wrap="wrap">
          <Button size="1" variant="surface" onClick={refresh}>
            Refresh
          </Button>
          <Button size="1" variant="surface" onClick={() => setInstallOpen(true)}>
            {installed ? "Update" : "Install"}
          </Button>
          {installed ? (
            <Button size="1" variant="surface" color="red" onClick={handleUninstall}>
              Uninstall
            </Button>
          ) : null}
          <Button
            size="1"
            variant={active ? "solid" : "soft"}
            disabled={!active}
            onClick={() => setOpenManagement(true)}
          >
            Open Management
          </Button>
        </Flex>
      </Flex>

      {status.kind === "ok" && status.value.journalTail ? (
        <Box mt="3">
          <Text size="1" color="gray">
            Recent journal
          </Text>
          <Box mt="1" p="2" className="mono" style={{ background: "var(--color-panel-translucent)", fontSize: 11, whiteSpace: "pre-wrap" }}>
            {status.value.journalTail}
          </Box>
        </Box>
      ) : null}

      <Dialog.Root open={installOpen} onOpenChange={setInstallOpen}>
        <Dialog.Content maxWidth="500px">
          <Dialog.Title>{installed ? "Update management service" : "Install management service"}</Dialog.Title>
          <Dialog.Description size="2" mb="3" color="gray">
            Uploads the bundled dune-server-service binary + systemd unit to{" "}
            <Text className="mono">/opt/dune-server-service/</Text>, enables the service, and starts it.
            The command-auth token below is written to{" "}
            <Text className="mono">/home/dune/.dune/state/command-auth-token</Text> (mode 0600). Leave blank to keep the
            existing token on the host.
          </Dialog.Description>
          <Flex direction="column" gap="2">
            <Text size="2" weight="medium">
              Command auth token (optional)
            </Text>
            <TextField.Root
              type="password"
              placeholder="paste token or leave blank"
              value={token}
              onChange={(e) => setToken(e.target.value)}
            />
            {installError ? (
              <Text size="1" color="red">
                {installError}
              </Text>
            ) : null}
          </Flex>
          <Flex gap="2" mt="4" justify="end">
            <Dialog.Close>
              <Button variant="soft" color="gray" disabled={installBusy}>
                Cancel
              </Button>
            </Dialog.Close>
            <Button onClick={handleInstall} disabled={installBusy}>
              {installBusy ? "Installing..." : installed ? "Update" : "Install"}
            </Button>
          </Flex>
        </Dialog.Content>
      </Dialog.Root>

      <ManagementDialog
        open={openManagement}
        onOpenChange={setOpenManagement}
        server={server}
      />
    </Card>
  );
}
