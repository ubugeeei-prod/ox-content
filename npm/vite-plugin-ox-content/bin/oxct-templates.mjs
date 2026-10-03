import { readFileSync } from "node:fs";

export const skins = [
  "analog-film",
  "atlas",
  "aurora",
  "bauhaus",
  "blueprint",
  "blur-glass",
  "brutalist",
  "clay",
  "editorial",
  "fabric",
  "fluid",
  "holo",
  "kiosk",
  "leather",
  "ledger",
  "liquid-glass",
  "manuscript",
  "neon",
  "noir",
  "paper",
  "pixel",
  "receipt",
  "risograph",
  "swiss",
  "terminal",
  "voltage",
  "zine",
];
export const palettes = [
  "arctic",
  "ayu",
  "cacao",
  "catppuccin",
  "commander",
  "coral",
  "dracula",
  "emerald",
  "everforest",
  "flexoki",
  "fuji",
  "github",
  "graphite",
  "gruvbox",
  "high-contrast",
  "horizon",
  "iceberg",
  "ink",
  "kanagawa",
  "material",
  "melange",
  "modus",
  "mono",
  "monokai",
  "moss",
  "night-owl",
  "nord",
  "oceanic",
  "one-dark",
  "palenight",
  "plum",
  "poimandres",
  "porcelain",
  "retro",
  "rose-pine",
  "sand",
  "sepia",
  "slate",
  "snow",
  "solarized",
  "stage",
  "synthwave",
  "tokyo-night",
  "vitesse",
  "voltage",
  "zenburn",
];

export function projectFiles(name, options) {
  const { version } = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));
  const vitePlus = options.manager === "vp";
  const run = vitePlus ? "vp" : "vite";
  const packageName =
    name
      .toLowerCase()
      .replace(/[^a-z0-9-]/g, "-")
      .replace(/^-+|-+$/g, "") || "my-content";
  const pkg = {
    name: packageName,
    version: "0.0.0",
    private: true,
    type: "module",
    scripts: {
      dev: `${run} dev`,
      build: `${run} build`,
      preview: `${run} preview`,
      lint: 'oxct lint "content/**/*.{md,mdx,mdc}"',
    },
    devDependencies: {
      "@ox-content/vite-plugin": version,
      [`@ox-content/theme-${options.skin}`]: version,
      [`@ox-content/theme-color-${options.palette}`]: version,
      ...(vitePlus ? { "vite-plus": "^0.3.2" } : { vite: "^8.0.0" }),
      typescript: "^5.8.0",
    },
  };
  const blog = options.template === "blog";
  const docs = options.template === "docs";
  return {
    "package.json": JSON.stringify(pkg, null, 2) + "\n",
    "vite.config.ts": `import { defineConfig } from ${JSON.stringify(vitePlus ? "vite-plus" : "vite")};
import { oxContent${blog ? ", defineCollections" : ""} } from "@ox-content/vite-plugin";
import skin from "@ox-content/theme-${options.skin}";
import palette from "@ox-content/theme-color-${options.palette}";

export default defineConfig({
  plugins: [oxContent({
    srcDir: "content",
    outDir: "dist",
    ${blog ? 'collections: defineCollections({ posts: { source: "posts/**/*.md" } }),\n    ' : ""}ssg: {
      siteName: ${JSON.stringify(name)},
      theme: [skin, palette, { nav: [{ text: "Home", link: "/" }${docs ? ', { text: "Guide", link: "/guide" }' : ""}], footer: { copyright: ${JSON.stringify(name)} } }],
    },
  })],
});
`,
    "tsconfig.json":
      JSON.stringify(
        {
          compilerOptions: {
            target: "ES2022",
            module: "ESNext",
            moduleResolution: "Bundler",
            strict: true,
            noEmit: true,
            skipLibCheck: true,
          },
          include: ["vite.config.ts"],
        },
        null,
        2,
      ) + "\n",
    ".gitignore": "node_modules/\ndist/\n.ox-content/\n.vite/\n",
    "README.md": `# ${name}\n\nAn Ox Content ${options.template} site using Vite.\n\nEdit Markdown in \`content/\`, then run the scripts in package.json.\n\n- \`dev\`: development server\n- \`build\`: static site in dist/\n- \`preview\`: preview the built site\n- \`lint\`: native Markdown checks\n\nSet up your editor with \`vpx oxct ide install\`.\n`,
    "content/index.md": `---\ntitle: ${JSON.stringify(name)}\ndescription: Welcome to your Ox Content site.\n---\n\n# ${name}\n\nYour content, beautifully rendered.\n\n${docs ? "[Read the guide](./guide.md)" : blog ? "[Read the first post](./posts/welcome.md)" : "Start writing Markdown here."}\n`,
    ...(docs
      ? {
          "content/guide.md":
            "---\ntitle: Getting started\n---\n\n# Getting started\n\nEdit a Markdown file and see it update immediately.\n\n## Code\n\n```ts\nconst greeting = 'Hello, Ox Content';\n```\n\n## Next steps\n\n- Write your first page\n- Choose your theme\n- Build and publish your site\n",
        }
      : {}),
    ...(blog
      ? {
          "content/posts/welcome.md":
            "---\ntitle: Welcome\ndescription: Your first post.\ndate: 2026-01-01\ntags: [welcome]\n---\n\n# Welcome\n\nThis is your first post. Make it your own.\n",
        }
      : {}),
  };
}
