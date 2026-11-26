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

// Binding names are emitted as bare identifiers. MDX string import names
// (`import { "a-b" as C }`) are emitted as string literals instead.
const JS_IDENTIFIER = /^[$_\p{ID_Start}][$\u200c\u200d\p{ID_Continue}]*$/u;

function renderLocalImport(binding: ResolvedDocumentComponentImport): string {
  if (!JS_IDENTIFIER.test(binding.localName)) return "";
  const specifier = quoteModuleString(binding.importPathRelativeToDocument.replace(/\\/g, "/"));
  if (binding.kind === "default") {
    return `import ${binding.localName} from ${specifier};`;
  }
  if (binding.imported === binding.localName) {
    return `import { ${binding.imported} } from ${specifier};`;
  }
  const imported = JS_IDENTIFIER.test(binding.imported)
    ? binding.imported
    : quoteModuleString(binding.imported);
  return `import { ${imported} as ${binding.localName} } from ${specifier};`;
}

function renderGlobalImport(
  name: string,
  componentPath: string,
  documentDir: string,
  root: string,
): string {
  if (!JS_IDENTIFIER.test(name)) return "";
  const absolutePath = path.resolve(root, componentPath.replace(/^\.\//, ""));
  const relativePath = path.relative(documentDir, absolutePath).replace(/\\/g, "/");
  const importPath = relativePath.startsWith(".") ? relativePath : `./${relativePath}`;
  return `import ${name} from ${quoteModuleString(importPath)};`;
}

/** Single-quoted JS string literal; the quote style matches earlier output. */
function quoteModuleString(value: string): string {
  const escaped = value.replace(/[\\'\n\r\u2028\u2029]/g, (ch) => {
    switch (ch) {
      case "\\":
        return "\\\\";
      case "'":
        return "\\'";
      case "\n":
        return "\\n";
      case "\r":
        return "\\r";
      default:
        return `\\u${ch.charCodeAt(0).toString(16)}`;
    }
  });
  return `'${escaped}'`;
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
