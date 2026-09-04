import * as fs from "node:fs/promises";
import { createRequire } from "node:module";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import {
  createDevServerCache,
  createOxContentFetchHandler,
  createOxContentMiddleware,
  createOxContentRouter,
  resolveOxContentRoute,
  type OxContentRouterMiddleware,
} from "./router";
import { resolveMarkdownSourceOptions } from "./markdown-source";
import type { ResolvedOptions } from "./types";
import packageJson from "../package.json" with { type: "json" };
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";

const require = createRequire(import.meta.url);
const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("ox-content router", () => {
  it("declares the router package subpath and bundle entry", () => {
    const exportsField = packageJson.exports as unknown as Record<string, PackageConditionalExport>;
    const router = exportsField["./router"];

    expect(router.import.types).toBe("./dist/router.d.mts");
    expect(router.import.default).toBe("./dist/router.mjs");
    expect(router.require.types).toBe("./dist/router.d.cts");
    expect(router.require.default).toBe("./dist/router.cjs");

    const entries: string[] = require("../vite.config.ts").default.pack.entry;
    expect(entries).toContain("src/router.ts");
  });

  it("resolves base and routePrefix before looking up source files", () => {
    const options = routerOptions({ base: "/docs/", routePrefix: "/blog" });

    expect(resolveOxContentRoute("/docs/blog/guide?draft=1", options)).toMatchObject({
      originalPathname: "/docs/blog/guide",
      pathname: "/blog/guide",
      routePathname: "/guide",
      base: "/docs/",
      routePrefix: "blog",
    });
    expect(resolveOxContentRoute("/guide", options)).toBeUndefined();
    expect(resolveOxContentRoute("/docs/guide", options)).toBeUndefined();
  });

  it("serves Markdown pages through a Fetch API router without Vite HMR", async () => {
    const root = await makeSite({ "guide.md": "# Guide\n\nBody.\n" });
    const router = createOxContentRouter(
      routerOptions({ base: "/docs/", routePrefix: "blog" }),
      root,
    );

    const miss = await router.fetch(new Request("http://localhost/docs/guide"));
    expect(miss).toBeUndefined();

    const response = await router.fetch(new Request("http://localhost/docs/blog/guide"));
    expect(response?.status).toBe(200);
    expect(response?.headers.get("content-type")).toBe("text/html");
    const html = await response?.text();

    expect(html).toContain("Guide");
    expect(html).toContain("Body.");
    expect(html).not.toContain("/@vite/client");
  });

  it("keeps HMR and static cache entries separate", async () => {
    const root = await makeSite({ "guide.md": "# Guide\n" });
    const cache = createDevServerCache();
    const hot = createOxContentRouter(routerOptions(), root, { cache, hmr: true });
    const staticRouter = createOxContentRouter(routerOptions(), root, { cache });

    const hotHtml = await (await hot.fetch(new Request("http://localhost/guide")))?.text();
    const staticHtml = await (
      await staticRouter.fetch(new Request("http://localhost/guide"))
    )?.text();

    expect(hotHtml).toContain("/@vite/client");
    expect(staticHtml).not.toContain("/@vite/client");
  });

  it("runs middleware around the built-in page renderer", async () => {
    const root = await makeSite({ "guide.md": "# Guide\n\nBody.\n" });
    const calls: string[] = [];
    const annotate: OxContentRouterMiddleware = async (context, next) => {
      calls.push(context.match.routePathname);
      const response = await next();
      if (!response) {
        return undefined;
      }
      const headers = new Headers(response.headers);
      headers.set("x-ox-route", context.match.routePathname);
      return new Response(await response.text(), {
        headers,
        status: response.status,
        statusText: response.statusText,
      });
    };
    const router = createOxContentRouter(routerOptions(), root, { middleware: [annotate] });

    const response = await router.fetch(new Request("http://localhost/guide"));

    expect(calls).toEqual(["/guide"]);
    expect(response?.headers.get("x-ox-route")).toBe("/guide");
    expect(await response?.text()).toContain("Guide");
  });

  it("composes as middleware for Deno and Bun style fetch handlers", async () => {
    const root = await makeSite({ "index.md": "# Home\n" });
    const middleware = createOxContentMiddleware(routerOptions(), root);

    const hit = await middleware(
      new Request("http://localhost/"),
      () => new Response("fallback", { status: 418 }),
    );
    const miss = await middleware(
      new Request("http://localhost/missing"),
      () => new Response("fallback", { status: 418 }),
    );

    expect(hit?.status).toBe(200);
    expect(await hit?.text()).toContain("Home");
    expect(miss?.status).toBe(418);
    expect(await miss?.text()).toBe("fallback");
  });

  it("serves markdown-source companions through the same router", async () => {
    const source = "---\ntitle: Guide\n---\n# Guide\n";
    const root = await makeSite({ "guide.md": source });
    const router = createOxContentRouter(
      routerOptions({
        base: "/docs/",
        markdownSource: true,
        routePrefix: "blog",
      }),
      root,
    );

    const response = await router.fetch(new Request("http://localhost/docs/blog/guide.md"));

    expect(response?.status).toBe(200);
    expect(response?.headers.get("content-type")).toBe("text/markdown; charset=utf-8");
    expect(await response?.text()).toBe(source);
  });

  it("returns a complete fetch handler with a not-found fallback", async () => {
    const root = await makeSite({ "index.md": "# Home\n" });
    const handler = createOxContentFetchHandler(routerOptions(), root, {
      notFound: () => new Response("custom missing", { status: 404 }),
    });

    expect((await handler(new Request("http://localhost/"))).status).toBe(200);
    const missing = await handler(new Request("http://localhost/missing"));
    expect(missing.status).toBe(404);
    expect(await missing.text()).toBe("custom missing");
  });
});

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-router-"));
  tempDirs.push(root);
  for (const [name, source] of Object.entries(files)) {
    const full = path.join(root, "content", name);
    await fs.mkdir(path.dirname(full), { recursive: true });
    await fs.writeFile(full, source);
  }
  return root;
}

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

function routerOptions(
  input: {
    base?: string;
    routePrefix?: string;
    markdownSource?: boolean;
  } = {},
): ResolvedOptions {
  const defaults = createDocsResolvedOptions();
  return createDocsResolvedOptions({
    base: input.base ?? "/",
    ssg: {
      ...defaults.ssg,
      routePrefix: input.routePrefix,
      markdownSource: resolveMarkdownSourceOptions(input.markdownSource),
    },
    search: {
      enabled: false,
      limit: 10,
      prefix: true,
      fuzzy: false,
      placeholder: "Search",
      hotkey: "/",
    },
  });
}
