import * as path from "node:path";
import { normalizeFilePath, publicModuleId } from "./custom-host-ssr-imports";
import type {
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetsResult,
} from "./custom-host-types";
import { withBase } from "./custom-host-utils";

const VIRTUAL_SSR_STYLESHEET_PREFIX = "\0ox-content:custom-host-ssr-stylesheet:";

export type SsrStylesheetBuildRecord = {
  moduleId: string;
  stylesheets?: readonly OxContentCustomHostStylesheet[];
};

export type WritableSsrStylesheetBuildRecord = SsrStylesheetBuildRecord & {
  entryName: string;
  css: readonly { file: string; moduleId?: string }[];
  referenceId?: string;
};

export type SsrStylesheetOutputBundle = Record<string, SsrStylesheetBundleEntry>;

type SsrStylesheetBundleEntry =
  | {
      type: "asset";
      fileName?: string;
    }
  | {
      type: "chunk";
      fileName?: string;
      imports?: string[];
      viteMetadata?: { importedCss?: Set<string> | string[] };
    };

export function resolveBuildSsrStylesheetRecords(input: {
  records: readonly SsrStylesheetBuildRecord[];
  base?: string;
}): OxContentCustomHostStylesheetsResult {
  const stylesheets: OxContentCustomHostStylesheet[] = [];
  const seen = new Set<string>();

  for (const record of input.records) {
    for (const recordStylesheet of record.stylesheets ?? []) {
      const stylesheet = {
        ...recordStylesheet,
        href: withBase(input.base ?? "/", recordStylesheet.href),
      };
      if (!seen.has(stylesheet.href)) {
        seen.add(stylesheet.href);
        stylesheets.push(stylesheet);
      }
    }
  }

  return { stylesheets, diagnostics: [], dependencies: [] };
}

export function ssrStylesheetEntryName(file: string, root: string): string {
  return path.posix
    .relative(normalizeFilePath(root), normalizeFilePath(file))
    .replace(/\.[^.]+$/u, "")
    .replace(/[^a-z0-9_/-]+/giu, "-")
    .replace(/\//gu, "-");
}

export function ssrStylesheetVirtualId(entryName: string): string {
  return `${VIRTUAL_SSR_STYLESHEET_PREFIX}${entryName}.js`;
}

export function isSsrStylesheetVirtualId(id: string): boolean {
  return id.startsWith(VIRTUAL_SSR_STYLESHEET_PREFIX);
}

export function ssrStylesheetVirtualCss(
  record: { css: readonly { file: string; moduleId?: string }[] },
  root: string,
): string {
  return `${record.css
    .map((stylesheet, index) => stylesheetImport(stylesheet, root, index))
    .join("\n")}\n`;
}

function stylesheetImport(
  stylesheet: { file: string; moduleId?: string },
  root: string,
  index: number,
): string {
  const moduleId = JSON.stringify(stylesheet.moduleId ?? publicModuleId(stylesheet.file, root));
  if (isCssModuleFile(stylesheet.file)) {
    const binding = `__oxContentSsrCssModule${index}`;
    return `import ${binding} from ${moduleId};\nexport const ${binding}ClassNames = Object.values(${binding});`;
  }
  return `import ${moduleId};`;
}

function isCssModuleFile(file: string): boolean {
  return /\.module\.css(?:[?#].*)?$/u.test(file);
}

export function resolveSsrStylesheetBundleOutput(
  record: WritableSsrStylesheetBuildRecord,
  bundle: SsrStylesheetOutputBundle,
  getFileName: (referenceId: string) => string,
): OxContentCustomHostStylesheet[] {
  if (record.css.length === 0) {
    return [];
  }
  if (!record.referenceId) {
    return [];
  }
  return cssOutputPaths(bundle, getFileName(record.referenceId), record.entryName).map(
    (outputPath) => ({
      kind: "style",
      href: `/${outputPath}`,
      moduleId: record.moduleId,
      outputPath,
    }),
  );
}

function cssOutputPaths(
  bundle: SsrStylesheetOutputBundle,
  fileName: string,
  entryName: string,
): string[] {
  const paths: string[] = [];
  const seenCss = new Set<string>();
  const visiting = new Set<string>();
  const visited = new Set<string>();

  const addCss = (css: string) => {
    if (!seenCss.has(css)) {
      seenCss.add(css);
      paths.push(css);
    }
  };

  const visit = (chunkFileName: string) => {
    if (visiting.has(chunkFileName) || visited.has(chunkFileName)) {
      return;
    }
    const entry = bundle[chunkFileName];
    if (!entry) {
      return;
    }
    if (entry.type === "asset") {
      if (chunkFileName.endsWith(".css")) {
        addCss(chunkFileName);
      }
      return;
    }
    visiting.add(chunkFileName);
    for (const imported of entry.imports ?? []) {
      visit(imported);
    }
    for (const css of viteImportedCss(entry)) {
      addCss(css);
    }
    visiting.delete(chunkFileName);
    visited.add(chunkFileName);
  };

  visit(fileName);
  if (paths.length === 0) {
    for (const css of findNamedCssAssets(bundle, entryName)) {
      addCss(css);
    }
  }
  return paths;
}

function findNamedCssAssets(bundle: SsrStylesheetOutputBundle, entryName: string): string[] {
  const prefix = `assets/${entryName}-`;
  const matches: string[] = [];
  for (const [fileName, entry] of Object.entries(bundle)) {
    if (entry.type === "asset" && fileName.startsWith(prefix) && fileName.endsWith(".css")) {
      matches.push(fileName);
    }
  }
  return matches;
}

function viteImportedCss(chunk: Extract<SsrStylesheetBundleEntry, { type: "chunk" }>): string[] {
  const importedCss = chunk.viteMetadata?.importedCss;
  return importedCss ? Array.from(importedCss) : [];
}
