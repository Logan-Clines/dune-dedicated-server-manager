import { describe, expect, it } from "vitest";
import type { BattleGroupDetail } from "./battlegroup";
import { isBattleGroupSettled, lifecycleStatusText } from "./lifecycle";

function detail(overrides: Partial<BattleGroupDetail>): BattleGroupDetail {
  return {
    namespace: "ns",
    name: "bg",
    title: "bg",
    phase: "Unknown",
    stop: false,
    databasePhase: "",
    serverGroupPhase: "",
    gatewayPhase: "",
    directorPhase: "",
    serverImage: "",
    utilityImages: [],
    serverSets: [],
    ...overrides
  };
}

describe("battlegroup lifecycle helpers", () => {
  it("treats healthy running details as settled", () => {
    expect(isBattleGroupSettled(detail({ phase: "Healthy", stop: false }), "running")).toBe(true);
  });

  it("treats stopped details as settled", () => {
    expect(isBattleGroupSettled(detail({ phase: "Stopped", stop: true }), "stopped")).toBe(true);
  });

  it("reports action-specific status text", () => {
    expect(lifecycleStatusText("restart", detail({ phase: "Starting" }))).toBe("Restarting, current phase Starting");
  });
});
