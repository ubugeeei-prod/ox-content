import type { ResolvedOgImageOptions } from "./types";
import { loadOgFonts } from "./fonts";
import { inlinePublicImages } from "./assets";

/** Reuse Takumi's native renderer and its registered fonts throughout the batch. */
export async function createTakumiRenderer(options: ResolvedOgImageOptions, root: string) {
  const [{ Renderer }, fonts] = await Promise.all([
    import("@takumi-rs/core"),
    loadOgFonts(options, root),
  ]);
  const renderer = new Renderer();
  await Promise.all(fonts.map((font) => renderer.registerFont(font)));
  return async (source: string): Promise<Buffer> =>
    renderer.render(await inlinePublicImages(source, root), {
      width: options.width,
      height: options.height,
      format: "png",
    });
}
