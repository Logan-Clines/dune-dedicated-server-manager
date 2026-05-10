import type { BattleGroupDetail } from "./battlegroup";

export type BattleGroupLifecycleAction = "start" | "stop" | "restart";
export type BattleGroupLifecycleTarget = "running" | "stopped";

export function isBattleGroupSettled(detail: BattleGroupDetail, target: BattleGroupLifecycleTarget) {
  const phase = detail.phase.toLowerCase();
  if (target === "stopped") {
    return detail.stop || ["stopped", "suspended"].includes(phase);
  }
  return !detail.stop && ["healthy", "running", "ready"].includes(phase);
}

export function lifecycleStatusText(action: BattleGroupLifecycleAction, detail: BattleGroupDetail) {
  const phase = detail.phase || "Unknown";
  if (action === "restart") return `Restarting, current phase ${phase}`;
  if (action === "start") return `Starting, current phase ${phase}`;
  return `Stopping, current phase ${phase}`;
}
