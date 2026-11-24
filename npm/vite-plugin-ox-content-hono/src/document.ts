import { createHash } from "node:crypto";
import { renderMarkdown, type OxContentOptions } from "@ox-content/vite-plugin";
import {
  renderHtmlHost,
  toHtmlHostClientModuleId,
  type HtmlHostComponentsMap,
} from "@ox-content/vite-plugin/html-host";
import type { Frontmatter } from "./context.js";
import { splitNestedIslands } from "./document-islands.js";

function jsonExpression(value: unknown) {
  const json = JSON.stringify(value);
  return json === undefined ? "undefined" : `JSON.parse(${JSON.stringify(json)})`;
}

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
  const calls: { props: Record<string, unknown>; slot?: string; moduleId: string; name: string }[] =
    [];
  const prefix = "ox-hono-" + createHash("sha256").update(input.source).digest("hex") + "-";
  const separator = `<!--${prefix}fragment-->`;
  // The resolver only needs an exported value to pass to the callback. Real
  // components remain ordinary Vite imports and execute at request/render time.
  const rendered = await renderHtmlHost({
    html: splitNestedIslands(parsed.html, prefix).join(separator),
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
      calls.push({ props, slot, moduleId: context.moduleId, name: context.component });
      return `${prefix}island-${index}:end`;
    },
  });
  if (rendered.diagnostics.length) {
    throw new Error(rendered.diagnostics.map((d) => d.message).join("\n"));
  }
  const selected = input.selectFrontmatter?.(parsed.frontmatter) ?? {};
  const modules = [...new Set(calls.map((call) => call.moduleId))];
  const importCode = modules.map((id, i) => `import * as m${i} from ${JSON.stringify(id)};`);
  const expressions = calls.map(({ props, moduleId, name }) => {
    const descriptor = rendered.modules.find(
      (m) => m.serverModuleId === moduleId && m.name === name,
    )!;
    return `(slot) => adapter.renderComponent(m${modules.indexOf(moduleId)}[${JSON.stringify(descriptor.exportName)}], ${jsonExpression(props)}, slot, {})`;
  });
  const markers = new RegExp(
    `${prefix}island-(\\d+):end|<!--${prefix}slot-(\\d+)-->|&lt;!--${prefix}slot-(\\d+)--&gt;`,
    "g",
  );
  return {
    parsed,
    code: [
      ...importCode,
      `import {jsx} from 'hono/jsx';`,
      `import {createHonoServerAdapter} from '@ox-content/vite-plugin-hono/server';`,
      `export const frontmatter = ${jsonExpression(parsed.frontmatter)};`,
      `export const toc = ${jsonExpression(parsed.toc)};`,
      `export default async function MarkdownContent(props = {}) {`,
      `const adapter = createHonoServerAdapter({frontmatter: ${jsonExpression(selected)}});`,
      `const islands = [${expressions.join(",")}];`,
      `const slots = [${calls.map((call) => jsonExpression(call.slot)).join(",")}];`,
      `const fragments = ${jsonExpression(rendered.html.split(separator))};`,
      `const markers = ${markers};`,
      // Resolve descendants before invoking the parent. Also reconstruct the
      // escaped data-ox-content slot used for every fresh browser mount.
      `const islandOutputs = [];
const fragmentOutputs = [];
async function expand(html) {
  if (html === undefined) return undefined;
  const values = await Promise.all(Array.from(html.matchAll(markers), async ([, island, child, escaped]) => {
    if (island !== undefined) return islandOutputs[island] ??= expand(slots[island]).then(islands[island]);
    const index = child ?? escaped;
    const nested = await (fragmentOutputs[index] ??= expand(fragments[index]));
    return escaped === undefined ? nested : nested
      .replaceAll("&", "&amp;").replaceAll("'", "&#39;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
  }));
  let index = 0;
  return html.replace(markers, () => values[index++]);
}
const html = await expand(fragments[0]);`,
      `return jsx('div', {class: props.class ?? 'ox-content', dangerouslySetInnerHTML: {__html: html}});`,
      `}`,
    ].join("\n"),
  };
}
