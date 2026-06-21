import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { resolveNavigationGroups } from "./ssg";
import {
  parseVitePressMigrationCliArgs,
  runVitePressMigrationCli,
  type VitePressMigrationCliRuntime,
} from "./vitepress-cli-runtime";
import {
  convertVitePressNav,
  convertVitePressSidebar,
  fromVitePressConfig,
  generateVitePressMigrationConfig,
  normalizeVitePressFrontmatter,
} from "./vitepress";

describe("vitepress migration helpers", () => {
  it("converts a flat VitePress sidebar into ox-content navigation groups", () => {
    const navigation = convertVitePressSidebar([
      { text: "Home", link: "/" },
      { text: "Getting Started", link: "/getting-started" },
      { text: "API", link: "/api.md" },
    ]);

    expect(navigation).toEqual([
      {
        title: "Guide",
        items: [
          { title: "Home", path: "/" },
          { title: "Getting Started", path: "/getting-started" },
          { title: "API", path: "/api" },
        ],
      },
    ]);
  });

  it("converts VitePress nav dropdowns into grouped sidebar navigation", () => {
    const navigation = convertVitePressNav([
      { text: "Guide", items: [{ text: "Intro", link: "/guide/intro" }] },
      { text: "GitHub", link: "https://github.com/ubugeeei-prod/ox-content" },
    ]);

    expect(navigation).toEqual([
      {
        title: "Navigation",
        items: [
          {
            title: "GitHub",
            href: "https://github.com/ubugeeei-prod/ox-content",
          },
        ],
      },
      {
        title: "Guide",
        items: [{ title: "Intro", path: "/guide/intro" }],
      },
    ]);
  });

  it("maps VitePress config into ox-content options and preserves user overrides", () => {
    const options = fromVitePressConfig(
      {
        title: "Docs",
        base: "/docs/",
        themeConfig: {
          logo: "/logo.svg",
          socialLinks: [{ icon: "github", link: "https://github.com/ubugeeei-prod/ox-content" }],
          footer: { copyright: "2026" },
          sidebar: [{ text: "Intro", link: "/intro" }],
          search: { placeholder: "Search VitePress docs" },
        },
      },
      {
        srcDir: "docs",
        ssg: {
          theme: {
            footer: {
              message: "Migrated from VitePress",
            },
          },
        },
      },
    );

    expect(options.base).toBe("/docs/");
    expect(options.srcDir).toBe("docs");
    expect(options.search).toEqual({ placeholder: "Search VitePress docs" });

    expect(options.ssg).not.toBe(false);
    if (!options.ssg || options.ssg === true) {
      throw new Error("Expected migrated SSG options");
    }

    const ssg = options.ssg;

    expect(ssg.siteName).toBe("Docs");
    expect(ssg.navigation).toEqual([
      {
        title: "Guide",
        items: [{ title: "Intro", path: "/intro" }],
      },
    ]);
    expect(ssg.theme?.header?.logo).toBe("/logo.svg");
    const socialLinks = ssg.theme?.socialLinks;
    if (!socialLinks || Array.isArray(socialLinks)) {
      throw new Error("Expected migrated social links");
    }
    expect(socialLinks.github).toBe("https://github.com/ubugeeei-prod/ox-content");
    expect(ssg.theme?.footer?.copyright).toBe("2026");
    expect(ssg.theme?.footer?.message).toBe("Migrated from VitePress");
  });

  it("generates an editable ox-content options module", () => {
    const source = generateVitePressMigrationConfig(
      {
        title: "Docs",
        base: "/docs/",
        themeConfig: {
          sidebar: [{ text: "Intro", link: "/intro.md" }],
          search: { placeholder: "Search docs" },
        },
      },
      {
        srcDir: "docs",
        outDir: "dist",
      },
    );

    expect(source).toMatchSnapshot();
  });

  it("parses migration CLI options without depending on a Node process", () => {
    expect(
      parseVitePressMigrationCliArgs([
        ".vitepress/config.ts",
        "--src-dir",
        "docs",
        "--out-dir",
        "dist",
        "--out",
        "ox-content.config.ts",
        "--force",
      ]),
    ).toEqual({
      configPath: ".vitepress/config.ts",
      srcDir: "docs",
      outDir: "dist",
      out: "ox-content.config.ts",
      force: true,
      help: false,
    });
  });
});

describe("vitepress migration CLI", () => {
  it("runs the migration CLI through a runtime adapter", async () => {
    const directory = await mkdtemp(path.join(tmpdir(), "ox-content-vitepress-cli-"));
    const configPath = path.join(directory, "config.mjs");
    const stdout: string[] = [];
    const runtime: VitePressMigrationCliRuntime = {
      name: "bun",
      argv: [configPath, "--src-dir", "docs"],
      cwd: () => directory,
      writeStdout: (value) => {
        stdout.push(value);
      },
      writeStderr: () => {},
      setExitCode: () => {},
    };

    try {
      await writeFile(
        configPath,
        `export default {
  title: "Docs",
  base: "/docs/",
  themeConfig: {
    sidebar: [{ text: "Intro", link: "/intro.md" }]
  }
};`,
      );

      await runVitePressMigrationCli(runtime);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }

    const source = stdout.join("");
    expect(source).toMatchSnapshot();
  });

  it("normalizes VitePress home frontmatter into ox-content entry frontmatter", () => {
    const frontmatter = normalizeVitePressFrontmatter({
      layout: "home",
      hero: {
        name: "Docs",
        image: {
          light: "/logo-light.svg",
          dark: "/logo-dark.svg",
          width: "120",
          height: 80,
        },
      },
    });

    expect(frontmatter).toEqual({
      layout: "entry",
      hero: {
        name: "Docs",
        image: {
          src: "/logo-light.svg",
          lightSrc: "/logo-light.svg",
          darkSrc: "/logo-dark.svg",
          width: 120,
          height: 80,
        },
      },
    });
  });

  it("preserves ox-content hero image theme sources", () => {
    const frontmatter = normalizeVitePressFrontmatter({
      layout: "entry",
      hero: {
        name: "Ox Content",
        image: {
          src: "oxcontent-dark.svg",
          lightSrc: "oxcontent-dark.svg",
          darkSrc: "oxcontent-light.svg",
          alt: "Ox Content wordmark",
        },
      },
    });

    expect(frontmatter).toEqual({
      layout: "entry",
      hero: {
        name: "Ox Content",
        image: {
          src: "oxcontent-dark.svg",
          lightSrc: "oxcontent-dark.svg",
          darkSrc: "oxcontent-light.svg",
          alt: "Ox Content wordmark",
        },
      },
    });
  });

  it("derives HTML hrefs from migrated navigation paths", () => {
    const navigation = resolveNavigationGroups(
      [
        {
          title: "Guide",
          items: [
            { title: "Intro", path: "/intro" },
            { title: "Top", path: "/guide", href: "/guide#top" },
          ],
        },
      ],
      "/docs/",
      ".html",
    );

    expect(navigation).toEqual([
      {
        title: "Guide",
        items: [
          {
            title: "Intro",
            path: "/intro",
            href: "/docs/intro/index.html",
          },
          {
            title: "Top",
            path: "/guide",
            href: "/docs/guide/index.html#top",
          },
        ],
      },
    ]);
  });
});
