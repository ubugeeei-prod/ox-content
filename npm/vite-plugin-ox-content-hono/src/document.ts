import { createHash } from "node:crypto";
import { renderMarkdown, type OxContentOptions } from "@ox-content/vite-plugin";
import {
  renderHtmlHost,
  toHtmlHostClientModuleId,
  type HtmlHostComponentsMap,
} from "@ox-content/vite-plugin/html-host";
import type { Frontmatter } from "./context.js";

/** Compile with the upstream parser and island resolver; generated modules are runtime-only. */
export async function compileHonoDocument(input: {
  source: string;
  documentPath: string;
  root: string;
  oxContent: OxContentOptions;
  components?: HtmlHostComponentsMap;
  selectFrontmatter?: (frontmatter: Frontmatter) => Frontmatter;
}) {
  const parsed = await renderMarkdown(input.source, input.documentPath, input.oxContent);
  const calls: string[] = [];
  const imports: string[] = [];
  const prefix = "ox-hono-" + createHash("sha256").update(input.source).digest("hex") + "-";
  // The resolver only needs an exported value to pass to the callback. Real
  // components remain ordinary Vite imports and execute at request/render time.
  const rendered = await renderHtmlHost({
    html: parsed.html,
    documentPath: input.documentPath,
    root: input.root,
    srcDir: input.oxContent.srcDir,
    imports: parsed.imports,
    components: input.components,
    resolveClientModule: (module) => toHtmlHostClientModuleId(module.serverModuleId, input.root),
    loadModule: async () =>
      new Proxy({}, { get: (_target, key) => (key === "then" ? undefined : () => null) }),
    renderComponent(_component, props, slot, context) {
      const index = calls.length;
      // Resolve the export from the same upstream module descriptor used for
      // browser registry discovery, including named MDX imports.
      imports.push(context.moduleId);
      calls.push(
        JSON.stringify({
          props,
          slot,
          moduleId: context.moduleId,
          name: context.component,
        }),
      );
      return `${prefix}${index}:end`;
    },
  });
  if (rendered.diagnostics.length) {
    throw new Error(rendered.diagnostics.map((d) => d.message).join("\n"));
  }
  const selected = input.selectFrontmatter?.(parsed.frontmatter) ?? {};
  const modules = [...new Set(imports)];
  const importCode = modules.map((id, i) => `import * as m${i} from ${JSON.stringify(id)};`);
  const expressions = calls.map((call) => {
    const { props, slot, moduleId, name } = JSON.parse(call);
    const descriptor = rendered.modules.find(
      (m) => m.serverModuleId === moduleId && m.name === name,
    )!;
    return `adapter.renderComponent(m${modules.indexOf(moduleId)}[${JSON.stringify(descriptor.exportName)}], ${JSON.stringify(props)}, ${JSON.stringify(slot) ?? "undefined"}, {})`;
  });
  return {
    parsed,
    code: [
      ...importCode,
      `import {jsx} from 'hono/jsx';`,
      `import {createHonoServerAdapter} from '@ox-content/vite-plugin-hono/server';`,
      `export const frontmatter = ${JSON.stringify(parsed.frontmatter)};`,
      `export const toc = ${JSON.stringify(parsed.toc)};`,
      `export default async function MarkdownContent(props = {}) {`,
      `const adapter = createHonoServerAdapter({frontmatter: ${JSON.stringify(selected)}});`,
      `const islands = await Promise.all([${expressions.join(",")}]);`,
      `let html = ${JSON.stringify(rendered.html)};`,
      ...expressions.map(
        (_, i) =>
          `html = html.replace(${JSON.stringify(`${prefix}${i}:end`)}, () => islands[${i}]);`,
      ),
      `return jsx('div', {class: props.class ?? 'ox-content', dangerouslySetInnerHTML: {__html: html}});`,
      `}`,
    ].join("\n"),
  };
}
