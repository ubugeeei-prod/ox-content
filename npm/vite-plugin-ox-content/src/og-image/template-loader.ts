import * as path from "node:path";
import type { OgImageTemplateFn, ResolvedOgImageOptions } from "./types";
import { getDefaultTemplate, getDefaultSatoriTemplate } from "./template";
import { resolveVueTemplate } from "./vue-template";
import { resolveSolidTemplate } from "./solid-template";
import { templateSources } from "./template-source";

/**
 * Resolves the template function from options.
 *
 * Dispatches by file extension:
 * - `.vue`  → Vue SFC (SSR via vue/server-renderer)
 * - `.svelte` → Svelte SFC (SSR via svelte/server)
 * - `.tsx`/`.jsx` → React Server Component (SSR via react-dom/server)
 * - others → TypeScript template (direct function export)
 */
export async function resolveTemplate(
  options: ResolvedOgImageOptions,
  root: string,
): Promise<OgImageTemplateFn> {
  if (!options.template) {
    return options.renderer !== "chromium" ? getDefaultSatoriTemplate() : getDefaultTemplate();
  }

  const templatePath = path.resolve(root, options.template);

  // Verify file exists
  const fs = await import("fs/promises");
  try {
    await fs.access(templatePath);
  } catch {
    throw new Error(`[ox-content:og-image] Template file not found: ${templatePath}`);
  }

  const ext = path.extname(templatePath).toLowerCase();

  let template: OgImageTemplateFn;
  let suffix = "";
  switch (ext) {
    case ".vue":
      template = await resolveVueTemplate(templatePath, options, root);
      suffix = "_vue";
      break;
    case ".svelte":
      template = await resolveSvelteTemplate(templatePath, root);
      suffix = "_svelte";
      break;
    case ".tsx":
    case ".jsx":
      template = await (options.jsxFramework === "solid"
        ? resolveSolidTemplate(templatePath, root)
        : resolveReactTemplate(templatePath, root));
      suffix = options.jsxFramework === "solid" ? "_solid" : "_react";
      break;
    default:
      template = await resolveTsTemplate(templatePath, options, root);
  }
  const compiled = await fs.readFile(
    path.join(root, ".cache", "og-images", `_template${suffix}.mjs`),
    "utf8",
  );
  templateSources.set(
    template,
    compiled + (ext === ".vue" ? (templateSources.get(template) ?? "") : ""),
  );
  return template;
}

/**
 * Matches this package and every subpath it exports.
 *
 * A template's natural runtime is whatever renders it, and for the
 * framework-less kinds that is this package: `renderToString`, `raw`, `when`
 * and `each` live at its root, and the JSX runtime under `./jsx-runtime`.
 * Inlining them instead drags the entire plugin — chokidar, fsevents and all
 * — into the template bundle, which is what made importing it fail outright.
 */
const OX_CONTENT_PACKAGE = /^@ox-content\/vite-plugin(\/.*)?$/;

/**
 * Whether `id` is a bare specifier, and so resolvable at runtime rather than
 * something the template bundle has to inline.
 *
 * Template bundles are written to `<root>/.cache/og-images/` and imported
 * from there, so Node resolves anything left external against the project's
 * own `node_modules`. Relative and absolute imports still bundle, which is
 * what a template actually needs — its own components travel with it.
 */
export function isBareSpecifier(id: string): boolean {
  if (id.startsWith(".") || id.startsWith("/") || id.startsWith("\0")) {
    return false;
  }
  // Windows drive letters and rolldown's virtual-module prefixes.
  return !/^[a-zA-Z]:[\\/]/.test(id);
}

/**
 * Rolldown input options for a `.ts` template bundle.
 *
 * A `.ts` template is the framework-less kind, so it has no single runtime to
 * externalize the way the `.vue`, `.svelte` and `.tsx` paths do — anything
 * from `node_modules` is better resolved at import time than inlined. Nothing
 * on this path has a compiler plugin, so nothing here needed bundling to be
 * loadable in the first place.
 */
export function tsTemplateBundleOptions(templatePath: string) {
  return {
    input: templatePath,
    platform: "node" as const,
    external: (id: string) => isBareSpecifier(id),
  };
}

/**
 * Resolves a plain TypeScript template (existing behavior).
 */
