import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { svelte as sveltePlugin } from "@sveltejs/vite-plugin-svelte";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build as viteBuild, type Plugin, type ViteDevServer } from "vite";
import {
  SVELTE_HTML_HOST_MODULES_VIRTUAL_ID,
  createSvelteHtmlHostIslandRegistry,
  renderSvelteHtmlHost,
  resolveSvelteHtmlHostIslandRegistry,
  toSvelteHtmlHostClientModuleId,
  type MdxImport,
} from ".";

const tempDirs: string[] = [];
const PACKAGE_ROOT = path.dirname(fileURLToPath(new URL("../package.json", import.meta.url)));

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("createSvelteHtmlHostIslandRegistry", () => {
  it("shares custom-host SSR marker identities with the generated client registry", async () => {
    const root = await createProject("ox-svelte-html-registry-");
    const documentPath = path.join(root, "content", "published.mdx");
    const imports = [defaultImport("Probe", "./published/Probe.svelte")];
    const html = '<div data-ox-island="Probe"></div>';
    const registry = createSvelteHtmlHostIslandRegistry({
      root,
      documents: [{ documentPath, html, imports }],
    });

    const rendered = await renderSvelteHtmlHost({
      html,
      documentPath,
      root,
      srcDir: "content",
      imports,
      resolveClientModule: (module) => registry.resolveClientModule(module),
      loadModule: async () => ({ default: "probe" }),
      renderComponent: (component) => `<strong>${String(component)}</strong>`,
    });
    const resolved = await registry.resolve();

    expect(rendered.clientModules).toEqual([
      { name: "Probe", moduleId: "/content/published/Probe.svelte", exportName: "default" },
    ]);
    expect(rendered.html).toContain('data-ox-module="/content/published/Probe.svelte"');
    expect(resolved.modules).toEqual(rendered.clientModules);
  });

  it("resolves selected documents and explicit approved entries without broad globs", async () => {
    const root = await createProject("ox-svelte-html-selection-");
    const result = await resolveSvelteHtmlHostIslandRegistry(
      {
        components: { Shared: "./src/Shared.svelte" },
        entries: [{ name: "Approved", moduleId: "./src/Approved.svelte" }],
        documents: [
          {
            documentPath: path.join(root, "content", "published.mdx"),
            html: '<div data-ox-island="Probe"></div><div data-ox-island="Shared"></div>',
            imports: [defaultImport("Probe", "./published/Probe.svelte")],
          },
        ],
      },
      { root, mode: "production", command: "build" },
    );

    expect(result.modules).toEqual([
      { name: "Probe", moduleId: "/content/published/Probe.svelte", exportName: "default" },
      { name: "Approved", moduleId: "/src/Approved.svelte", exportName: "default" },
      { name: "Shared", moduleId: "/src/Shared.svelte", exportName: "default" },
    ]);
    expect(result.modules).not.toContainEqual(
      expect.objectContaining({ moduleId: expect.stringContaining("draft") }),
    );
  });

  it("builds a browser registry that includes reachable selected chunks only", async () => {
    const root = await createProject("ox-svelte-html-build-");
    const registry = createSvelteHtmlHostIslandRegistry({
      root,
      documents: [
        {
          documentPath: path.join(root, "content", "published.mdx"),
          html: '<div data-ox-island="Probe"></div>',
          imports: [defaultImport("Probe", "./published/Probe.svelte")],
        },
      ],
      watch: ["content/publication.json"],
    });

    await fs.writeFile(
      path.join(root, "src", "client.js"),
      [
        `import { clientModules, modules } from "${SVELTE_HTML_HOST_MODULES_VIRTUAL_ID}";`,
        "globalThis.__oxModules = modules;",
        "globalThis.__oxClientModules = clientModules;",
      ].join("\n"),
    );

    await viteBuild({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [registry.plugin, sveltePlugin()],
      build: {
        outDir: "dist",
        emptyOutDir: true,
        manifest: true,
        minify: false,
        rollupOptions: { input: path.join(root, "src", "client.js") },
      },
    });

    const output = await readDist(root);
    expect(output).toContain("PUBLIC_SENTINEL");
    expect(output).toContain("SHARED_HELPER_SENTINEL");
    expect(output).not.toContain("DRAFT_ONLY_SENTINEL");
    expect(output).not.toContain("MISSING_POLICY_SENTINEL");
    expect(output).not.toContain("@ox-content/vite-plugin");
    expect(output).not.toContain("node:");
  });

  it("invalidates the virtual registry when selected content changes in dev", async () => {
    const root = await createProject("ox-svelte-html-dev-");
    let document = {
      html: '<div data-ox-island="Probe"></div>',
      imports: [defaultImport("Probe", "./published/Probe.svelte")],
    };
    const registry = createSvelteHtmlHostIslandRegistry({
      root,
      documents: () => [
        {
          documentPath: path.join(root, "content", "published.mdx"),
          ...document,
        },
      ],
    });
    const plugin = registry.plugin as Plugin;
    await (plugin.configResolved as (config: unknown) => void | Promise<void>)({
      root,
      mode: "development",
    });

    const id = `\0${SVELTE_HTML_HOST_MODULES_VIRTUAL_ID}`;
    const first = await loadVirtual(plugin, id);
    document = {
      html: '<div data-ox-island="DraftProbe"></div>',
      imports: [defaultImport("DraftProbe", "./draft.svelte")],
    };
    const invalidated: unknown[] = [];
    const messages: unknown[] = [];
    const handleHotUpdate = plugin.handleHotUpdate as (ctx: unknown) => Promise<unknown[]>;
    const updated = await handleHotUpdate({
      file: path.join(root, "content", "published.mdx"),
      modules: [],
      server: {
        moduleGraph: {
          getModuleById: () => ({}),
          invalidateModule: (mod: unknown) => invalidated.push(mod),
        },
        ws: { send: (message: unknown) => messages.push(message) },
      } as unknown as ViteDevServer,
    } as never);
    const second = await loadVirtual(plugin, id);

    expect(first).toContain("/content/published/Probe.svelte");
    expect(second).toContain("/content/draft.svelte");
    expect(second).not.toContain("/content/published/Probe.svelte");
    expect(updated).toEqual([]);
    expect(invalidated).toHaveLength(1);
    expect(messages).toEqual([{ type: "full-reload" }]);
  });

  it("normalizes filesystem and Vite module ids for browser loaders", () => {
    expect(toSvelteHtmlHostClientModuleId("/repo/src/Chart.svelte", "/repo")).toBe(
      "/src/Chart.svelte",
    );
    expect(toSvelteHtmlHostClientModuleId("./src/Chart.svelte", "/repo")).toBe("/src/Chart.svelte");
    expect(toSvelteHtmlHostClientModuleId("svelte")).toBe("svelte");
  });
});

