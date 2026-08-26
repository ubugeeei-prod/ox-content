import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

describe("image gallery transform", () => {
  it("leaves gallery source literal unless opted in", async () => {
    const markdown = [
      '::: gallery title="Screenshots"',
      '- ![Hero](/hero.png "Hero view")',
      "- ![Chart](https://example.com/chart.png)",
      ":::",
      "",
    ].join("\n");

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/galleries.md",
      createResolvedOptions(),
    );
    expect(defaultResult.html).not.toContain("ox-image-gallery");
    expect(defaultResult.html).toContain("gallery");

    const enabledResult = await transformMarkdown(
      markdown,
      "docs/galleries.md",
      createResolvedOptions({
        imageGalleries: { enabled: true, missingAlt: "error", empty: "error" },
      }),
    );
    expect(enabledResult.html).toContain('<figure class="ox-image-gallery">');
    expect(enabledResult.html).toContain(
      '<figcaption class="ox-image-gallery__caption">Screenshots</figcaption>',
    );
    expect(enabledResult.html).toContain('<img src="/hero.png" alt="Hero" loading="lazy">');
    expect(enabledResult.html).toContain(
      '<figcaption class="ox-image-gallery__item-caption">Hero view</figcaption>',
    );
    expect(enabledResult.html).toContain(
      '<img src="https://example.com/chart.png" alt="Chart" loading="lazy">',
    );
    expect(enabledResult.html).not.toContain("<script");
  });

  it("reports strict missing-alt diagnostics without rewriting the block", async () => {
    const { result, warnings } = await captureTransformWarnings(() =>
      transformMarkdown(
        "::: gallery\n![](/x.png)\n:::\n",
        "docs/galleries.md",
        createResolvedOptions({
          imageGalleries: { enabled: true, missingAlt: "error", empty: "error" },
        }),
      ),
    );
    expect(flattenWarnings(warnings)).toContain("missing alt text");
    expect(result.html).not.toContain("ox-image-gallery");
    expect(result.html).toContain("gallery");
  });

  it("can warn for missing alt text and still render", async () => {
    const { result, warnings } = await captureTransformWarnings(() =>
      transformMarkdown(
        "::: gallery\n![](/x.png)\n:::\n",
        "docs/galleries.md",
        createResolvedOptions({
          imageGalleries: { enabled: true, missingAlt: "warn", empty: "error" },
        }),
      ),
    );
    expect(flattenWarnings(warnings)).toContain("missing alt text");
    expect(result.html).toContain('<img src="/x.png" alt="" loading="lazy">');
  });

  it("composes image attrs, dimensions, and eager loading", async () => {
    const result = await transformMarkdown(
      "::: gallery\n![Wide](./wide.png){.hero width=640 height=360 data-kind=wide}\n:::\n",
      "docs/galleries.md",
      createResolvedOptions({
        attrs: { enabled: true },
        images: { enabled: true, lazy: false },
        imageGalleries: { enabled: true, missingAlt: "error", empty: "error" },
      }),
    );
    expect(result.html).toContain(
      '<img src="./wide.png" alt="Wide" class="hero" data-kind="wide" width="640" height="360">',
    );
    expect(result.html).not.toContain("loading=");
    expect(result.html).not.toContain("{.hero");
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
