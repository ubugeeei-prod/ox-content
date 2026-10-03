import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const publicValues = [
  "createCollectionAssetsMiddleware",
  "planCollectionAssets",
  "resolveGitLastmods",
  "resolveSelfHostedAssetManifest",
  "planCollectionAssetsFromDocuments",
  "writeCollectionAssets",
  "writeSelfHostedAssets",
];
const publicTypes = [
  "CollectionAssetInput",
  "CollectionAssetManifest",
  "CollectionAssetManifestEntry",
  "CollectionAssetDocumentDiagnostic",
  "CollectionAssetDocumentDiagnosticCode",
  "CollectionAssetDocumentInput",
  "CollectionAssetDocumentReference",
  "CollectionAssetResolvedDocumentReference",
  "OxContentCustomHostAssetsContext",
  "OxContentCustomHostOptions",
  "OxContentCustomHostSsrStylesheetDescriptor",
  "OxContentCustomHostSsrStylesheetsInput",
  "OxContentCustomHostSsrStylesheetsOptions",
  "OxContentCustomHostSsrStylesheetsResult",
  "OxContentAssetManifest",
  "OxContentAssetPreload",
  "PlanCollectionAssetsInput",
  "PlanCollectionAssetsFromDocumentsInput",
  "PlanCollectionAssetsFromDocumentsResult",
  "SelfHostedAssetOptions",
  "SsgOutputPageInput",
  "WriteCollectionAssetsInput",
  "WriteCollectionAssetsResult",
  "WriteSelfHostedAssetsInput",
  "WriteSelfHostedAssetsResult",
];
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const tscBin = join("node_modules", ".bin", process.platform === "win32" ? "tsc.cmd" : "tsc");
const virtualReference = '/// <reference path="./virtual.d.ts" />';
const virtualModules = [
  "virtual:ox-content/collections",
  "virtual:ox-content/assets.css",
  "virtual:ox-content/asset-manifest",
  "virtual:ox-content/html-host/modules",
];

export function checkVitePluginDeclarations({ pkg, tarball, packDir, failures, readPackedFile }) {
  for (const extension of ["mts", "cts"]) {
    const declaration = readPackedFile(tarball, `dist/index.d.${extension}`);

    for (const name of [...publicValues, ...publicTypes]) {
      if (!new RegExp(`\\b${name}\\b`).test(declaration)) {
        failures.push(`${pkg.name} index.d.${extension} is missing ${name}`);
      }
    }

    if (!declaration.startsWith(virtualReference)) {
      failures.push(`${pkg.name} index.d.${extension} is missing ${virtualReference}`);
    }
  }

  const virtualDeclaration = readPackedFile(tarball, "dist/virtual.d.ts");
  for (const moduleId of virtualModules) {
    if (!virtualDeclaration.includes(`declare module "${moduleId}"`)) {
      failures.push(`${pkg.name} virtual.d.ts is missing ${moduleId}`);
    }
  }
  if (!virtualDeclaration.includes('import("@ox-content/vite-plugin").CollectionEntry')) {
    failures.push(`${pkg.name} virtual.d.ts does not reference public collection types`);
  }

  for (const mode of ["bundler", "nodenext", "node16"]) {
    checkVirtualCollectionsConsumer({ pkg, tarball, packDir, failures, mode });
  }
  for (const skipLibCheck of [false, true]) {
    checkRootCustomHostConsumer({ pkg, tarball, packDir, failures, skipLibCheck });
  }
}

function checkRootCustomHostConsumer({ pkg, tarball, packDir, failures, skipLibCheck }) {
  const consumerRoot = prepareVitePluginConsumer({
    pkg,
    tarball,
    packDir,
    name: `vite-plugin-root-custom-host-${skipLibCheck ? "skip" : "strict"}-`,
  });

  writeFileSync(join(consumerRoot, "package.json"), JSON.stringify({ type: "module" }));
  writeFileSync(join(consumerRoot, "root-custom-host-fixture.ts"), rootCustomHostFixture());
  writeFileSync(
    join(consumerRoot, "tsconfig.json"),
    tsconfig("bundler", ["root-custom-host-fixture.ts"], skipLibCheck),
  );

  const result = spawnSync(tscBin, ["-p", join(consumerRoot, "tsconfig.json")], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    const mode = skipLibCheck ? "skipLibCheck" : "strict";
    failures.push(
      `${pkg.name} root custom-host ${mode} consumer failed:\n${result.stdout}${result.stderr}`,
    );
  }
}

