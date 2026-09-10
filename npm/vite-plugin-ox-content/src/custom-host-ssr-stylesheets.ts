import type { DocumentAssetManifest } from "./document-assets";
import {
  isSsrStylesheetVirtualId,
  resolveBuildSsrStylesheetRecords,
  resolveSsrStylesheetBundleOutput,
  type SsrStylesheetOutputBundle,
  ssrStylesheetVirtualCss,
  ssrStylesheetVirtualId,
} from "./custom-host-ssr-build-stylesheets";
import {
  discoverRoot,
  expandConfiguredModules,
  findRecord,
  type RootRecord,
} from "./custom-host-ssr-discovery";
import {
  resolveStaticDevSsrStylesheets,
  type CustomHostDevImportResolver,
} from "./custom-host-ssr-dev-stylesheets";
import { hasGlobSyntax } from "./custom-host-ssr-imports";
import {
  resolveCustomHostStylesheets,
  type CustomHostDevModuleGraph,
} from "./custom-host-stylesheets";
import type {
  OxContentCustomHostDependency,
  OxContentCustomHostSsrStylesheetDescriptor,
  OxContentCustomHostSsrStylesheetsInput,
  OxContentCustomHostSsrStylesheetsOptions,
  OxContentCustomHostSsrStylesheetsResult,
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetDiagnostic,
  OxContentCustomHostStylesheetsResult,
} from "./custom-host-types";

export interface CustomHostSsrStylesheetController {
  buildStart(context: CustomHostSsrRollupContext, root: string): Promise<void>;
  resolveId(id: string): string | undefined;
  load(id: string): string | undefined;
  writeBundle(
    bundle: SsrStylesheetOutputBundle,
    getFileName: (referenceId: string) => string,
  ): void;
  write(outDir: string): Promise<void>;
  resolve(input: ResolveCustomHostSsrStylesheetsInput): OxContentCustomHostSsrStylesheetsResult;
  dependencies(): OxContentCustomHostDependency[];
}

export interface CustomHostSsrRollupContext {
  resolve(
    id: string,
    importer?: string,
    options?: { skipSelf?: boolean },
  ): Promise<{ id: string; external?: boolean | "absolute" | "relative" } | null>;
  emitFile(file: { type: "chunk"; id: string; name?: string }): string;
  addWatchFile(id: string): void;
}

export interface ResolveCustomHostSsrStylesheetsInput extends OxContentCustomHostSsrStylesheetsInput {
  root?: string;
  manifest?: DocumentAssetManifest;
  moduleGraph?: CustomHostDevModuleGraph;
  resolveImport?: CustomHostDevImportResolver;
}

export function createCustomHostSsrStylesheetController(
  options: false | OxContentCustomHostSsrStylesheetsOptions | undefined,
): CustomHostSsrStylesheetController {
  let buildRoot: string | undefined;
  let buildRecords: RootRecord[] = [];
  let records = new Map<string, RootRecord>();
  let virtualModules = new Map<string, string>();
  const configuredModules = options === false ? [] : (options?.modules ?? []);
  let configured = false;

  return {
    async buildStart(context, root) {
      buildRoot = root;
      buildRecords = [];
      records = new Map();
      virtualModules = new Map();
      configured = configuredModules.length > 0;
      if (!configured) {
        return;
      }
      for (const moduleId of await expandConfiguredModules(configuredModules, root)) {
        const record = await discoverRoot(context, moduleId, root);
        if (record.css.length > 0) {
          const virtualId = ssrStylesheetVirtualId(record.entryName);
          virtualModules.set(virtualId, ssrStylesheetVirtualCss(record, root));
          record.referenceId = context.emitFile({
            type: "chunk",
            id: virtualId,
            name: record.entryName,
          });
        }
        buildRecords.push(record);
        records.set(record.file, record);
        records.set(record.moduleId, record);
      }
    },
    resolveId(id) {
      return isSsrStylesheetVirtualId(id) ? id : undefined;
    },
    load(id) {
      return virtualModules.get(id);
    },
    writeBundle(bundle, getFileName) {
      for (const record of buildRecords) {
        record.stylesheets = resolveSsrStylesheetBundleOutput(record, bundle, getFileName);
      }
    },
    async write(_outDir) {},
    resolve(input) {
      if (input.manifest || buildRoot) {
        return resolveBuild(input, records, configured, buildRoot);
      }
      if (input.moduleGraph) {
        return resolveDev(input);
      }
      return missingResolverResult(input.modules);
    },
    dependencies() {
      return configuredModules.map((moduleId) => ({
        path: moduleId,
        kind: hasGlobSyntax(moduleId) ? "glob" : "file",
      }));
    },
  };
}

