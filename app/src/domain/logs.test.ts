import { describe, expect, it } from "vitest";
import { parseAnsiLine, visibleLogLines } from "./logs";

describe("log helpers", () => {
  it("filters log lines case-insensitively", () => {
    expect(visibleLogLines(["AMQP connected", "warning raised"], "warn")).toEqual(["warning raised"]);
  });

  it("parses stripped 256-color ANSI warning lines", () => {
    const segments = parseAnsiLine("[38;5;214m2026-05-10 [warning] hello[0m");
    expect(segments).toEqual([
      {
        text: "2026-05-10 [warning] hello",
        className: "log-severity-warning",
        style: { color: "#ffaf00" }
      }
    ]);
  });

  it("parses normal escape-prefixed ANSI lines", () => {
    const segments = parseAnsiLine("\u001b[31m[error] bad\u001b[0m");
    expect(segments[0].text).toBe("[error] bad");
    expect(segments[0].className).toBe("log-severity-error");
    expect(segments[0].style?.color).toBe("#ff7675");
  });
});
