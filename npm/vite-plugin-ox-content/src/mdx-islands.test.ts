import { describe, expect, it } from "vite-plus/test";
import {
  collectMdxIslandNamesFromHtml,
  collectMdxJsxNamesFromAst,
  discoverRegisteredMdxComponents,
  intersectRegisteredComponentNames,
} from "./mdx-islands";

describe("collectMdxJsxNamesFromAst", () => {
  it("walks nested JSX and skips fragments", () => {
    const ast = {
      type: "root",
      children: [
        {
          type: "mdxJsxFlowElement",
          name: "Callout",
          children: [
            {
              type: "paragraph",
              children: [{ type: "mdxJsxTextElement", name: "Badge", children: [] }],
            },
          ],
        },
        {
          type: "mdxJsxFlowElement",
          name: null,
          children: [{ type: "mdxJsxFlowElement", name: "Alert", children: [] }],
        },
      ],
    };

    expect(collectMdxJsxNamesFromAst(ast)).toEqual(["Callout", "Badge", "Alert"]);
  });
});

describe("collectMdxIslandNamesFromHtml", () => {
  it("collects unique data-ox-island names", () => {
    const html = [
      '<div class="ox-island" data-ox-island="Callout"></div>',
      '<span data-ox-island="Badge"></span>',
      '<div data-ox-island="Callout"></div>',
    ].join("");

    expect(collectMdxIslandNamesFromHtml(html)).toEqual(["Callout", "Badge"]);
  });
});

describe("intersectRegisteredComponentNames", () => {
  it("keeps registered names from objects and maps", () => {
    expect(
      intersectRegisteredComponentNames(["Alert", "Unknown"], { Alert: "./Alert.tsx" }),
    ).toEqual(["Alert"]);
    expect(
      intersectRegisteredComponentNames(["Alert", "Badge"], new Map([["Badge", "./Badge.vue"]])),
    ).toEqual(["Badge"]);
  });
});

describe("discoverRegisteredMdxComponents", () => {
  it("finds nested registered names from the MDX AST", async () => {
    const used = await discoverRegisteredMdxComponents({
      source: '<Callout>\n\n# Title\n\n<Badge title="hi" />\n\n</Callout>\n',
      components: { Callout: "./Callout.tsx", Badge: "./Badge.tsx" },
    });
    expect(used).toEqual(["Callout", "Badge"]);
  });

  it("ignores fenced JSX and unregistered names", async () => {
    const used = await discoverRegisteredMdxComponents({
      source: ["<Alert />", "", "```tsx", "<Alert />", "```", "", "<Unknown />", ""].join("\n"),
      components: { Alert: "./Alert.tsx" },
    });
    expect(used).toEqual(["Alert"]);
  });

  it("finds a registered component inside a fragment", async () => {
    const used = await discoverRegisteredMdxComponents({
      source: '<>\n<Alert tone="info" />\n</>\n',
      components: { Alert: "./Alert.tsx" },
    });
    expect(used).toEqual(["Alert"]);
  });

  it("treats document-local bindings as hydratable and lets them override globals", async () => {
    const localOnly = await discoverRegisteredMdxComponents({
      source: '<GtvChart title="ok" />\n',
      components: {},
      localNames: ["GtvChart"],
    });
    expect(localOnly).toEqual(["GtvChart"]);

    const override = await discoverRegisteredMdxComponents({
      source: "<Chart />\n<Alert />\n",
      components: { Chart: "./GlobalChart.tsx", Alert: "./Alert.tsx" },
      localNames: ["Chart"],
    });
    expect(override).toEqual(["Chart", "Alert"]);
  });

  it("reserves built-in embed names unless a document-local import overrides them", async () => {
    const builtin = await discoverRegisteredMdxComponents({
      source: '<Tweet id="1234567890" />\n<OgCard url="https://example.com" />\n<Alert />\n',
      components: { Tweet: "./Tweet.tsx", OgCard: "./OgCard.tsx", Alert: "./Alert.tsx" },
    });
    expect(builtin).toEqual(["Alert"]);

    const localOverride = await discoverRegisteredMdxComponents({
      source: '<Tweet id="1234567890" />\n<Alert />\n',
      components: { Tweet: "./GlobalTweet.tsx", Alert: "./Alert.tsx" },
      localNames: ["Tweet"],
    });
    expect(localOverride).toEqual(["Tweet", "Alert"]);
  });
});
