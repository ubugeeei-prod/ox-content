import fs from "node:fs/promises";
import * as os from "node:os";
import path from "node:path";
import solid from "@solidjs/vite-plugin";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build as viteBuild, type Plugin, type ViteDevServer } from "vite";
import {
  SOLID_HTML_HOST_MODULES_VIRTUAL_ID,
  createSolidHtmlHostIslandRegistry,
  renderSolidHtmlHost,
  resolveSolidHtmlHostIslandRegistry,
  resolveSolidIslandStylesheets,
  toSolidHtmlHostClientModuleId,
  type MdxImport,
} from ".";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("createSolidHtmlHostIslandRegistry", () => {
  it("shares custom-host SSR marker identities with the generated client registry", async () => {
    const root = await createProject("ox-solid-html-registry-");
    const documentPath = path.join(root, "content", "published.mdx");
    const imports = [defaultImport("Probe", "./published/Probe.tsx")];
    const html = '<div data-ox-island="Probe"></div>';
    const registry = createSolidHtmlHostIslandRegistry({
      root,
      documents: [{ documentPath, html, imports }],
    });

    const rendered = await renderSolidHtmlHost({
      html,
      documentPath,
      root,
      srcDir: "content",
      imports,
      resolveClientModule: (module) => registry.resolveClientModule(module),
      loadModule: async () => ({ default: "probe" }),
      renderComponent: (component) => `<strong>${component as string}</strong>`,
    });
    const resolved = await registry.resolve();

    expect(rendered.clientModules).toEqual([
      { name: "Probe", moduleId: "/content/published/Probe.tsx", exportName: "default" },
    ]);
    expect(rendered.html).toContain('data-ox-module="/content/published/Probe.tsx"');
    expect(resolved.modules).toEqual(rendered.clientModules);
  });

  it("resolves selected documents and explicit approved entries without broad globs", async () => {
    const root = await createProject("ox-solid-html-selection-");
    const result = await resolveSolidHtmlHostIslandRegistry(
      {
        components: { Shared: "./src/Shared.tsx" },
        entries: [{ name: "Approved", moduleId: "./src/Approved.tsx" }],
        documents: [
          {
            documentPath: path.join(root, "content", "published.mdx"),
            html: '<div data-ox-island="Probe"></div><div data-ox-island="Shared"></div>',
            imports: [defaultImport("Probe", "./published/Probe.tsx")],
          },
        ],
      },
      { root, mode: "production", command: "build" },
    );

    expect(result.modules).toEqual([
      { name: "Probe", moduleId: "/content/published/Probe.tsx", exportName: "default" },
      { name: "Approved", moduleId: "/src/Approved.tsx", exportName: "default" },
      { name: "Shared", moduleId: "/src/Shared.tsx", exportName: "default" },
    ]);
    expect(result.modules).not.toContainEqual(
      expect.objectContaining({ moduleId: expect.stringContaining("draft") }),
    );
  });

  it("builds a browser registry that includes reachable selected chunks only", async () => {
    const root = await createProject("ox-solid-html-build-");
    const registry = createSolidHtmlHostIslandRegistry({
      root,
      documents: [
        {
          documentPath: path.join(root, "content", "published.mdx"),
          html: '<div data-ox-island="Probe"></div><div data-ox-island="SharedProbe"></div>',
          imports: [
            defaultImport("Probe", "./published/Probe.ts"),
            namedImport("SharedProbe", "SharedProbe", "./published/Probe.ts"),
          ],
        },
        {
          documentPath: path.join(root, "content", "second.mdx"),
          html: '<div data-ox-island="SharedProbe"></div>',
          imports: [namedImport("SharedProbe", "SharedProbe", "./published/Probe.ts")],
        },
      ],
      watch: ["content/publication.json"],
    });

    await fs.writeFile(
      path.join(root, "src", "client.tsx"),
      [
        `import { clientModules, modules } from "${SOLID_HTML_HOST_MODULES_VIRTUAL_ID}";`,
        "globalThis.__oxModules = modules;",
        "globalThis.__oxClientModules = clientModules;",
      ].join("\n"),
    );

    await viteBuild({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [registry.plugin, solid({ compiler: "native" })],
      build: {
        outDir: "dist",
        emptyOutDir: true,
        manifest: true,
        minify: false,
        rollupOptions: { input: path.join(root, "src", "client.tsx") },
      },
    });

    const output = await readDist(root);
    expect(output).toContain("PUBLIC_SENTINEL");
    expect(output).toContain("SHARED_HELPER_SENTINEL");
    expect(output).not.toContain("DRAFT_ONLY_SENTINEL");
    expect(output).not.toContain("MISSING_POLICY_SENTINEL");
    expect(output).not.toContain("@ox-content/vite-plugin");
    expect(output).not.toContain("node:");

    const manifest = JSON.parse(
      await fs.readFile(path.join(root, "dist", ".vite", "manifest.json"), "utf8"),
    );
    const resolved = await registry.resolve();
    const styles = resolveSolidIslandStylesheets({
      modules: resolved.modules.map((module) => module.moduleId),
      manifest,
    });
    expect(styles.diagnostics).toEqual([]);
    expect(styles.stylesheets.map((style) => style.href).join("\n")).toContain("Probe");
  });

  it("invalidates the virtual registry when selected content changes in dev", async () => {
    const root = await createProject("ox-solid-html-dev-");
    let document = {
      html: '<div data-ox-island="Probe"></div>',
      imports: [defaultImport("Probe", "./published/Probe.tsx")],
    };
    const registry = createSolidHtmlHostIslandRegistry({
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

    const id = `\0${SOLID_HTML_HOST_MODULES_VIRTUAL_ID}`;
    const first = await loadVirtual(plugin, id);
    document = {
      html: '<div data-ox-island="DraftProbe"></div>',
      imports: [defaultImport("DraftProbe", "./draft.tsx")],
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

    expect(first).toContain("/content/published/Probe.tsx");
    expect(second).toContain("/content/draft.tsx");
    expect(second).not.toContain("/content/published/Probe.tsx");
    expect(updated).toEqual([]);
    expect(invalidated).toHaveLength(1);
    expect(messages).toEqual([{ type: "full-reload" }]);
  });

  it("normalizes filesystem and Vite module ids for browser loaders", () => {
    expect(toSolidHtmlHostClientModuleId("/repo/src/Chart.tsx", "/repo")).toBe("/src/Chart.tsx");
    expect(toSolidHtmlHostClientModuleId("./src/Chart.tsx", "/repo")).toBe("/src/Chart.tsx");
    expect(toSolidHtmlHostClientModuleId("solid-js")).toBe("solid-js");
  });
});

async function createProject(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(root);
  await fs.mkdir(path.join(root, "content", "published"), { recursive: true });
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.writeFile(path.join(root, "package.json"), '{"type":"module"}\n');
  await fs.writeFile(path.join(root, "content", "published.mdx"), "# Published\n<Probe />\n");
  await fs.writeFile(path.join(root, "content", "draft.mdx"), "# Draft\n<DraftProbe />\n");
  await fs.writeFile(path.join(root, "content", "missing.mdx"), "# Missing\n<MissingProbe />\n");
  const probeModule = [
    'import "./Probe.css";',
    'import { helper } from "./helper";',
    'export function SharedProbe() { console.log("PUBLIC_SENTINEL", helper); return "shared"; }',
    'export default function Probe() { console.log("PUBLIC_SENTINEL", helper); return "probe"; }',
  ].join("\n");
  await fs.writeFile(path.join(root, "content", "published", "Probe.ts"), probeModule);
  await fs.writeFile(path.join(root, "content", "published", "Probe.tsx"), probeModule);
  await fs.writeFile(
    path.join(root, "content", "published", "helper.ts"),
    'export const helper = "SHARED_HELPER_SENTINEL";\n',
  );
  await fs.writeFile(path.join(root, "content", "published", "Probe.css"), ".Probe{color:red}\n");
  await fs.writeFile(
    path.join(root, "content", "draft.tsx"),
    'export default function DraftProbe() { console.log("DRAFT_ONLY_SENTINEL"); return <i />; }\n',
  );
  await fs.writeFile(
    path.join(root, "content", "missing.tsx"),
    'export default function MissingProbe() { console.log("MISSING_POLICY_SENTINEL"); return <i />; }\n',
  );
  return root;
}

function defaultImport(local: string, source: string): MdxImport {
  return { source, specifiers: [{ imported: "default", local, kind: "default" }] };
}

function namedImport(local: string, imported: string, source: string): MdxImport {
  return { source, specifiers: [{ imported, local, kind: "named" }] };
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