async function resolveTsTemplate(
  templatePath: string,
  options: ResolvedOgImageOptions,
  root: string,
): Promise<OgImageTemplateFn> {
  const fs = await import("fs/promises");
  const { rolldown } = await import("rolldown");
  const cacheDir = path.join(root, ".cache", "og-images");
  await fs.mkdir(cacheDir, { recursive: true });

  const outfile = path.join(cacheDir, "_template.mjs");

  const bundle = await rolldown(tsTemplateBundleOptions(templatePath));
  await bundle.write({
    file: outfile,
    format: "esm",
  });
  await bundle.close();

  const mod = await import(`${outfile}?t=${Date.now()}`);
  const templateFn = mod.default;

  if (typeof templateFn !== "function") {
    throw new Error(
      `[ox-content:og-image] Template must default-export a function: ${options.template}`,
    );
  }

  return templateFn as OgImageTemplateFn;
}

/**
 * Resolves a Svelte SFC template via SSR.
 *
 * Compiles the SFC with svelte/compiler (server mode + runes),
 * bundles with rolldown, then wraps with svelte/server render().
 */
async function resolveSvelteTemplate(
  templatePath: string,
  root: string,
): Promise<OgImageTemplateFn> {
  const fs = await import("fs/promises");
  const { rolldown } = await import("rolldown");
  const cacheDir = path.join(root, ".cache", "og-images");
  await fs.mkdir(cacheDir, { recursive: true });

  const outfile = path.join(cacheDir, "_template_svelte.mjs");

  const bundle = await rolldown({
    input: templatePath,
    platform: "node",
    external: [
      "svelte",
      "svelte/server",
      "svelte/internal",
      "svelte/internal/server",
      OX_CONTENT_PACKAGE,
    ],
    plugins: [createSvelteCompilerPlugin()],
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
      `[ox-content:og-image] Svelte template must have a default export: ${templatePath}`,
    );
  }

  // Import Svelte SSR utility
  const { render } = (await import("svelte/server")) as {
    render: (
      component: unknown,
      options: { props: Record<string, unknown> },
    ) => { body: string; head: string };
  };

  return async (props) => {
    const { body, head } = render(Component, { props });
    return head + body;
  };
}

/**
 * Creates a rolldown plugin that compiles Svelte SFCs using svelte/compiler.
 */
function createSvelteCompilerPlugin(): import("rolldown").Plugin {
  return {
    name: "ox-content-svelte-sfc",
    async transform(code, id) {
      if (!id.endsWith(".svelte")) return null;

      let svelteCompiler: typeof import("svelte/compiler");
      try {
        svelteCompiler = await import("svelte/compiler");
      } catch {
        throw new Error(
          "[ox-content:og-image] svelte is required for .svelte templates. " +
            "Install it with: pnpm add -D svelte",
        );
      }

      const result = svelteCompiler.compile(code, {
        generate: "server",
        css: "injected",
        runes: true,
        filename: id,
      });

      return { code: result.js.code };
    },
  };
}

/**
 * Resolves a React (.tsx/.jsx) template via SSR.
 *
 * Bundles with rolldown (JSX transform), then wraps with
 * react-dom/server renderToReadableStream for async Server Component support.
 */
async function resolveReactTemplate(
  templatePath: string,
  root: string,
): Promise<OgImageTemplateFn> {
  const fs = await import("fs/promises");
  const { rolldown } = await import("rolldown");
  const cacheDir = path.join(root, ".cache", "og-images");
  await fs.mkdir(cacheDir, { recursive: true });

  const outfile = path.join(cacheDir, "_template_react.mjs");

  const bundle = await rolldown({
    input: templatePath,
    platform: "node",
    external: [
      "react",
      "react/jsx-runtime",
      "react/jsx-dev-runtime",
      "react-dom",
      "react-dom/server",
      OX_CONTENT_PACKAGE,
    ],
    transform: {
      jsx: "react-jsx",
    },
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
      `[ox-content:og-image] React template must have a default export: ${templatePath}`,
    );
  }

  // Import React SSR utilities
  let React: typeof import("react");
  let ReactDOMServer: typeof import("react-dom/server");
  try {
    React = await import("react");
    ReactDOMServer = await import("react-dom/server");
  } catch {
    throw new Error(
      "[ox-content:og-image] react and react-dom are required for .tsx/.jsx templates. " +
        "Install them with: pnpm add -D react react-dom",
    );
  }

  return async (props) => {
    const element = React.createElement(Component, props);
    // Use renderToReadableStream for async Server Component support
    const stream = await ReactDOMServer.renderToReadableStream(element);
    const reader = stream.getReader();
    const chunks: Uint8Array[] = [];
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
    }
    const decoder = new TextDecoder();
    return (
      chunks.map((chunk) => decoder.decode(chunk, { stream: true })).join("") + decoder.decode()
    );
  };
}
