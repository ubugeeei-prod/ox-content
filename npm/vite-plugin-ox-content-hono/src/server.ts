import { Fragment, jsx } from "hono/jsx";
import { raw } from "hono/html";
import { FrontmatterContext, type Frontmatter } from "./context.js";
import { componentProps, islandContext } from "./shared.js";

/** Structural HTML-host connection without a compiler dependency in runtime types. */
export interface HonoServerAdapter {
  frameworkName: string;
  renderComponent(
    component: unknown,
    props: Record<string, unknown>,
    slotHtml: string | undefined,
    context: { component: string; moduleId: string; documentPath: string },
  ): Promise<string>;
}

export interface HonoServerOptions {
  /** JSON data intentionally exposed to each island; defaults to no metadata. */
  frontmatter?: Frontmatter;
}

/** A renderer callback only. No filesystem, Vite or Markdown engine at runtime. */
export function createHonoServerAdapter(options: HonoServerOptions = {}): HonoServerAdapter {
  // Round-trip once so server values match the browser's JSON values (e.g. dates).
  const json = JSON.stringify(options.frontmatter ?? {});
  const frontmatter: Frontmatter = JSON.parse(json);
  if (!frontmatter || Array.isArray(frontmatter) || typeof frontmatter !== "object") {
    throw new TypeError("frontmatter must serialize to a JSON object");
  }
  return {
    frameworkName: "Hono JSX",
    async renderComponent(component, props, slotHtml) {
      if (typeof component !== "function")
        throw new TypeError("Hono islands must export a function component");
      const safeJson = JSON.stringify(islandContext(frontmatter, props))
        .replace(/</g, "\\u003c")
        .replace(/>/g, "\\u003e")
        .replace(/&/g, "\\u0026");
      const children = slotHtml === undefined ? props.children : raw(slotHtml);
      const node = jsx(
        Fragment,
        null,
        jsx(
          FrontmatterContext.Provider,
          { value: frontmatter },
          jsx(component, { ...componentProps(props), children }),
        ),
        jsx("script", {
          type: "application/json",
          "data-ox-hono-context": "",
          dangerouslySetInnerHTML: { __html: safeJson },
        }),
      );
      return String(await node.toString());
    },
  };
}
