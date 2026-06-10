import type { DocsSortStrategy, ExtractedDocs, NavItem } from "./types";
import { toRustDocsModules } from "./docs";
import { importNapiModuleSync } from "./napi";

export interface GenerateNavMetadataOptions {
  /**
   * Route prefix used by generated navigation links.
   * @default '/api'
   */
  basePath?: string;

  /**
   * Generated Markdown output path strategy.
   * @default 'flat'
   */
  pathStrategy?: "flat" | "typedoc";

  /**
   * TypeDoc-style group order for nav groups.
   * @default undefined
   */
  groupOrder?: string[];

  /**
   * TypeDoc-style sort strategies applied to nav leaf entries.
   * @default undefined
   */
  sort?: DocsSortStrategy[];

  /**
   * Sort entry points alphabetically instead of preserving caller order.
   * @default true
   */
  sortEntryPoints?: boolean;

  /**
   * TypeDoc-style declaration kind ranking for nav groups.
   * @default undefined
   */
  kindSortOrder?: string[];
}

export function generateNavMetadata(
  docs: ExtractedDocs[],
  basePathOrOptions: string | GenerateNavMetadataOptions = "/api",
): NavItem[] {
  const options: GenerateNavMetadataOptions =
    typeof basePathOrOptions === "string" ? { basePath: basePathOrOptions } : basePathOrOptions;
  const basePath = options.basePath ?? "/api";
  const napi = importNapiModuleSync();

  if (options.pathStrategy === "typedoc") {
    return napi.generateDocsNavMetadataFromDocs(toRustDocsModules(docs), {
      basePath,
      pathStrategy: "typedoc",
      groupOrder: options.groupOrder,
      sort: options.sort,
      sortEntryPoints: options.sortEntryPoints,
      kindSortOrder: options.kindSortOrder,
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
