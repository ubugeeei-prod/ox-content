import { defineConfig } from "vite-plus";
import oxContent, {
  defineMdastPlugin,
  type MdastRoot,
  type OxContentPlugin,
} from "@ox-content/unplugin/vite";

const annotateHeadings = defineMdastPlugin("annotate-headings", (tree, context) => {
  const rawBadge = context.frontmatter.badge;
  const badge =
    rawBadge === undefined || rawBadge === null
      ? "mdast bridge"
      : typeof rawBadge === "string"
        ? rawBadge
        : JSON.stringify(rawBadge);

  for (const node of tree.children) {
    if (node.type !== "heading" || node.depth !== 1 || !Array.isArray(node.children)) {
      continue;
    }

    node.children.push({
      type: "text",
      value: ` [${badge}]`,
    });
    break;
  }
});

function remarkExposeFrontmatter() {
  return (tree: MdastRoot, file: { data?: { matter?: { title?: string; stage?: string } } }) => {
    tree.children.push({
      type: "paragraph",
      children: [
        {
          type: "text",
          value:
            `remark saw frontmatter title: ${file.data?.matter?.title ?? "missing-title"} ` +
            `and stage: ${file.data?.matter?.stage ?? "missing-stage"}.`,
        },
      ],
    });
  };
}

const addReadingTime: OxContentPlugin = (html) => {
  const wordCount = html.replace(/<[^>]*>/g, "").split(/\s+/).length;
  const minutes = Math.ceil(wordCount / 200);
  return `<p class="reading-time">Reading time: ${minutes} min</p>\n${html}`;
};

const wrapInArticle: OxContentPlugin = (html) => {
  return `<article class="ox-content-demo">${html}</article>`;
};

export default defineConfig({
  plugins: [
    oxContent({
      toc: true,
      plugin: {
        mdast: [annotateHeadings],
        remark: [remarkExposeFrontmatter],
        oxContent: [addReadingTime, wrapInArticle],
      },
    }),
  ],
});
