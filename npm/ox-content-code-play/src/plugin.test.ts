import { describe, expect, it } from "vite-plus/test";
import { codePlay } from "./plugin";

describe("codePlay vite plugin", () => {
  it("is inert until languages are enabled and rewrites only those fences", () => {
    const plugin = codePlay({ languages: { typescript: true } });
    expect(plugin.name).toBe("@ox-content/code-play");
    const transform = hookFn(plugin.transform) as (code: string, id: string) => string | null;
    expect(transform("```ts play\nconst n = 1;\n```\n", "page.md")).toContain("<!--ox-code-play:");
    expect(transform("```python play\nprint(1)\n```\n", "page.md")).toBeNull();
  });

  it("exposes a virtual client module id", () => {
    const plugin = codePlay();
    const resolveId = hookFn(plugin.resolveId) as (id: string) => string | null;
    expect(resolveId("virtual:ox-content/code-play")).toBe("\0virtual:ox-content/code-play");
  });
});

function hookFn(hook: unknown): (...args: never[]) => unknown {
  if (typeof hook === "function") {
    return hook as (...args: never[]) => unknown;
  }
  if (hook && typeof hook === "object" && "handler" in hook && typeof hook.handler === "function") {
    return hook.handler as (...args: never[]) => unknown;
  }
  throw new Error("expected a plugin hook");
}
