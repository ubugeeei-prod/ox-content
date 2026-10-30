import type { ResolvedOgImageOptions } from "./types";
import { loadOgFonts } from "./fonts";
import { inlinePublicImages } from "./assets";

/** Load fonts and renderer modules once per batch, rather than once per card. */
export async function createSatoriRenderer(options: ResolvedOgImageOptions, root: string) {
  const [fonts, { Resvg }, { default: satori }, { html }] = await Promise.all([
    loadOgFonts(options, root),
    import("@resvg/resvg-js"),
    import("satori"),
    import("satori-html"),
  ]);
  return async (source: string): Promise<Buffer> => {
    const vnode = html(await inlinePublicImages(source, root)) as unknown as Parameters<
      typeof satori
    >[0];
    normalizeImageSizes(vnode);
    const svg = await satori(vnode, { width: options.width, height: options.height, fonts });
    return new Resvg(svg, { fitTo: { mode: "original" }, font: { loadSystemFonts: false } })
      .render()
      .asPng();
  };
}

// HTML attributes are strings; Satori expects numeric image dimensions.
function normalizeImageSizes(node: unknown): void {
  if (!node || typeof node !== "object") return;
  const { type, props } = node as { type?: string; props?: Record<string, unknown> };
  if (!props) return;
  if (type === "img")
    for (const key of ["width", "height"]) {
      if (typeof props[key] === "string" && /^\d+$/.test(props[key]))
        props[key] = Number(props[key]);
    }
  for (const child of Array.isArray(props.children) ? props.children : [props.children])
    normalizeImageSizes(child);
}
