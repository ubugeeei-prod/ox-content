import type { Plugin, ResolvedConfig } from "vite";
import { LOAD_ENV } from "./custom-host-constants";
import { runCustomHostBuild } from "./custom-host-build";
import { configureDevServer } from "./custom-host-dev";
import { resolveThemeTokens } from "./custom-host-assets";
import { createCustomHostSsrStylesheetController } from "./custom-host-ssr-stylesheets";
import type { OxContentCustomHostOptions } from "./custom-host-types";
import { resolveOptions } from "./resolve-options";
import { customHostOxContentOptions } from "./custom-host-options";

export { customHostOxContentOptions } from "./custom-host-options";

export type {
  OxContentCustomHostMarkdownInput,
  OxContentCustomHostMarkdownRenderContext,
  OxContentCustomHostMarkdownRenderer,
  OxContentCustomHostMarkdownRenderResult,
  OxContentCustomHostMarkdownResult,
} from "./custom-host-markdown";

export type {
  MaybePromise,
  OxContentCustomHostAssetsContext,
  OxContentCustomHostBaseContext,
  OxContentCustomHostBuildOptions,
  OxContentCustomHostCollectionAssetsContext,
  OxContentCustomHostCollectionAssetsOptions,
  OxContentCustomHostDependency,
  OxContentCustomHostDependencyDescriptor,
  OxContentCustomHostDevOptions,
  OxContentCustomHostMemo,
  OxContentCustomHostModule,
  OxContentCustomHostNotFoundContext,
  OxContentCustomHostOptions,
  OxContentCustomHostOutputData,
  OxContentCustomHostOutputsContext,
  OxContentCustomHostRenderContext,
  OxContentCustomHostRenderResult,
  OxContentCustomHostRoute,
  OxContentCustomHostRoutesContext,
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetContent,
  OxContentCustomHostStylesheetContentDiagnostic,
  OxContentCustomHostStylesheetContentInput,
  OxContentCustomHostStylesheetContentResult,
  OxContentCustomHostStylesheetDiagnostic,
  OxContentCustomHostSsrStylesheetDescriptor,
  OxContentCustomHostSsrStylesheetsInput,
  OxContentCustomHostSsrStylesheetsOptions,
  OxContentCustomHostSsrStylesheetsResult,
  OxContentCustomHostStylesheetsInput,
  OxContentCustomHostStylesheetsResult,
  OxContentCustomHostThemeTokensOptions,
} from "./custom-host-types";

export function createOxContentCustomHostPlugin(input: OxContentCustomHostOptions): Plugin {
  const oxContentOptions = customHostOxContentOptions(input.oxContent ?? {});
  const resolvedOptions = resolveOptions(oxContentOptions);
  const themeTokens = resolveThemeTokens(input.themeTokens, resolvedOptions.base);
  const ssrStylesheets = createCustomHostSsrStylesheetController(input.ssrStylesheets);
  let command: "build" | "serve" | undefined;
  let config: ResolvedConfig | undefined;
  let buildRun: Promise<void> | undefined;

  return {
    name: "ox-content:custom-host",
    enforce: "post",

    config(_config, env) {
      command = env.command;
      if (env.command === "build") {
        return { build: { manifest: true } };
      }
      return undefined;
    },

    configResolved(resolvedConfig) {
      config = resolvedConfig;
    },

    configureServer(server) {
      if (process.env[LOAD_ENV] === "1" || input.dev?.enabled === false) {
        return;
      }
      configureDevServer(
        server,
        input,
        resolvedOptions,
        themeTokens,
        oxContentOptions,
        ssrStylesheets,
      );
    },

    async buildStart() {
      if (!config || command !== "build" || input.build?.enabled === false) {
        return;
      }
      if (config.mode === "test" && input.build?.runInTest !== true) {
        return;
      }
      await ssrStylesheets.buildStart(this, config.root);
    },

    resolveId(id) {
      return ssrStylesheets.resolveId(id);
    },

    load(id) {
      return ssrStylesheets.load(id);
    },

    writeBundle(_options, bundle) {
      ssrStylesheets.writeBundle(bundle, (referenceId) => this.getFileName(referenceId));
    },

    async closeBundle() {
      if (!config || command !== "build" || input.build?.enabled === false) {
        return;
      }
      if (config.mode === "test" && input.build?.runInTest !== true) {
        return;
      }
      buildRun ??= runCustomHostBuild(
        config,
        input,
        resolvedOptions,
        themeTokens,
        oxContentOptions,
        ssrStylesheets,
      );
      await buildRun;
    },
  };
}
