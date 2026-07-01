import { Flex } from "@radix-ui/themes";

import type { Update } from "../../services/updater";
import type { RemoteServerRecord, RemoteServerStatus } from "../../types/server";
import type { ActivePage } from "../../types/ui";
import TopNav from "./TopNav";
import UpdateHeaderControl from "./UpdateHeaderControl";

export type HeaderProps = {
  activePage: ActivePage;
  servers: RemoteServerRecord[];
  statuses: Record<string, RemoteServerStatus>;
  statusErrors: Record<string, string>;
  busyMap: Record<string, string>;
  onOpenServersList: () => void;
  onOpenServer: (serverId: string) => void;
  onAddServer: () => void;
  update: Update | null;
  updateProgress: string | null;
  onOpenUpdate: () => void;
};

export default function Header({
  activePage,
  servers,
  statuses,
  statusErrors,
  busyMap,
  onOpenServersList,
  onOpenServer,
  onAddServer,
  update,
  updateProgress,
  onOpenUpdate,
}: HeaderProps) {
  return (
    <Flex asChild align="center" justify="between" px="4" py="3" className="app-header">
      <header>
        <Flex align="center" gap="4">
          <Flex align="center" gap="3">
            <span className="app-glyph" aria-hidden>
              D
            </span>
            <Flex direction="column" gap="0">
              <span className="app-title">Dune Dedicated Server Manager</span>
              <span className="app-title-sub">Operator console</span>
            </Flex>
          </Flex>

          <TopNav
            activePage={activePage}
            servers={servers}
            statuses={statuses}
            statusErrors={statusErrors}
            busyMap={busyMap}
            onOpenServersList={onOpenServersList}
            onOpenServer={onOpenServer}
            onAddServer={onAddServer}
          />
        </Flex>

        <UpdateHeaderControl
          update={update}
          progress={updateProgress}
          onOpenUpdate={onOpenUpdate}
        />
      </header>
    </Flex>
  );
}
