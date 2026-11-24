import {
  createHtmlHostCollectionDocuments,
  type HtmlHostMarkdownMetadata,
} from "@ox-content/vite-plugin/html-host";
import type { OxContentCustomHostModule } from "@ox-content/vite-plugin/custom-host";
import { renderHonoMarkdown } from "@ox-content/vite-plugin-hono/vite";

const documents = createHtmlHostCollectionDocuments({
  oxContent: { srcDir: "content" },
});
export default {
  async routes(context) {
    const pages = await documents({
      root: context.root,
      command: context.mode,
      mode: context.mode === "build" ? "production" : "development",
    });
    return pages.map((document) => ({
      path: document.path === "/" ? "/" : document.path.replace(/\/?$/, "/"),
      inputPath: document.documentPath,
      dependencies: [document.documentPath],
      async render(ctx) {
        const page = await ctx.markdown.render<HtmlHostMarkdownMetadata>({
          source: document.source,
          documentPath: document.documentPath,
          renderHtml(markdown) {
            return renderHonoMarkdown({
              context: markdown,
              frontmatter: {
                title: markdown.transform.frontmatter.title ?? "",
              },
              documentAssets: {
                clientEntries: markdown.transform.components?.length ? ["src/client.ts"] : [],
              },
            });
          },
        });
        const head = page.metadata?.documentAssets?.headHtml ?? "";
        return {
          html: `<!doctype html><html><head>${head}</head><body><main>${page.html}</main></body></html>`,
          dependencies: page.dependencies,
        };
      },
    }));
  },
} satisfies OxContentCustomHostModule;
