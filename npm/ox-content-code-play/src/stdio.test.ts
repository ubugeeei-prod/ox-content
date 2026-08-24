import { describe, expect, it } from "vite-plus/test";
import { joinStream, selectStream, StdioBuffer, withStdioText } from "./stdio";
import type { StdioEvent } from "./types";

const events: StdioEvent[] = [
  { stream: "stdout", text: "a", timestampMs: 1 },
  { stream: "stderr", text: "e1", timestampMs: 2 },
  { stream: "stdout", text: "b", timestampMs: 3 },
  { stream: "stderr", text: "e2", timestampMs: 4 },
  { stream: "stdin", text: "in", timestampMs: 5 },
];

describe("stdio helpers", () => {
  it("joins stdout and stderr in stream order without mixing the other stream", () => {
    expect(joinStream(events, "stdout")).toBe("ab");
    expect(joinStream(events, "stderr")).toBe("e1e2");
    expect(joinStream(events, "stdin")).toBe("in");
    expect(joinStream([], "stdout")).toBe("");
  });

  it("selects only the requested stream and preserves timestamps", () => {
    expect(selectStream(events, "stderr")).toEqual([
      { stream: "stderr", text: "e1", timestampMs: 2 },
      { stream: "stderr", text: "e2", timestampMs: 4 },
    ]);
    expect(selectStream(events, "stdout")).toHaveLength(2);
  });

  it("attaches exact stdout and stderr strings without mutating the original result", () => {
    const result = {
      status: "ok" as const,
      stdio: events,
      extra: true,
    };
    const next = withStdioText(result);
    expect(next.stdout).toBe("ab");
    expect(next.stderr).toBe("e1e2");
    expect(next.extra).toBe(true);
    expect(result).not.toHaveProperty("stdout");
    expect(result).not.toHaveProperty("stderr");
  });

  it("records buffer events with stream identity", () => {
    const buffer = new StdioBuffer(0);
    buffer.push("stdout", "out\n", 1);
    buffer.push("stderr", "err\n", 2);
    expect(buffer.snapshot()).toEqual([
      { stream: "stdout", text: "out\n", timestampMs: 1 },
      { stream: "stderr", text: "err\n", timestampMs: 2 },
    ]);
  });
});
