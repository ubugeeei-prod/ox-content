import { describe, expect, it } from "vite-plus/test";
import { render } from "svelte/server";
import { transformMarkdownWithSvelte } from "./transform";
import {
  createOptions,
  stripSvelteComments,
  withGeneratedModule,
} from "../test/fixtures/transform-harness";

const OPTIONS = { components: {}, mdxDocumentProps: true, ssr: true } as const;

/** Replaces the document-local import with an inline SSR component. */
function stubCounter(code: string, body: string): string {
  return code.replace(
    "import Counter from './Counter.svelte';",
    ["const Counter = ($$renderer, $$props) => {", body, "};"].join("\n"),
  );
}

async function transform(source: string) {
  return transformMarkdownWithSvelte(source, "/repo/docs/page.mdx", createOptions(OPTIONS));
}

describe("MDX document-props islands", () => {
  it("leaves an unmarked component as a plain child with no island runtime", async () => {
    const result = await transform(
      "import Counter from './Counter.svelte'\n\n<Counter initial={initial} />\n",
    );

    expect(result.code).not.toContain("data-ox-island");
    expect(result.code).not.toContain("initIslands");
    expect(result.code).not.toContain("hydrateIslands");
  });

  it("wraps a marked component in an island that carries its own SSR output", async () => {
    const result = await transform(
      "import Counter from './Counter.svelte'\n\n<Counter initial={initial} oxIsland />\n",
    );
    const code = stubCounter(
      result.code,
      "  $$renderer.push('<button>' + $$props.initial + '</button>');",
    );

    await withGeneratedModule(code, (Page, mod) => {
      const html = stripSvelteComments(render(Page, { props: { initial: 3 } }).body);

      expect(html).toContain('data-ox-island="Counter"');
      expect(html).toContain('data-ox-ssr="true"');
      expect(html).toContain('data-ox-load="eager"');
      // The server render lives inside the wrapper, so the runtime hydrates it
      // rather than mounting a second copy.
      expect(html).toContain("<button>3</button>");
      // The directive is consumed, never handed to the component.
      expect(html).not.toContain("oxIsland");
      expect(typeof mod.hydrateIslands).toBe("function");
    });
  });

  it("serialises the resolved document props onto the wrapper", async () => {
    const result = await transform(
      "import Counter from './Counter.svelte'\n\n<Counter initial={counts.start} label=\"Votes\" oxIsland />\n",
    );
    const code = stubCounter(result.code, "  $$renderer.push('<span></span>');");

    await withGeneratedModule(code, (Page) => {
      const html = render(Page, { props: { counts: { start: 7 } } }).body;
      const props = JSON.parse(
        /data-ox-props="([^"]*)"/.exec(html)?.[1].replaceAll("&quot;", '"') ?? "{}",
      );

      expect(props).toEqual({ label: "Votes", initial: 7 });
    });
  });

  it("carries a spread through to the wrapper in attribute order", async () => {
    const result = await transform(
      "import Counter from './Counter.svelte'\n\n<Counter {...base} initial={override} oxIsland />\n",
    );
    const code = stubCounter(result.code, "  $$renderer.push('<span></span>');");

    await withGeneratedModule(code, (Page) => {
      const html = render(Page, {
        props: { base: { initial: 1, tone: "quiet" }, override: 9 },
      }).body;
      const props = JSON.parse(
        /data-ox-props="([^"]*)"/.exec(html)?.[1].replaceAll("&quot;", '"') ?? "{}",
      );

      expect(props).toEqual({ initial: 9, tone: "quiet" });
    });
  });

  it("keeps authored children as the island slot", async () => {
    const result = await transform(
      "import Counter from './Counter.svelte'\n\n<Counter oxIsland>Press **me**</Counter>\n",
    );
    const code = stubCounter(
      result.code,
      [
        "  $$renderer.push('<button>');",
        "  $$props.children?.($$renderer);",
        "  $$renderer.push('</button>');",
      ].join("\n"),
    );

    await withGeneratedModule(code, (Page) => {
      const html = render(Page, { props: {} }).body;
      expect(html).toContain('data-ox-island="Counter"');
      expect(html).toContain("<strong>me</strong>");
    });
  });
});

