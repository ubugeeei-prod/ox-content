import type { Plugin, ResolvedConfig } from "vite";
import { LOAD_ENV } from "./custom-host-constants";
import { runCustomHostBuild } from "./custom-host-build";
import { configureDevServer } from "./custom-host-dev";
import { resolveThemeTokens } from "./custom-host-assets";
import type { OxContentCustomHostOptions } from "./custom-host-types";
import { resolveOptions } from "./resolve-options";
import type { OxContentOptions } from "./types";

export type {
  MaybePromise,
  OxContentCustomHostAssetsContext,
  OxContentCustomHostBaseContext,
  OxContentCustomHostBuildOptions,
  OxContentCustomHostDevOptions,
  OxContentCustomHostModule,
  OxContentCustomHostNotFoundContext,
  OxContentCustomHostOptions,
  OxContentCustomHostRenderContext,
  OxContentCustomHostRenderResult,
  OxContentCustomHostRoute,
  OxContentCustomHostRoutesContext,
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetDiagnostic,
  OxContentCustomHostStylesheetsInput,
  OxContentCustomHostStylesheetsResult,
  OxContentCustomHostThemeTokensOptions,
} from "./custom-host-types";

export function createOxContentCustomHostPlugin(input: OxContentCustomHostOptions): Plugin {
  const oxContentOptions = customHostOxContentOptions(input.oxContent ?? {});
  const resolvedOptions = resolveOptions(oxContentOptions);
  const themeTokens = resolveThemeTokens(input.themeTokens, resolvedOptions.base);
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
      configureDevServer(server, input, resolvedOptions, themeTokens);
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
      );
      await buildRun;
    },
  };
}

export function customHostOxContentOptions(options: OxContentOptions = {}): OxContentOptions {
  const ssg = options.ssg;
  const disabledSsg =
    ssg && typeof ssg === "object" ? { ...ssg, enabled: false } : { enabled: false };
  return { ...options, ssg: disabledSsg };
}
