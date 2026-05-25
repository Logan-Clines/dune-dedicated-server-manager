import { invoke } from "@tauri-apps/api/core";

import type {
  ClusterDto,
  CommandSpec,
  HealthDto,
  HistoryDto,
  ItemDto,
  LogDto,
  ManagementConnRequest,
  ManagementInstallRequest,
  ManagementInstallResult,
  ManagementServiceStatus,
  PlayerDto,
  PublishResultDto,
  RunDto,
  VehicleDto,
} from "../types/management";

export const managementService = {
  install: (req: ManagementInstallRequest) =>
    invoke<ManagementInstallResult>("install_management_service", { request: req }),
  uninstall: (req: ManagementConnRequest) =>
    invoke<void>("uninstall_management_service", { request: req }),
  status: (req: ManagementConnRequest) =>
    invoke<ManagementServiceStatus>("management_service_status", { request: req }),
};

export const managementApi = {
  health: (tunnelId: string) => invoke<HealthDto>("ms_health", { tunnelId }),
  listRuns: (tunnelId: string, limit?: number, task?: string) =>
    invoke<RunDto[]>("ms_list_runs", { tunnelId, limit, task }),
  listLogs: (tunnelId: string, limit?: number, runId?: number) =>
    invoke<LogDto[]>("ms_list_logs", { tunnelId, limit, runId }),
  triggerRun: (tunnelId: string, task: string) =>
    invoke<{ ok: boolean; task: string }>("ms_trigger_run", { tunnelId, task }),
  listCommands: (tunnelId: string) =>
    invoke<CommandSpec[]>("ms_list_commands", { tunnelId }),
  searchItems: (tunnelId: string, q: string, limit?: number) =>
    invoke<ItemDto[]>("ms_search_items", { tunnelId, q, limit }),
  searchVehicles: (tunnelId: string, q: string, limit?: number) =>
    invoke<VehicleDto[]>("ms_search_vehicles", { tunnelId, q, limit }),
  searchPlayers: (tunnelId: string, q: string, limit?: number) =>
    invoke<PlayerDto[]>("ms_search_players", { tunnelId, q, limit }),
  cluster: (tunnelId: string) => invoke<ClusterDto>("ms_cluster", { tunnelId }),
  history: (tunnelId: string, limit?: number) =>
    invoke<HistoryDto[]>("ms_history", { tunnelId, limit }),
  publish: (tunnelId: string, command: string, fields: Record<string, unknown>) =>
    invoke<PublishResultDto>("ms_publish", { tunnelId, command, fields }),
};
