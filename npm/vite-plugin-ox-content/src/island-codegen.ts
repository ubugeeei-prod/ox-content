/**
 * Emit static component imports for framework Markdown modules.
 *
 * Document-local bindings win over the global `components` map for that file
 * only. Two documents that bind the same local name therefore emit different
 * specifiers and do not share one module id.
 */

import path from "node:path";
import type { ResolvedDocumentComponentImport } from "./document-imports";
import { stripViteQuery } from "./document-imports";

export type GlobalComponentMap = Readonly<Record<string, string>> | ReadonlyMap<string, string>;

export interface RenderIslandComponentImportsInput {
  globalComponents: GlobalComponentMap;
  localBindings?: ReadonlyMap<string, ResolvedDocumentComponentImport>;
  documentPath: string;
  root?: string;
}

export function renderIslandComponentImports(
  usedComponents: readonly string[],
  input: RenderIslandComponentImportsInput,
): string {
  const documentDir = path.dirname(stripViteQuery(input.documentPath));
  const root = input.root || process.cwd();

  return usedComponents
    .map((name) => {
      const local = input.localBindings?.get(name);
      if (local) {
        return renderLocalImport(local);
      }
      const componentPath = getGlobalComponentPath(input.globalComponents, name);
      if (!componentPath) return "";
      return renderGlobalImport(name, componentPath, documentDir, root);
    })
    .filter(Boolean)
    .join("\n");
}

function renderLocalImport(binding: ResolvedDocumentComponentImport): string {
  const specifier = binding.importPathRelativeToDocument.replace(/\\/g, "/");
  if (binding.kind === "default") {
    return `import ${binding.localName} from '${specifier}';`;
  }
  if (binding.imported === binding.localName) {
    return `import { ${binding.imported} } from '${specifier}';`;
  }
  return `import { ${binding.imported} as ${binding.localName} } from '${specifier}';`;
}

function renderGlobalImport(
  name: string,
  componentPath: string,
  documentDir: string,
  root: string,
): string {
  const absolutePath = path.resolve(root, componentPath.replace(/^\.\//, ""));
  const relativePath = path.relative(documentDir, absolutePath).replace(/\\/g, "/");
  const importPath = relativePath.startsWith(".") ? relativePath : `./${relativePath}`;
  return `import ${name} from '${importPath}';`;
}

function getGlobalComponentPath(components: GlobalComponentMap, name: string): string | undefined {
  if (isMapRegistry(components)) {
    return components.get(name);
  }
  return Object.prototype.hasOwnProperty.call(components, name) ? components[name] : undefined;
}

function isMapRegistry(value: GlobalComponentMap): value is ReadonlyMap<string, string> {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as ReadonlyMap<string, string>).has === "function" &&
    typeof (value as ReadonlyMap<string, string>).get === "function"
  );
}
