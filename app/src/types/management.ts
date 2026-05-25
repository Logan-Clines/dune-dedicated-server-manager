export type ManagementInstallRequest = {
  host: string;
  user: string;
  keyPath?: string;
  port?: number;
  commandAuthToken?: string;
};

export type ManagementConnRequest = {
  host: string;
  user: string;
  keyPath?: string;
  port?: number;
};

export type ManagementInstallResult = {
  installed: boolean;
  started: boolean;
  initSystem: string;
  message: string;
};

export type ManagementServiceStatus = {
  installed: boolean;
  active: boolean;
  initSystem: string;
  journalTail: string;
};

export type HealthDto = {
  ok: boolean;
  version: string;
  now: string;
};

export type RunDto = {
  id: number;
  taskId: string;
  trigger: "scheduled" | "manual" | "startup";
  dryRun: boolean;
  status: "running" | "success" | "failed" | "skipped";
  startedAt: string;
  finishedAt: string | null;
  durationMs: number | null;
  error: string | null;
};

export type LogDto = {
  id: number;
  createdAt: string;
  level: "info" | "warn" | "error";
  message: string;
  taskId: string | null;
  runId: number | null;
};

export type FieldKind = "string" | "int" | "float" | "bool" | "select" | "text";

export type SelectOption = {
  value: string;
  label: string;
};

export type FieldSpec = {
  key: string;
  label: string;
  kind: FieldKind;
  required?: boolean;
  default?: unknown;
  helper?: string;
  options?: SelectOption[];
};

export type Category =
  | "items"
  | "movement"
  | "broadcast"
  | "progression"
  | "player"
  | "journey"
  | "exec";

export type CommandSpec = {
  id: string;
  label: string;
  category: Category;
  destructive?: boolean;
  needsPlayer: boolean;
  allowAllPlayers: boolean;
  describe: string;
  fields: FieldSpec[];
};

export type ItemDto = {
  id: string;
  name: string;
  category: string;
  source: string;
};

export type VehicleDto = {
  id: string;
  actor_class: string;
  templates: string[];
};

export type PlayerDto = {
  flsId: string;
  name: string;
  online: string;
  lastSeen: string;
};

export type ClusterDto = {
  namespace: string;
  mqPod: string;
  dbPod: string | null;
  serviceVersion: string;
};

export type HistoryDto = {
  id: number;
  createdAt: string;
  command: string;
  payload: Record<string, unknown>;
  ok: boolean;
  message: string | null;
};

export type PublishResultDto = {
  ok: boolean;
  command: string;
  output: string;
  error: string | null;
  inner: Record<string, unknown>;
};
