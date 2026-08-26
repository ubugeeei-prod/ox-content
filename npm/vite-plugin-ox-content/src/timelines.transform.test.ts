import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

describe("timeline transform", () => {
  it("leaves timeline source literal unless opted in", async () => {
    const markdown = [
      '::: timeline title="Release history"',
      '- 2026-08-26 RC cut {status=done label="RC" href="/releases/rc"}',
      "  Shipped **candidate** builds.",
      "- [2026-09] GA window {status=planned}",
      "- Follow-up hardening {label=Next}",
      ":::",
      "",
    ].join("\n");

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/releases.md",
      createResolvedOptions(),
    );
    expect(defaultResult.html).not.toContain("ox-timeline");
    expect(defaultResult.html).toContain("timeline");

    const enabledResult = await transformMarkdown(
      markdown,
      "docs/releases.md",
      createResolvedOptions({
        timelines: {
          enabled: true,
          ordered: true,
          invalidDate: "error",
          unknownMeta: "error",
          empty: "error",
        },
      }),
    );
    expect(enabledResult.html).toContain('<section class="ox-timeline" aria-label="Timeline">');
    expect(enabledResult.html).toContain('<ol class="ox-timeline__items">');
    expect(enabledResult.html).toContain(
      '<time class="ox-timeline__date" datetime="2026-08-26">2026-08-26</time>',
    );
    expect(enabledResult.html).toContain('data-status="done"');
    expect(enabledResult.html).toContain('<span class="ox-timeline__label">RC</span>');
    expect(enabledResult.html).toContain('<span class="ox-timeline__label">Next</span>');
    expect(enabledResult.html).toContain('<a href="/releases/rc">RC cut</a>');
    expect(enabledResult.html).toContain("<strong>candidate</strong>");
    expect(enabledResult.html).not.toContain("<script");
  });

  it("reports malformed dates without rewriting the block", async () => {
    const { result, warnings } = await captureTransformWarnings(() =>
      transformMarkdown(
        "::: timeline\n- 2026-02-31 Impossible\n:::\n",
        "docs/releases.md",
        createResolvedOptions({
          timelines: {
            enabled: true,
            ordered: true,
            invalidDate: "error",
            unknownMeta: "error",
            empty: "error",
          },
        }),
      ),
    );
    expect(flattenWarnings(warnings)).toContain("invalid date");
    expect(result.html).not.toContain("ox-timeline");
    expect(result.html).toContain("timeline");
  });

  it("can warn for malformed dates and still render without datetime", async () => {
    const { result, warnings } = await captureTransformWarnings(() =>
      transformMarkdown(
        "::: timeline\n- 2026-02-31 Impossible\n:::\n",
        "docs/releases.md",
        createResolvedOptions({
          timelines: {
            enabled: true,
            ordered: true,
            invalidDate: "warn",
            unknownMeta: "error",
            empty: "error",
          },
        }),
      ),
    );
    expect(flattenWarnings(warnings)).toContain("invalid date");
    expect(result.html).toContain(
      '<span class="ox-timeline__date ox-timeline__date--invalid">2026-02-31</span>',
    );
  });

  it("keeps nested markdown, containers, and code-fence markers in item bodies", async () => {
    const result = await transformMarkdown(
      [
        "::: timeline",
        "- 2026-08-26 Authoring polish",
        "  ::: tip",
        "  Nested **container** copy.",
        "  :::",
        "  ",
        "  ```md",
        "  ::: timeline",
        "  ```",
        ":::",
        "",
      ].join("\n"),
      "docs/releases.md",
      createResolvedOptions({
        containers: { enabled: true, types: {} },
        timelines: {
          enabled: true,
          ordered: true,
          invalidDate: "error",
          unknownMeta: "error",
          empty: "error",
        },
      }),
    );
    expect(result.html).toContain("ox-timeline__content");
    expect(result.html).toContain("ox-container--tip");
    expect(result.html).toContain("<strong>container</strong>");
    expect(result.html).toContain("::: timeline");
    expect(result.html).toContain("<code");
  });
});

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return createDocsResolvedOptions({ highlight: false, ...overrides });
}

async function captureTransformWarnings<T>(
  run: () => Promise<T>,
): Promise<{ result: T; warnings: unknown[][] }> {
  const warnings: unknown[][] = [];
  const original = console.warn;
  console.warn = (...args: unknown[]) => {
    warnings.push(args);
  };
  try {
    return { result: await run(), warnings };
  } finally {
    console.warn = original;
  }
}

function flattenWarnings(warnings: unknown[][]): string {
  return warnings.flatMap((args) => args.map(String)).join("\n");
}
