import assert from "node:assert/strict";
import { test } from "vite-plus/test";
import { jsx } from "hono/jsx";
import { transform } from "@ox-content/napi";
import { createHonoServerAdapter } from "./server.js";
import { createHonoRenderer } from "./vite.js";
import { useFrontmatter } from "./context.js";

test("public Ox Content pipeline renders props, static slots and selected frontmatter", async () => {
  const parsed = transform(
    'import Card from "./Card.tsx"\n\n<Card title="&lt;safe&gt;">**bold**</Card>',
    { mdx: true },
  );
  const renderer = createHonoRenderer({
    root: "/site",
    frontmatter: { title: "Shared" },
    loadModule: async () => ({
      default: ({ title, children }) =>
        jsx("section", null, title, useFrontmatter().title, children),
    }),
  });
  const result = await renderer(parsed.html, {
    documentPath: "/site/page.mdx",
    imports: parsed.imports,
  });
  assert.deepEqual(result.diagnostics, []);
  assert.match(result.html, /Shared/);
  assert.match(result.html, /<strong>bold<\/strong>/);
  assert.match(result.html, /data-ox-module=/);
});

test("frontmatter is opt-in, escaped inside JSON and isolated across async renders", async () => {
  const component = async () => {
    await new Promise((r) => setTimeout(r, 2));
    return jsx("p", null, useFrontmatter().title ?? "none");
  };
  const html = await createHonoServerAdapter().renderComponent(component, {}, undefined, {});
  assert.match(html, /<p>none<\/p>/);
  const dangerous = "</script><img src=x onerror=alert(1)>";
  const safe = await createHonoServerAdapter({
    frontmatter: { title: dangerous },
  }).renderComponent(component, {}, undefined, {});
  assert.equal((safe.match(/<\/script>/g) ?? []).length, 1);
  assert.doesNotMatch(safe, /<img/);
  const results = await Promise.all(
    Array.from({ length: 12 }, (_, i) =>
      createHonoServerAdapter({
        frontmatter: { title: String(i) },
      }).renderComponent(component, {}, undefined, {}),
    ),
  );
  results.forEach((value, i) => assert.match(value, new RegExp("<p>" + i + "</p>")));
});

test("unsupported components and missing imports are explicit failures", async () => {
  await assert.rejects(
    createHonoServerAdapter().renderComponent({}, {}, undefined, {}),
    /function component/,
  );
  const render = createHonoRenderer({
    root: "/site",
    loadModule: async () => ({}),
  });
  await assert.rejects(
    render('<div data-ox-island="Missing"></div>', {
      documentPath: "/site/page.mdx",
    }),
    /missing-component/,
  );
  assert.throws(() => createHonoServerAdapter({ frontmatter: { a: 1n } }), /BigInt/);
});

test("loading directives are validated, serialized and not passed as component props", async () => {
  const adapter = createHonoServerAdapter();
  const component = (props) => jsx("p", null, Object.keys(props).join(","));
  for (const [directive, value, expected] of [
    ["client:load", true, "eager"],
    ["client:idle", true, "idle"],
    ["client:visible", true, "visible"],
    ["client:media", "(max-width: 500px)", "media"],
  ]) {
    const html = await adapter.renderComponent(
      component,
      { [directive]: value, initial: 2 },
      undefined,
      {},
    );
    assert.match(html, new RegExp('"load":"' + expected + '"'));
    assert.doesNotMatch(html, /<p>[^<]*client:/);
  }
  await assert.rejects(
    adapter.renderComponent(
      component,
      { "client:idle": true, "client:visible": true },
      undefined,
      {},
    ),
    /one client/,
  );
  await assert.rejects(
    adapter.renderComponent(component, { "client:media": true }, undefined, {}),
    /media query/,
  );
  await assert.rejects(
    adapter.renderComponent(component, { "client:only": true }, undefined, {}),
    /Supported Hono/,
  );
});

test("native MDX imports carry a loading directive through the public HTML host", async () => {
  const parsed = transform('import Card from "./Card.tsx"\n\n<Card client:visible />', {
    mdx: true,
  });
  const render = createHonoRenderer({
    root: "/site",
    loadModule: async () => ({ default: () => jsx("b", null, "SSR") }),
  });
  const result = await render(parsed.html, {
    documentPath: "/site/page.mdx",
    imports: parsed.imports,
  });
  assert.match(result.html, /"load":"visible"/);
  assert.match(result.html, /<b>SSR<\/b>/);
});