function checkVirtualCollectionsConsumer({ pkg, tarball, packDir, failures, mode }) {
  const consumerRoot = prepareVitePluginConsumer({
    pkg,
    tarball,
    packDir,
    name: `vite-plugin-virtual-${mode}-`,
  });

  writeFileSync(join(consumerRoot, "package.json"), JSON.stringify({ type: "module" }));
  writeFileSync(join(consumerRoot, "collections-fixture.ts"), collectionsFixture());
  writeFileSync(join(consumerRoot, "collections-fixture.cts"), cjsCollectionsFixture());
  writeFileSync(join(consumerRoot, "tsconfig.json"), tsconfig(mode, undefined, true));

  const result = spawnSync(tscBin, ["-p", join(consumerRoot, "tsconfig.json")], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    failures.push(
      `${pkg.name} virtual collections ${mode} consumer failed:\n${result.stdout}${result.stderr}`,
    );
  }
}

function prepareVitePluginConsumer({ pkg, tarball, packDir, name }) {
  const consumerRoot = mkdtempSync(join(packDir, name));
  const packageRoot = join(consumerRoot, "node_modules", "@ox-content", "vite-plugin");
  mkdirSync(packageRoot, { recursive: true });

  const extract = spawnSync("tar", ["-xzf", tarball, "-C", packageRoot, "--strip-components=1"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (extract.error) throw extract.error;
  if (extract.status !== 0) {
    throw new Error(extract.stderr || `Failed to extract ${pkg.name} into ${consumerRoot}`);
  }

  for (const dependency of [
    "@types/node",
    "@ox-content/napi",
    "@standard-schema/spec",
    "minimatch",
    "yaml",
    "glob",
    "rehype-parse",
    "rehype-stringify",
    "unified",
    "vite",
  ]) {
    linkPackageDependency(consumerRoot, "npm/vite-plugin-ox-content", dependency);
  }

  return consumerRoot;
}

function linkPackageDependency(consumerRoot, packageDir, dependency) {
  const source = resolve(root, packageDir, "node_modules", ...dependency.split("/"));
  if (!existsSync(source)) return;
  const target = join(consumerRoot, "node_modules", ...dependency.split("/"));
  mkdirSync(dirname(target), { recursive: true });
  try {
    symlinkSync(source, target, "junction");
  } catch (error) {
    if (error?.code !== "EEXIST") throw error;
  }
}

function collectionsFixture() {
  return [
    'import api, { collectionNames, collections, getCollection, queryCollection } from "virtual:ox-content/collections";',
    'import { planCollectionAssetsFromDocuments } from "@ox-content/vite-plugin";',
    'import type { CollectionAssetDocumentDiagnostic, CollectionEntry, CollectionQueryBuilder, PlanCollectionAssetsFromDocumentsInput } from "@ox-content/vite-plugin";',
    "",
    "interface PostEntry extends CollectionEntry {",
    "  title: string;",
    "  frontmatter: { draft?: boolean; rank: number };",
    "}",
    "",
    'const rows: CollectionEntry[] = collections.content ?? getCollection("content");',
    "const names: string[] = collectionNames;",
    'const builder: CollectionQueryBuilder<PostEntry> = queryCollection<PostEntry>("blog")',
    '  .path("/posts/hello")',
    '  .where("frontmatter.rank", ">=", 1);',
    'const selected = builder.select("title", "frontmatter");',
    "async function useTypedQuery(): Promise<number> {",
    "  const first = await selected.first();",
    "  return first?.frontmatter.rank ?? 0;",
    "}",
    "async function useDocumentAssets(): Promise<CollectionAssetDocumentDiagnostic[]> {",
    "  const input: PlanCollectionAssetsFromDocumentsInput = {",
    '    contentRoot: "content",',
    '    documents: [{ documentPath: "content/posts/hello.md", pagePath: "/posts/hello" }],',
    "    publicPath: (reference) => [reference.publicPath],",
    "  };",
    "  const result = await planCollectionAssetsFromDocuments(input);",
    "  return result.diagnostics;",
    "}",
    "",
    "void api;",
    "void names;",
    "void rows;",
    "void useTypedQuery;",
    "void useDocumentAssets;",
  ].join("\n");
}

function cjsCollectionsFixture() {
  return [
    'import collectionsApi = require("virtual:ox-content/collections");',
    'import vitePlugin = require("@ox-content/vite-plugin");',
    'type CollectionAssetDocumentDiagnostic = import("@ox-content/vite-plugin").CollectionAssetDocumentDiagnostic;',
    'type CollectionEntry = import("@ox-content/vite-plugin").CollectionEntry;',
    'type CollectionQueryBuilder<T extends CollectionEntry = CollectionEntry> = import("@ox-content/vite-plugin").CollectionQueryBuilder<T>;',
    'type PlanCollectionAssetsFromDocumentsInput = import("@ox-content/vite-plugin").PlanCollectionAssetsFromDocumentsInput;',
    "",
    "interface PostEntry extends CollectionEntry {",
    "  title: string;",
    "  frontmatter: { draft?: boolean; rank: number };",
    "}",
    "",
    "const names: string[] = collectionsApi.collectionNames;",
    'const builder: CollectionQueryBuilder<PostEntry> = collectionsApi.queryCollection<PostEntry>("blog");',
    "async function useTypedQuery(): Promise<boolean> {",
    '  const entries = await builder.where("frontmatter.draft", false).all();',
    "  return entries.some((entry) => entry.frontmatter.draft === false);",
    "}",
    "async function useDocumentAssets(): Promise<CollectionAssetDocumentDiagnostic[]> {",
    "  const input: PlanCollectionAssetsFromDocumentsInput = {",
    '    documents: [{ documentPath: "content/posts/hello.md", pagePath: "/posts/hello" }],',
    "  };",
    "  const result = await vitePlugin.planCollectionAssetsFromDocuments(input);",
    "  return result.diagnostics;",
    "}",
    "",
    "void collectionsApi.default;",
    "void names;",
    "void useTypedQuery;",
    "void useDocumentAssets;",
  ].join("\n");
}

function rootCustomHostFixture() {
  return [
    'import { createOxContentCustomHostPlugin, oxContentCustomHost, planCollectionAssets } from "@ox-content/vite-plugin";',
    'import { createOxContentCustomHostPlugin as createCustomHostSubpath } from "@ox-content/vite-plugin/custom-host";',
    'import type { CollectionAssetInput, CollectionAssetManifest, CollectionAssetManifestEntry, OxContentCustomHostAssetsContext, OxContentCustomHostOptions, OxContentCustomHostSsrStylesheetsResult, PlanCollectionAssetsInput } from "@ox-content/vite-plugin";',
    'import type { OxContentCustomHostOptions as SubpathOptions } from "@ox-content/vite-plugin/custom-host";',
    "",
    "type IsAny<T> = 0 extends 1 & T ? true : false;",
    "type ExpectFalse<T extends false> = T;",
    "type ManifestIsNotAny = ExpectFalse<IsAny<CollectionAssetManifest>>;",
    "type ContextIsNotAny = ExpectFalse<IsAny<OxContentCustomHostAssetsContext>>;",
    "type ManifestEntry = CollectionAssetManifest['assets'][number];",
    "type PublicPathsAreStrings = ManifestEntry['publicPaths'][number] extends string ? true : never;",
    "",
    "const options: OxContentCustomHostOptions = {",
    "  host: {",
    "    routes: () => [],",
    "    outputs: () => ({}),",
    "  },",
    "};",
    "const subpathOptions: SubpathOptions = options;",
    "const input: CollectionAssetInput = { sourcePath: 'content/page.md', publicPath: '/page.md' };",
    "const planInput: PlanCollectionAssetsInput = { assets: [input] };",
    "const entry: CollectionAssetManifestEntry = {",
    "  sourcePath: '/workspace/content/page.md',",
    "  publicPaths: ['/page.md'],",
    "  contentPath: '/assets/content/page.md',",
    "};",
    "const manifest: CollectionAssetManifest = { assets: [entry] };",
    "declare const assets: OxContentCustomHostAssetsContext;",
    "declare const stylesheets: OxContentCustomHostSsrStylesheetsResult;",
    "async function usePublicTypes(): Promise<string[]> {",
    "  const planned = await planCollectionAssets(planInput);",
    "  const customManifest = await assets.collectionManifest();",
    "  return [",
    "    ...manifest.assets.flatMap((asset) => asset.publicPaths),",
    "    ...planned.assets.flatMap((asset) => asset.publicPaths),",
    "    ...(customManifest?.assets.flatMap((asset) => asset.publicPaths) ?? []),",
    "    ...stylesheets.stylesheets.map((stylesheet) => stylesheet.href),",
    "  ];",
    "}",
    "",
    "void createOxContentCustomHostPlugin(options);",
    "void oxContentCustomHost(options);",
    "void createCustomHostSubpath(subpathOptions);",
    "void usePublicTypes;",
    "void (undefined as unknown as ManifestIsNotAny);",
    "void (undefined as unknown as ContextIsNotAny);",
    "void (undefined as unknown as PublicPathsAreStrings);",
  ].join("\n");
}

function tsconfig(mode, files, skipLibCheck) {
  const compilerOptions =
    mode === "bundler"
      ? {
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "Bundler",
          lib: ["ES2022", "DOM"],
          strict: true,
          skipLibCheck,
          noEmit: true,
          types: ["@ox-content/vite-plugin"],
        }
      : {
          target: "ES2022",
          module: mode === "nodenext" ? "NodeNext" : "Node16",
          moduleResolution: mode === "nodenext" ? "NodeNext" : "Node16",
          lib: ["ES2022", "DOM"],
          strict: true,
          skipLibCheck,
          noEmit: true,
          types: ["@ox-content/vite-plugin"],
        };

  return JSON.stringify(
    {
      compilerOptions,
      files:
        files ??
        (mode === "bundler"
          ? ["collections-fixture.ts"]
          : ["collections-fixture.ts", "collections-fixture.cts"]),
    },
    null,
    2,
  );
}
