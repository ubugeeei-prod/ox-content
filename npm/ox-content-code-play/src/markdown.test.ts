import { describe, expect, it } from "vite-plus/test";
import { enhanceGeneratedModule, enhancePlayHtml } from "./html";
import { parseCodePlayTags, parsePlayFences, rewritePlayFences, stripPlayMeta } from "./markdown";
import { decodePayload, encodePayload } from "./payload";
import { payloadFromFence } from "./payload-factory";
import { DEFAULT_VIEWERS, resolveCodePlayOptions } from "./config";
import {
  renderConfigHtml,
  renderProvenanceHtml,
  renderStdioHtml,
  renderTimingHtml,
} from "./viewers";
import { renderPlayUi } from "./ui";

describe("markdown and viewers", () => {
  it("parses only top-level play fences and leaves nested examples alone", () => {
    const source = [
      "````md",
      "```ts play",
      "const nested = 1;",
      "```",
      "````",
      "",
      '```ts play typecheck annotate="highlight:1"',
      "const live = 1;",
      "```",
    ].join("\n");
    const fences = parsePlayFences(source);
    expect(fences).toHaveLength(1);
    expect(fences[0]?.code).toBe("const live = 1;");
    expect(fences[0]?.typecheck).toBe(true);
    expect(stripPlayMeta(fences[0]?.meta ?? "")).toBe('annotate="highlight:1"');
  });

  it("parses authoring options without preserving play-only metadata", () => {
    const source = [
      '```ts play typecheck play-title="Strict sample" play-compact play-timeout=2500 play-strict=false play-target=ESNext play-viewers=stdio,stderr,-timing annotate="highlight:1"',
      "const live = 1;",
      "```",
    ].join("\n");
    const fence = parsePlayFences(source)[0];
    expect(fence?.title).toBe("Strict sample");
    expect(fence?.ui).toBe("compact");
    expect(fence?.timeoutMs).toBe(2500);
    expect(fence?.config).toEqual({ strict: false, target: "ESNext" });
    expect(fence?.viewers).toEqual({ stdio: true, stderr: true, timing: false });
    expect(stripPlayMeta(fence?.meta ?? "")).toBe('annotate="highlight:1"');
  });

  it("rewrites play fences to comments and parses CodePlay tags", () => {
    const options = resolveCodePlayOptions({ languages: { typescript: true } });
    const rewritten = rewritePlayFences("```ts play\nconst n = 1;\n```", (fence) =>
      encodePayload(payloadFromFence(fence, options)),
    );
    expect(rewritten).toContain("<!--ox-code-play:");
    expect(rewritten).toContain("```ts\nconst n = 1;\n```");

    const tags = parseCodePlayTags(`<CodePlay lang="python" typecheck>\nprint(1)\n</CodePlay>`);
    expect(tags[0]).toMatchObject({
      language: "python",
      typecheck: true,
      code: "print(1)",
      config: {},
    });
    const configuredTags = parseCodePlayTags(
      `<CodePlay lang="ts" title="Loose TS" ui="compact" timeout="3000" config-strict="false" config-with-vet="false">\nconst n = 1\n</CodePlay>`,
    );
    expect(configuredTags[0]).toMatchObject({
      title: "Loose TS",
      ui: "compact",
      timeoutMs: 3000,
      config: { strict: false, withVet: false },
    });
  });

  it("wraps commented pre blocks and matching SSG fences in HTML", () => {
    const options = resolveCodePlayOptions({ languages: { javascript: true } });
    const payload = encodePayload(
      payloadFromFence(
        {
          language: "js",
          meta: "play",
          code: "console.log(1)",
          raw: "",
          start: 0,
          end: 0,
          typecheck: false,
          config: {},
        },
        options,
      ),
    );
    const commented = enhancePlayHtml(
      `<!--ox-code-play:${payload}-->\n<pre><code class="language-js">console.log(1)</code></pre>`,
      {
        decodePayload,
        encodePayload,
        scriptSrc: "/ox-code-play.js",
      },
    );
    expect(commented).toContain("<ox-code-play data-ox-code-play=");
    expect(commented).toContain(" inert>");
    expect(commented).toContain("/ox-code-play.js");

    const matched = enhancePlayHtml(`<pre><code class="language-js">console.log(1)</code></pre>`, {
      decodePayload,
      encodePayload,
      matchFences: [{ language: "js", code: "console.log(1)", payload }],
    });
    expect(matched).toContain("data-ox-code-play");

    const rustPayload = encodePayload(
      payloadFromFence(
        {
          language: "rust",
          meta: "play",
          code: "let v: Vec<u8> = vec![];",
          raw: "",
          start: 0,
          end: 0,
          typecheck: false,
          config: {},
        },
        resolveCodePlayOptions({ languages: { rust: true } }),
      ),
    );
    const numeric = enhancePlayHtml(
      `<pre><code class="language-rust">let v: Vec&#x3C;u8> = vec![];</code></pre>`,
      {
        decodePayload,
        encodePayload,
        matchFences: [{ language: "rust", code: "let v: Vec<u8> = vec![];", payload: rustPayload }],
      },
    );
    expect(numeric).toContain("data-ox-code-play");

    const moduleSource = `export const html = ${JSON.stringify(`<!--ox-code-play:${payload}--><pre><code>x</code></pre>`)};`;
    expect(enhanceGeneratedModule(moduleSource, { decodePayload, encodePayload })).toContain(
      "ox-code-play",
    );
  });

  it("does not advertise TypeScript typecheck on a CodePlay tag without an endpoint", () => {
    const html = enhancePlayHtml(`<CodePlay lang="ts" typecheck>\nconst n = 1;\n</CodePlay>`, {
      decodePayload,
      encodePayload,
    });
    const encoded = /data-ox-code-play="([^"]+)"/.exec(html)?.[1];
    expect(encoded).toBeTruthy();
    const payload = decodePayload(encoded ?? "");
    expect(payload.capabilities.typecheck).toBe(false);
    expect(payload.endpoints?.typecheck).toBeUndefined();
  });

  it("renders config, stdio, provenance, and timing viewers", () => {
    expect(
      renderStdioHtml([
        { stream: "stdout", text: "ok\n", timestampMs: 1.2 },
        { stream: "stderr", text: "bad\n", timestampMs: 3 },
      ]),
    ).toContain("stdout +1.2ms");
    expect(
      renderConfigHtml([{ key: "strict", label: "Strict", type: "boolean", default: true }], {
        strict: true,
      }),
    ).toContain('name="strict"');
    expect(
      renderProvenanceHtml({
        compile: { host: "play.rust-lang.org", runtime: "rustc", version: "stable" },
        execute: { host: "play.rust-lang.org", runtime: "rust-playground", sandbox: "playground" },
      }),
    ).toMatch(/Compiled[\s\S]*play\.rust-lang\.org/);
    expect(
      renderTimingHtml({
        totalMs: 10,
        phases: [{ id: "compile", label: "Compile", startMs: 0, durationMs: 4 }],
      }),
    ).toContain("Total 10.0ms");
    expect(
      renderPlayUi({
        payload: {
          language: "typescript",
          code: "const n = 1;",
          title: "Strict TS",
          capabilities: { execute: true, typecheck: true },
          config: { strict: true },
          viewers: { ...DEFAULT_VIEWERS },
          ui: "default",
          timeoutMs: 1000,
        },
      }),
    ).toContain("Typecheck");
    expect(
      renderPlayUi({
        payload: {
          language: "typescript",
          code: "const n = 1;",
          title: "Strict TS",
          capabilities: { execute: true, typecheck: true },
          config: { strict: true },
          viewers: { ...DEFAULT_VIEWERS },
          ui: "default",
          timeoutMs: 1000,
        },
      }),
    ).toContain('role="region"');
  });

  it("decodes the same payload it encodes", () => {
    const payload = {
      language: "go",
      code: "package main",
      capabilities: { execute: true, typecheck: true },
      config: { withVet: true },
      viewers: { ...DEFAULT_VIEWERS },
      ui: "compact" as const,
      timeoutMs: 5,
    };
    expect(decodePayload(encodePayload(payload))).toEqual(payload);
  });

  it("fills a missing stderr viewer flag when decoding older payloads", () => {
    const encoded = Buffer.from(
      JSON.stringify({
        language: "javascript",
        code: "1",
        capabilities: { execute: true, typecheck: false },
        config: {},
        viewers: { config: true, stdio: true, provenance: true, timing: true },
        ui: "default",
        timeoutMs: 1,
      }),
    ).toString("base64");
    expect(decodePayload(encoded).viewers.stderr).toBe(true);
    expect(decodePayload(encoded).viewers.stdio).toBe(true);
  });
});
