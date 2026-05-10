import { describe, expect, it } from "vitest";
import { formatDuration, managerWorkloadsToUi } from "./utils";

describe("utility helpers", () => {
  it("formats compact durations", () => {
    expect(formatDuration(8)).toBe("8s");
    expect(formatDuration(68)).toBe("1m 8s");
    expect(formatDuration(3668)).toBe("1h 1m 8s");
  });

  it("maps manager pod containers for log selection", () => {
    const workloads = managerWorkloadsToUi({
      pods: [
        {
          name: "pod-a",
          phase: "Running",
          ready: true,
          restarts: 1,
          containers: ["main", "sidecar"],
          nodeName: "node",
          createdAt: "now"
        }
      ],
      services: []
    });

    expect(workloads.pods.items?.[0].status?.containers).toEqual(["main", "sidecar"]);
  });
});
