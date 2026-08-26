import { describe, expect, it } from "vite-plus/test";
import { DEV_TYPECHECK_PATH } from "./config";
import { decodePayload } from "./payload";
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

  it("does not embed the Vite typecheck proxy into SSG payloads", () => {
    const plugin = codePlay({ languages: { typescript: { execute: true, typecheck: true } } });
    resolveCommand(plugin, "build");
    const payload = payloadFromTransform(plugin, "```ts play\nconst n = 1;\n```\n");
    expect(payload.capabilities.typecheck).toBe(false);
    expect(payload.endpoints?.typecheck).toBeUndefined();
  });

  it("keeps an explicit typecheck endpoint in SSG payloads", () => {
    const plugin = codePlay({
      languages: { typescript: { execute: true, typecheck: true } },
      endpoints: { typecheck: "https://example.test/typecheck" },
    });
    resolveCommand(plugin, "build");
    const payload = payloadFromTransform(plugin, "```ts play\nconst n = 1;\n```\n");
    expect(payload.capabilities.typecheck).toBe(true);
    expect(payload.endpoints?.typecheck).toBe("https://example.test/typecheck");
  });

  it("embeds remote language endpoints for hydrated Python runs", () => {
    const plugin = codePlay({
      languages: { python: { endpoint: "https://exec.example/api/v2/piston" } },
    });
    resolveCommand(plugin, "build");
    const payload = payloadFromTransform(plugin, "```python play\nprint(1)\n```\n");
    expect(payload.endpoint).toBe("https://exec.example/api/v2/piston");
    expect(payload.capabilities.execute).toBe(true);
  });

  it("embeds per-sample authoring options in payloads", () => {
    const plugin = codePlay({ languages: { typescript: true } });
    resolveCommand(plugin, "serve");
    const payload = payloadFromTransform(
      plugin,
      '```ts play play-title="Loose sample" play-compact play-timeout=2500 play-strict=false\nconst n = 1;\n```',
    );
    expect(payload.title).toBe("Loose sample");
    expect(payload.ui).toBe("compact");
    expect(payload.timeoutMs).toBe(2500);
    expect(payload.config.strict).toBe(false);
  });

  it("does not request a browser client asset until a play fence is transformed", async () => {
    const plugin = codePlay({ languages: { javascript: true } });
    resolveCommand(plugin, "build");
    const emitted: unknown[] = [];
    const context = { emitFile: (file: unknown) => emitted.push(file) };
    await hookFn(plugin.generateBundle).call(context);
    expect(emitted).toEqual([]);
    const transform = hookFn(plugin.transform) as (source: string, id: string) => string | null;
    expect(transform("```js\nconsole.log(1);\n```", "plain.md")).toBeNull();
    await hookFn(plugin.generateBundle).call(context);
    expect(emitted).toEqual([]);
  });

  it("keeps the Vite typecheck proxy on the dev server", () => {
    const plugin = codePlay({ languages: { typescript: { execute: true, typecheck: true } } });
    resolveCommand(plugin, "serve");
    const payload = payloadFromTransform(plugin, "```ts play\nconst n = 1;\n```\n");
    expect(payload.capabilities.typecheck).toBe(true);
    expect(payload.endpoints?.typecheck).toBe(DEV_TYPECHECK_PATH);
  });

  it("does not embed the Vite typecheck proxy when proxy is disabled", () => {
    const plugin = codePlay({
      languages: { typescript: { execute: true, typecheck: true } },
      proxy: false,
    });
    resolveCommand(plugin, "serve");
    const payload = payloadFromTransform(plugin, "```ts play\nconst n = 1;\n```\n");
    expect(payload.capabilities.typecheck).toBe(false);
    expect(payload.endpoints?.typecheck).toBeUndefined();
  });
});

function resolveCommand(plugin: { configResolved?: unknown }, command: "build" | "serve"): void {
  hookFn(plugin.configResolved)({
    command,
    root: "/",
    base: "/",
    build: { outDir: "dist" },
  } as never);
}

function payloadFromTransform(plugin: { transform?: unknown }, code: string) {
  const rewritten = (hookFn(plugin.transform) as (source: string, id: string) => string | null)(
    code,
    "page.md",
  );
  const match =
    typeof rewritten === "string" ? /<!--ox-code-play:([A-Za-z0-9+/=]+)-->/.exec(rewritten) : null;
  if (!match?.[1]) {
    throw new Error("expected a Code Play payload comment");
  }
  return decodePayload(match[1]);
}

function hookFn(hook: unknown): (...args: never[]) => unknown {
  if (typeof hook === "function") {
    return hook as (...args: never[]) => unknown;
  }
  if (hook && typeof hook === "object" && "handler" in hook && typeof hook.handler === "function") {
    return hook.handler as (...args: never[]) => unknown;
  }
  throw new Error("expected a plugin hook");
}
