/**
 * Combine document-local import resolution with MDX island discovery.
 */

import {
  resolveContentRootPath,
  resolveDocumentComponentImports,
  type DocumentImportDiagnostic,
  type ResolvedDocumentComponentImport,
  type ResolveDocumentComponentImportsInput,
} from "./document-imports";
import { discoverRegisteredMdxComponents, type ComponentRegistry } from "./mdx-islands";
import type { MdxImport } from "./types";

export interface DiscoverDocumentMdxIslandsInput {
  source: string;
  html?: string;
  components: ComponentRegistry;
  imports: readonly MdxImport[];
  documentPath: string;
  contentRoot?: string;
  srcDir?: string;
  root?: string;
}

export interface DiscoverDocumentMdxIslandsResult {
  usedComponents: string[];
  localBindings: Map<string, ResolvedDocumentComponentImport>;
  diagnostics: DocumentImportDiagnostic[];
}

export async function discoverDocumentMdxIslands(
  input: DiscoverDocumentMdxIslandsInput,
): Promise<DiscoverDocumentMdxIslandsResult> {
  const resolved = resolveDocumentComponentImports({
    imports: input.imports,
    documentPath: input.documentPath,
    contentRoot: input.contentRoot ?? resolveContentRootPath(input),
    srcDir: input.srcDir,
  } satisfies ResolveDocumentComponentImportsInput);
  const localBindings = new Map(
    resolved.bindings.map((binding) => [binding.localName, binding] as const),
  );
  const usedComponents = await discoverRegisteredMdxComponents({
    source: input.source,
    html: input.html,
    components: input.components,
    localNames: localBindings.keys(),
  });
  return {
    usedComponents,
    localBindings,
    diagnostics: resolved.diagnostics,
  };
}
