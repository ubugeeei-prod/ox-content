import type { Frontmatter } from "./context.js";

export interface IslandContext {
  frontmatter: Frontmatter;
  load: "eager" | "idle" | "visible" | "media";
  media?: string;
}

export function componentProps(props: Record<string, unknown>) {
  return Object.fromEntries(Object.entries(props).filter(([key]) => !key.startsWith("client:")));
}

export function islandContext(
  frontmatter: Frontmatter,
  props: Record<string, unknown>,
): IslandContext {
  const directives = Object.entries(props).filter(
    ([key, value]) => key.startsWith("client:") && value !== false,
  );
  if (directives.length > 1)
    throw new TypeError("Use one client loading directive per Hono island");
  if (!directives.length) return { frontmatter, load: "eager" };
  const [name, value] = directives[0];
  const load = name.slice("client:".length);
  if (load === "media") {
    if (typeof value !== "string" || !value.trim())
      throw new TypeError("client:media requires a media query string");
    return { frontmatter, load, media: value };
  }
  if (!["load", "idle", "visible"].includes(load) || value !== true) {
    throw new TypeError(
      "Supported Hono directives: client:load, client:idle, client:visible, client:media",
    );
  }
  return {
    frontmatter,
    load: load === "load" ? "eager" : (load as "idle" | "visible"),
  };
}