function resolveBuild(
  input: ResolveCustomHostSsrStylesheetsInput,
  records: Map<string, RootRecord>,
  configured: boolean,
  root: string | undefined,
): OxContentCustomHostSsrStylesheetsResult {
  if (!configured) {
    return missingResolverResult(input.modules);
  }
  const descriptors: OxContentCustomHostSsrStylesheetDescriptor[] = [];
  const styleRecords: {
    moduleId: string;
    stylesheets?: readonly OxContentCustomHostStylesheet[];
  }[] = [];
  const diagnostics: OxContentCustomHostStylesheetDiagnostic[] = [];

  for (const moduleId of input.modules) {
    const record = findRecord(records, moduleId, root ?? input.root);
    if (!record) {
      diagnostics.push({
        code: "missing-module",
        moduleId,
        message: `SSR stylesheet module "${moduleId}" was not discovered during buildStart.`,
      });
      continue;
    }
    const buildRecord = { moduleId, stylesheets: record.stylesheets };
    styleRecords.push(buildRecord);
    diagnostics.push(...record.diagnostics);
    descriptors.push({
      moduleId,
      stylesheets: resolveBuildSsrStylesheetRecords({
        records: [buildRecord],
        base: input.base,
      }).stylesheets,
      dependencies: record.dependencies,
    });
  }

  const styles = resolveBuildSsrStylesheetRecords({
    records: styleRecords,
    base: input.base,
  });
  return mergeResult(styles, diagnostics, descriptors);
}

function resolveDev(
  input: ResolveCustomHostSsrStylesheetsInput,
): OxContentCustomHostSsrStylesheetsResult {
  const styles = resolveCustomHostStylesheets(input);
  if (styles.stylesheets.length > 0) {
    return withDevDescriptors(input, styles);
  }
  if (input.root) {
    const staticResult = resolveStaticDevSsrStylesheets({
      modules: input.modules,
      base: input.base,
      root: input.root,
      resolveImport: input.resolveImport,
    });
    if (staticResult) {
      return staticResult;
    }
  }
  return withDevDescriptors(input, styles);
}

function withDevDescriptors(
  input: ResolveCustomHostSsrStylesheetsInput,
  styles: OxContentCustomHostStylesheetsResult,
): OxContentCustomHostSsrStylesheetsResult {
  const descriptors = input.modules.map((moduleId): OxContentCustomHostSsrStylesheetDescriptor => {
    const result = resolveCustomHostStylesheets({ ...input, modules: [moduleId] });
    return {
      moduleId,
      stylesheets: result.stylesheets,
      dependencies: result.dependencies,
    };
  });
  return { ...styles, descriptors };
}

function mergeResult(
  styles: OxContentCustomHostStylesheetsResult,
  diagnostics: OxContentCustomHostStylesheetDiagnostic[],
  descriptors: OxContentCustomHostSsrStylesheetDescriptor[],
): OxContentCustomHostSsrStylesheetsResult {
  return { ...styles, diagnostics: [...diagnostics, ...styles.diagnostics], descriptors };
}

function missingResolverResult(
  modules: readonly string[],
): OxContentCustomHostSsrStylesheetsResult {
  return {
    stylesheets: [],
    dependencies: [],
    descriptors: [],
    diagnostics: modules.map((moduleId) => ({
      code: "missing-resolver",
      moduleId,
      message:
        `SSR stylesheet discovery is not configured for "${moduleId}". ` +
        "Add custom-host ssrStylesheets.modules entries for production builds.",
    })),
  };
}
