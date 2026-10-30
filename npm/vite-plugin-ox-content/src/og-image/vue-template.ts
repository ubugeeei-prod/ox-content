import * as path from "node:path";
import type { OgImageTemplateFn, ResolvedOgImageOptions } from "./types";
import { templateSources } from "./template-source";

const OX_CONTENT_PACKAGE = /^@ox-content\/vite-plugin(\/.*)?$/;

/**
 * Resolves a Vue SFC template via SSR.
 *
 * Compiles the SFC with @vue/compiler-sfc (or @vizejs/vite-plugin),
 * bundles with rolldown, then wraps with createSSRApp + renderToString.
 */
export async function resolveVueTemplate(
  templatePath: string,
  options: ResolvedOgImageOptions,
  root: string,
): Promise<OgImageTemplateFn> {
  const fs = await import("fs/promises");
  const { rolldown } = await import("rolldown");
  const cacheDir = path.join(root, ".cache", "og-images");
  await fs.mkdir(cacheDir, { recursive: true });

  const outfile = path.join(cacheDir, "_template_vue.mjs");
  const styles: string[] = [];

  const plugins =
    options.vuePlugin === "vizejs" ? await getVizejsPlugin() : [createVueCompilerPlugin(styles)];

  const bundle = await rolldown({
    input: templatePath,
    platform: "node",
    external: ["vue", "vue/server-renderer", OX_CONTENT_PACKAGE],
    plugins,
  });
  await bundle.write({
    file: outfile,
    format: "esm",
  });
  await bundle.close();

  const mod = await import(`${outfile}?t=${Date.now()}`);
  const Component = mod.default;

  if (!Component) {
    throw new Error(
      `[ox-content:og-image] Vue template must have a default export: ${templatePath}`,
    );
  }

  if (options.vuePlugin === "vizejs" && !mod.__vize_css__) {
    const { parse } = await import("@vue/compiler-sfc");
    const { descriptor } = parse(await fs.readFile(templatePath, "utf-8"));
    styles.push(...descriptor.styles.map((style) => style.content));
  }
  const extractedCss = [mod.__vize_css__ ?? "", ...styles].join("\n");

  // Import Vue SSR utilities
  const { createSSRApp } = await import("vue");
  const { renderToString } = await import("vue/server-renderer");

  const template: OgImageTemplateFn = async (props) => {
    const app = createSSRApp(Component, props);
    const html = await renderToString(app);
    if (extractedCss) {
      return `<style>${extractedCss}</style>${html}`;
    }
    return html;
  };
  templateSources.set(template, extractedCss);
  return template;
}

/**
 * Creates a rolldown plugin that compiles Vue SFCs using @vue/compiler-sfc.
 */
function createVueCompilerPlugin(styles: string[]): import("rolldown").Plugin {
  return {
    name: "ox-content-vue-sfc",
    async transform(code, id) {
      if (!id.endsWith(".vue")) return null;

      let compilerSfc: typeof import("@vue/compiler-sfc");
      try {
        compilerSfc = await import("@vue/compiler-sfc");
      } catch {
        throw new Error(
          "[ox-content:og-image] @vue/compiler-sfc is required for .vue templates. " +
            "Install it with: pnpm add -D @vue/compiler-sfc",
        );
      }

      const { descriptor } = compilerSfc.parse(code, { filename: id });
      for (const style of descriptor.styles) {
        const compiled = await compilerSfc.compileStyleAsync({
          source: style.content,
          filename: id,
          id,
          scoped: false,
        });
        if (compiled.errors.length)
          throw new AggregateError(compiled.errors, "OG template CSS failed: " + id);
        styles.push(compiled.code);
      }

      // Compile <script setup> or <script>
      let scriptCode: string;
      if (descriptor.scriptSetup || descriptor.script) {
        const compiled = compilerSfc.compileScript(descriptor, {
          id,
          inlineTemplate: true,
        });
        scriptCode = compiled.content;
      } else {
        // Template-only SFC: compile template separately
        if (!descriptor.template) {
          throw new Error(
            `[ox-content:og-image] Vue SFC must have a <template> or <script>: ${id}`,
          );
        }
        const templateResult = compilerSfc.compileTemplate({
          source: descriptor.template.content,
          filename: id,
          id,
        });
        if (templateResult.errors.length > 0) {
          throw new Error(
            `[ox-content:og-image] Vue template compilation errors in ${id}: ${templateResult.errors.map(String).join(", ")}`,
          );
        }
        scriptCode = `${templateResult.code}\nexport default { render }`;
      }

      // Determine if the compiled output contains TypeScript
      const isTs = !!(descriptor.scriptSetup?.lang === "ts" || descriptor.script?.lang === "ts");

      return { code: scriptCode, moduleType: isTs ? "ts" : "js" };
    },
  };
}

/**
 * Loads @vizejs/vite-plugin as a rolldown plugin for Vue SFC compilation.
 */
async function getVizejsPlugin(): Promise<import("rolldown").Plugin[]> {
  try {
    const vizejs = await import("@vizejs/vite-plugin");
    const plugin = vizejs.default?.() ?? vizejs;
    return Array.isArray(plugin) ? plugin : [plugin];
  } catch {
    throw new Error(
      "[ox-content:og-image] @vizejs/vite-plugin is required when vuePlugin is 'vizejs'. " +
        "Install it with: pnpm add -D @vizejs/vite-plugin",
    );
  }
}
