import { createRequire } from "node:module";
import { describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };
import {
  renderDocumentAssetTag,
  renderDocumentAssets,
  type DocumentAssetManifest,
} from "./document-assets";

const require = createRequire(import.meta.url);

describe("document asset public API", () => {
  it("declares standalone document-assets and Vite custom-host subpaths", () => {
    const exportsField = packageJson.exports as unknown as Record<string, PackageConditionalExport>;
    const documentAssets = exportsField["./document-assets"];
    const customHost = exportsField["./custom-host"];

    expect(documentAssets.import.types).toBe("./dist/document-assets.d.mts");
    expect(documentAssets.import.default).toBe("./dist/document-assets.mjs");
    expect(documentAssets.require.types).toBe("./dist/document-assets.d.cts");
    expect(documentAssets.require.default).toBe("./dist/document-assets.cjs");
    expect(customHost.import.types).toBe("./dist/custom-host.d.mts");
    expect(customHost.import.default).toBe("./dist/custom-host.mjs");

    const entries: string[] = require("../vite.config.ts").default.pack.entry;
    expect(entries).toEqual(
      expect.arrayContaining([
        "src/document-assets.ts",
        "src/custom-host.ts",
        "src/theme-bootstrap.ts",
      ]),
    );
  });

  it("renders complete head assets from typed descriptors without HTML scraping", () => {
    const manifest: DocumentAssetManifest = {
      "src/main.ts": {
        src: "src/main.ts",
        file: "assets/main.123.js",
        imports: ["_vendor.js"],
        css: ['assets/page.css?theme=light&name="quoted"#top'],
      },
      "_vendor.js": {
        file: "assets/vendor.456.js",
        css: ["assets/vendor.css", 'assets/page.css?theme=light&name="quoted"#top'],
      },
    };

    const result = renderDocumentAssets({
      base: "/docs/",
      head: { html: "<title>Guide</title>" },
      manifest,
      selfHostedAssets: {
        preloads: [
          {
            href: "/docs/__ox_fonts__/ox-test.woff2",
            as: "font",
            type: "font/woff2",
            crossorigin: true,
          },
        ],
        stylesheets: ["/docs/__ox_icons__/icons.css"],
      },
      links: [{ rel: "manifest", href: "/site.webmanifest" }],
      sharedStyles: ["/src/shared.css"],
      pageStyles: [
        {
          href: 'assets/page.css?theme=light&name="quoted"#top',
          attrs: { "data-owner": "page&asset" },
        },
      ],
      islandStyles: [{ href: "assets/island.css", crossorigin: "anonymous" }],
      inlineStyles: [{ content: 'body::before{content:"</StYle> & keep";}', nonce: "style-nonce" }],
      clientEntries: ["src/main.ts"],
      scripts: [{ content: 'globalThis.__x="</ScRiPt>";', nonce: "script-nonce" }],
      crossorigin: true,
    });

    expect(result.headHtml).toContain("<title>Guide</title>");
    expect(result.headHtml).toContain(
      '<link rel="preload" href="/docs/__ox_fonts__/ox-test.woff2" as="font" type="font/woff2" crossorigin>',
    );
    expect(result.headHtml).toContain(
      '<link rel="stylesheet" href="/docs/assets/page.css?theme=light&amp;name=&quot;quoted&quot;#top" data-owner="page&amp;asset">',
    );
    expect(result.headHtml).toContain('href="/docs/assets/vendor.css"');
    expect(result.headHtml).toContain(
      '<script type="module" src="/docs/assets/main.123.js" crossorigin></script>',
    );
    expect(result.headHtml).toContain("<\\/StYle>");
    expect(result.headHtml).toContain("<\\/ScRiPt>");
    expect(result.headHtml.toLowerCase()).not.toContain("</style> & keep");
    expect(result.headHtml.toLowerCase()).not.toContain("</script>;");

    const vendorIndex = result.headHtml.indexOf("vendor.css");
    const pageIndex = result.headHtml.indexOf("page.css");
    const islandIndex = result.headHtml.indexOf("island.css");
    const scriptIndex = result.headHtml.indexOf("main.123.js");
    expect(vendorIndex).toBeGreaterThan(0);
    expect(pageIndex).toBeGreaterThan(0);
    expect(islandIndex).toBeGreaterThan(pageIndex);
    expect(scriptIndex).toBeGreaterThan(islandIndex);
    expect(scriptIndex).toBeGreaterThan(vendorIndex);
    expect(result.styles.filter((style) => style.href?.includes("page.css"))).toHaveLength(1);
  });

  it("preserves attrs for individual tags", () => {
    expect(
      renderDocumentAssetTag({
        kind: "script",
        src: "/assets/app.js",
        type: "module",
        attrs: { async: true, "data-name": 'a"b&c' },
      }),
    ).toBe('<script type="module" src="/assets/app.js" async data-name="a&quot;b&amp;c"></script>');
  });
});

interface PackageConditionalExport {
  import: {
    types: string;
    default: string;
  };
  require: {
    types: string;
    default: string;
  };
}
