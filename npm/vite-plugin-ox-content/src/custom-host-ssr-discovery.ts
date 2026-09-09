import * as fs from "node:fs/promises";
import * as path from "node:path";
import { glob } from "glob";
import type { CustomHostSsrRollupContext } from "./custom-host-ssr-stylesheets";
import {
  cleanModulePath,
  dynamicDiagnostic,
  fileFromModuleId,
  hasGlobSyntax,
  isFrameworkStyleRoot,
  isLocalSpecifier,
  isSourceFile,
  isStyleFile,
  isWithinRoot,
  moduleIdCandidates,
  normalizeFilePath,
  outsideRootDiagnostic,
  parseImports,
  publicModuleId,
  unresolvedDiagnostic,
  unique,
} from "./custom-host-ssr-imports";
import { ssrStylesheetEntryName } from "./custom-host-ssr-build-stylesheets";
import type {
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetDiagnostic,
} from "./custom-host-types";

type CssRecord = {
  moduleId: string;
  file: string;
};

export type RootRecord = {
  moduleId: string;
  file: string;
  entryName: string;
  css: CssRecord[];
  dependencies: string[];
  diagnostics: OxContentCustomHostStylesheetDiagnostic[];
  referenceId?: string;
  stylesheets?: readonly OxContentCustomHostStylesheet[];
};

export async function expandConfiguredModules(
  modules: readonly string[],
  root: string,
): Promise<string[]> {
  const expanded: string[] = [];
  for (const moduleId of modules) {
    if (!hasGlobSyntax(moduleId)) {
      expanded.push(moduleId);
      continue;
    }
    expanded.push(...(await glob(moduleId, { absolute: true, cwd: root, nodir: true })));
  }
  return unique(expanded);
}

export async function discoverRoot(
  context: CustomHostSsrRollupContext,
  moduleId: string,
  root: string,
): Promise<RootRecord> {
  const resolved = await resolveModule(context, moduleId, undefined, root);
  const fallback = fileFromModuleId(moduleId, root) ?? path.resolve(root, moduleId);
  const file = resolved?.file ?? normalizeFilePath(fallback);
  const record: RootRecord = {
    moduleId: publicModuleId(file, root),
    file,
    entryName: ssrStylesheetEntryName(file, root),
    css: [],
    dependencies: [],
    diagnostics: resolved
      ? []
      : [
          {
            code: "missing-module",
            moduleId,
            message: `SSR stylesheet root "${moduleId}" could not be resolved.`,
          },
        ],
  };
  if (resolved) {
    await visitSource(context, resolved.file, root, record, new Set());
  }
  return record;
}

export function findRecord(
  records: Map<string, RootRecord>,
  moduleId: string,
  root: string | undefined,
): RootRecord | undefined {
  for (const candidate of moduleIdCandidates(moduleId, root)) {
    const record = records.get(candidate);
    if (record) {
      return record;
    }
  }
  return undefined;
}

async function visitSource(
  context: CustomHostSsrRollupContext,
  file: string,
  root: string,
  record: RootRecord,
  seen: Set<string>,
): Promise<void> {
  const clean = normalizeFilePath(file);
  if (seen.has(clean)) {
    return;
  }
  seen.add(clean);
  record.dependencies.push(clean);
  context.addWatchFile(clean);

  const source = await fs.readFile(clean, "utf8");
  if (isFrameworkStyleRoot(clean) && hasFrameworkStyleBlock(source)) {
    const moduleId = publicModuleId(clean, root);
    if (!record.css.some((entry) => entry.moduleId === moduleId)) {
      record.css.push({ moduleId, file: clean });
    }
  }
  for (const item of parseImports(source)) {
    if (!isLocalSpecifier(item.specifier)) {
      continue;
    }
    if (item.dynamic) {
      record.diagnostics.push(dynamicDiagnostic(record.moduleId, item.specifier, clean));
      continue;
    }
    const resolved = await resolveModule(context, item.specifier, clean, root);
    if (resolved?.kind === "outside-root") {
      record.diagnostics.push(outsideRootDiagnostic(record.moduleId, item.specifier, clean));
      continue;
    }
    if (!resolved) {
      record.diagnostics.push(unresolvedDiagnostic(record.moduleId, item.specifier, clean));
      continue;
    }
    if (resolved.kind === "style") {
      record.dependencies.push(resolved.file);
      context.addWatchFile(resolved.file);
      record.css.push({ moduleId: publicModuleId(resolved.file, root), file: resolved.file });
    } else {
      await visitSource(context, resolved.file, root, record, seen);
    }
  }
}

function hasFrameworkStyleBlock(source: string): boolean {
  return /<style(?:\s|>)/iu.test(source);
}

async function resolveModule(
  context: CustomHostSsrRollupContext,
  specifier: string,
  importer: string | undefined,
  root: string,
): Promise<{ file: string; kind: "outside-root" | "source" | "style" } | undefined> {
  const resolved = await context.resolve(specifier, importer, { skipSelf: true });
  const resolvedId = resolved?.id ?? specifier;
  const file = fileFromModuleId(resolvedId, root);
  const clean = cleanModulePath(resolvedId);
  if (!file && path.isAbsolute(clean) && isStyleFile(clean)) {
    return { file: normalizeFilePath(clean), kind: "outside-root" };
  }
  if (!file || !isWithinRoot(file, root) || !(await fileExists(file))) {
    return undefined;
  }
  if (isStyleFile(file)) {
    return { file, kind: "style" };
  }
  return isSourceFile(file) ? { file, kind: "source" } : undefined;
}

async function fileExists(file: string): Promise<boolean> {
  try {
    return (await fs.stat(file)).isFile();
  } catch {
    return false;
  }
}
