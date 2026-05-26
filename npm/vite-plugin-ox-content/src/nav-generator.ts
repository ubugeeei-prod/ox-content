import type { ExtractedDocs, NavItem } from "./types";
import { importNapiModuleSync } from "./napi";

export function generateNavMetadata(docs: ExtractedDocs[], basePath: string = "/api"): NavItem[] {
  return importNapiModuleSync().generateDocsNavMetadata(
    docs.map((doc) => doc.file),
    basePath,
  );
}

export function generateNavCode(navItems: NavItem[], exportName: string = "apiNav"): string {
  return importNapiModuleSync().generateDocsNavCode(navItems, exportName);
}