describe("MDX document-props island load strategies", () => {
  for (const load of ["idle", "visible"] as const) {
    it(`passes the ${load} strategy through to the wrapper`, async () => {
      const result = await transform(
        `import Counter from './Counter.svelte'\n\n<Counter oxIsland="${load}" />\n`,
      );
      const code = stubCounter(result.code, "  $$renderer.push('<span></span>');");

      await withGeneratedModule(code, (Page) => {
        expect(render(Page, { props: {} }).body).toContain(`data-ox-load="${load}"`);
      });
    });
  }

  it("carries the media query alongside the media strategy", async () => {
    const result = await transform(
      'import Counter from \'./Counter.svelte\'\n\n<Counter oxIsland="media" oxIslandMedia="(min-width: 40em)" />\n',
    );
    const code = stubCounter(result.code, "  $$renderer.push('<span></span>');");

    await withGeneratedModule(code, (Page) => {
      const html = render(Page, { props: {} }).body;
      expect(html).toContain('data-ox-load="media"');
      expect(html).toContain("(min-width: 40em)");
    });
  });

  it("rejects the media strategy with no query to wait for", async () => {
    await expect(
      transform("import Counter from './Counter.svelte'\n\n<Counter oxIsland=\"media\" />\n"),
    ).rejects.toThrow(/needs oxIslandMedia with the query to wait for/);
  });

  it("rejects an unknown strategy", async () => {
    await expect(
      transform("import Counter from './Counter.svelte'\n\n<Counter oxIsland=\"soon\" />\n"),
    ).rejects.toThrow(/Unknown island load strategy "soon"/);
  });

  it("rejects a strategy resolved from a document prop", async () => {
    await expect(
      transform("import Counter from './Counter.svelte'\n\n<Counter oxIsland={strategy} />\n"),
    ).rejects.toThrow(/has to be a literal, not a document prop/);
  });
});

describe("MDX document-props island serialisation diagnostics", () => {
  it("names the prop that cannot cross to the client", async () => {
    const result = await transform(
      "import Counter from './Counter.svelte'\n\n<Counter onSelect={handlers.onSelect} oxIsland />\n",
    );
    const code = stubCounter(result.code, "  $$renderer.push('<span></span>');");

    await withGeneratedModule(code, (Page) => {
      expect(() => render(Page, { props: { handlers: { onSelect: () => {} } } }).body).toThrow(
        /received a function for prop "onSelect"/,
      );
    });
  });

  it("names a nested prop rather than the object holding it", async () => {
    const result = await transform(
      "import Counter from './Counter.svelte'\n\n<Counter config={config} oxIsland />\n",
    );
    const code = stubCounter(result.code, "  $$renderer.push('<span></span>');");

    await withGeneratedModule(code, (Page) => {
      expect(
        () => render(Page, { props: { config: { retries: 2, onDone: () => {} } } }).body,
      ).toThrow(/prop "config\.onDone"/);
    });
  });

  it("reports a cycle instead of throwing an opaque stringify error", async () => {
    const result = await transform(
      "import Counter from './Counter.svelte'\n\n<Counter config={config} oxIsland />\n",
    );
    const code = stubCounter(result.code, "  $$renderer.push('<span></span>');");
    const config: Record<string, unknown> = {};
    config.self = config;

    await withGeneratedModule(code, (Page) => {
      expect(() => render(Page, { props: { config } }).body).toThrow(/circular value/);
    });
  });

  it("lets a plain JSON prop through untouched", async () => {
    const result = await transform(
      "import Counter from './Counter.svelte'\n\n<Counter config={config} oxIsland />\n",
    );
    const code = stubCounter(result.code, "  $$renderer.push('<span></span>');");

    await withGeneratedModule(code, (Page) => {
      const html = render(Page, { props: { config: { retries: 2, tags: ["a", "b"] } } }).body;
      const props = JSON.parse(
        /data-ox-props="([^"]*)"/.exec(html)?.[1].replaceAll("&quot;", '"') ?? "{}",
      );
      expect(props).toEqual({ config: { retries: 2, tags: ["a", "b"] } });
    });
  });
});
