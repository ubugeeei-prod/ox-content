import type { ExtractedDocs, NavItem } from "./types";
import { toRustDocsModules } from "./docs";
import { importNapiModuleSync } from "./napi";

export interface GenerateNavMetadataOptions {
  basePath?: string;
  pathStrategy?: "flat" | "typedoc";
}

export function generateNavMetadata(
  docs: ExtractedDocs[],
  basePathOrOptions: string | GenerateNavMetadataOptions = "/api",
): NavItem[] {
  const options: GenerateNavMetadataOptions =
    typeof basePathOrOptions === "string"
      ? { basePath: basePathOrOptions }
      : basePathOrOptions;
  const basePath = options.basePath ?? "/api";
  const napi = importNapiModuleSync();

  if (options.pathStrategy === "typedoc") {
    return napi.generateDocsNavMetadataFromDocs(toRustDocsModules(docs), {
      basePath,
      pathStrategy: "typedoc",
    });
  }

  return napi.generateDocsNavMetadata(
    docs.map((doc) => doc.file),
    basePath,
  );
}

export function generateNavCode(navItems: NavItem[], exportName: string = "apiNav"): string {
  return importNapiModuleSync().generateDocsNavCode(navItems, exportName);
}