async function createProject(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(PACKAGE_ROOT, `.tmp-${prefix}`));
  tempDirs.push(root);
  await fs.mkdir(path.join(root, "content", "published"), { recursive: true });
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.writeFile(path.join(root, "package.json"), '{"type":"module"}\n');
  await fs.writeFile(path.join(root, "content", "published.mdx"), "# Published\n<Probe />\n");
  await fs.writeFile(path.join(root, "content", "draft.mdx"), "# Draft\n<DraftProbe />\n");
  await fs.writeFile(path.join(root, "content", "missing.mdx"), "# Missing\n<MissingProbe />\n");
  await fs.writeFile(path.join(root, "content", "published", "Probe.svelte"), probeModule());
  await fs.writeFile(
    path.join(root, "content", "published", "helper.js"),
    'export const helper = "SHARED_HELPER_SENTINEL";\n',
  );
  await fs.writeFile(
    path.join(root, "content", "draft.svelte"),
    '<script>console.log("DRAFT_ONLY_SENTINEL");</script><i />\n',
  );
  await fs.writeFile(
    path.join(root, "content", "missing.svelte"),
    '<script>console.log("MISSING_POLICY_SENTINEL");</script><i />\n',
  );
  await fs.writeFile(path.join(root, "src", "Approved.svelte"), "<span>approved</span>\n");
  await fs.writeFile(path.join(root, "src", "Shared.svelte"), "<span>shared</span>\n");
  return root;
}

function probeModule(): string {
  return [
    "<script>",
    '  import { helper } from "./helper.js";',
    '  console.log("PUBLIC_SENTINEL", helper);',
    "</script>",
    '<button class="Probe">probe</button>',
    "<style>.Probe{color:red}</style>",
  ].join("\n");
}

function defaultImport(local: string, source: string): MdxImport {
  return { source, specifiers: [{ imported: "default", local, kind: "default" }] };
}

async function loadVirtual(plugin: Plugin, id: string): Promise<string> {
  const loaded = await (plugin.load as (id: string) => Promise<string>)(id);
  return String(loaded);
}

async function readDist(root: string): Promise<string> {
  const chunks: string[] = [];
  await collectFiles(path.join(root, "dist"), chunks);
  return chunks.join("\n");
}

async function collectFiles(dir: string, chunks: string[]): Promise<void> {
  for (const entry of await fs.readdir(dir, { withFileTypes: true })) {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      await collectFiles(file, chunks);
    } else if (entry.isFile()) {
      chunks.push(await fs.readFile(file, "utf8"));
    }
  }
}
