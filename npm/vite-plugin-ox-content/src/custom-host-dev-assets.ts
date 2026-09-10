import type { ViteDevServer } from "vite";
import type { CollectionAssetManifest } from "./collection-assets";
import { createAssetsContext } from "./custom-host-assets";
import {
  createCombinedDevModuleGraph,
  createDevImportResolver,
  createDevStylesheetContentResolver,
} from "./custom-host-dev-resolvers";
import type { CustomHostSsrStylesheetController } from "./custom-host-ssr-stylesheets";
import type { OxContentCustomHostAssetsContext, ResolvedThemeTokens } from "./custom-host-types";
import type { ResolvedOptions } from "./types";

interface CreateCustomHostDevAssetsContextInput {
  server: ViteDevServer;
  options: ResolvedOptions;
  outDir: string;
  themeTokens: ResolvedThemeTokens | undefined;
  root: string;
  collectionManifest: () => Promise<CollectionAssetManifest | undefined>;
  ssrStylesheets: CustomHostSsrStylesheetController;
}

export function createCustomHostDevAssetsContext({
  server,
  options,
  outDir,
  themeTokens,
  root,
  collectionManifest,
  ssrStylesheets,
}: CreateCustomHostDevAssetsContextInput): OxContentCustomHostAssetsContext {
  return createAssetsContext(
    options,
    outDir,
    undefined,
    themeTokens,
    createCombinedDevModuleGraph(server),
    root,
    collectionManifest,
    ssrStylesheets,
    createDevImportResolver(server),
    createDevStylesheetContentResolver(server, options.base),
  );
}
