import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { CollectionValidationError } from "./collection-validation";
import {
  buildCollectionManifest,
  generateCollectionsVirtualModule,
  resolveCollectionsOptions,
} from "./collections";
import type { CollectionValidationContext, ResolvedOptions } from "./types";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("collections", () => {
  it("builds metadata-only entries without embedding rendered html by default", async () => {
    const root = await createFixture();
    const manifest = await buildCollectionManifest(root, createOptions());

    expect(manifest.collections.docs).toEqual([
      {
        id: "docs/guide/install",
        collection: "docs",
        path: "/docs/guide/install",
        stem: "docs/guide/install",
        source: "docs/1.guide/2.install.md",
        extension: ".md",
        title: "Install",
        description: "Setup guide",
        draft: false,
        frontmatter: {
          title: "Install",
          description: "Setup guide",
          draft: false,
        },
      },
    ]);
    expect(manifest.collections.blog[0].body).toContain("# First Post");
    expect(manifest.collections.blog[0]).not.toHaveProperty("html");
    expect(manifest.collections.blog[0]).not.toHaveProperty("toc");
  });

  it("generates a Nuxt-like query builder virtual module", async () => {
    const root = await createFixture();
    const code = await generateCollectionsVirtualModule(root, createOptions());
    const mod = await import(`data:text/javascript;charset=utf-8,${encodeURIComponent(code)}`);

    await expect(
      mod
        .queryCollection("blog")
        .where("draft", "=", false)
        .order("date", "DESC")
        .select("title", "path")
        .all(),
    ).resolves.toEqual([
      { title: "Second Post", path: "/blog/second" },
      { title: "First Post", path: "/blog/first" },
    ]);
    await expect(
      mod.queryCollection("docs").path("/docs/guide/install").first(),
    ).resolves.toMatchObject({
      title: "Install",
      path: "/docs/guide/install",
    });
    await expect(
      mod.queryCollection("blog").where("path", "LIKE", "/blog/%").count(),
    ).resolves.toBe(2);
  });

  it("runs validate hooks with file-tree paths before permalink rewrites", async () => {
    const root = await createPermalinkFixture({
      "blog/first/index.md": "---\ntitle: First\npermalink: /custom-first\n---\n# First\n",
    });
    const contexts: Array<Pick<CollectionValidationContext, "source" | "documentPath" | "path">> =
      [];

    const manifest = await buildCollectionManifest(
      root,
      createOptions({
        permalinks: { enabled: true },
        collections: resolveCollectionsOptions({
          blog: {
            source: "blog/*/index.md",
            validate(context) {
              contexts.push({
                source: context.source,
                documentPath: context.documentPath,
                path: context.path,
              });
            },
          },
        }),
      }),
    );

    expect(contexts).toEqual([
      {
        source: "blog/first/index.md",
        documentPath: path.join(root, "content/blog/first/index.md"),
        path: "/blog/first",
      },
    ]);
    expect(manifest.collections.blog[0]?.path).toBe("/custom-first");
  });

  it("fails with aggregated diagnostics from collection validate hooks", async () => {
    const root = await createPermalinkFixture({
      "blog/first/index.md": "---\ntitle: First\npermalink: /blog/wrong\n---\n# First\n",
      "blog/second/index.md": "---\ntitle: Second\n---\n# Second\n",
      "docs/guide.md": "---\ntitle: Guide\n---\n# Guide\n",
    });

    let thrown: unknown;
    try {
      await buildCollectionManifest(
        root,
        createOptions({
          collections: resolveCollectionsOptions({
            blog: {
              source: "blog/*/index.md",
              validate: ({ frontmatter, path }) => {
                const permalink = frontmatter.permalink;
                return permalink === path
                  ? undefined
                  : `expected permalink ${path}, found ${formatPermalinkValue(permalink)}`;
              },
            },
            docs: {
              source: "docs/**/*.md",
              validate: () => undefined,
            },
          }),
        }),
      );
    } catch (error) {
      thrown = error;
    }

    expect(thrown).toBeInstanceOf(CollectionValidationError);
    expect((thrown as CollectionValidationError).diagnostics).toEqual([
      {
        collection: "blog",
        source: "blog/first/index.md",
        documentPath: path.join(root, "content/blog/first/index.md"),
        path: "/blog/first",
        message: "expected permalink /blog/first, found /blog/wrong",
      },
      {
        collection: "blog",
        source: "blog/second/index.md",
        documentPath: path.join(root, "content/blog/second/index.md"),
        path: "/blog/second",
        message: "expected permalink /blog/second, found (missing)",
      },
    ]);
    expect((thrown as Error).message).toContain(
      "[ox-content] Collection validation failed with 2 diagnostics.",
    );
    expect((thrown as Error).message).toContain(
      "- blog: blog/second/index.md (/blog/second): expected permalink /blog/second, found (missing)",
    );
  });
});

async function createFixture(): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-collections-"));
  tempDirs.push(root);
  await fs.mkdir(path.join(root, "content/blog"), { recursive: true });
  await fs.mkdir(path.join(root, "content/docs/1.guide"), { recursive: true });
  await fs.writeFile(
    path.join(root, "content/blog/1.first.md"),
    "---\ntitle: First Post\ndate: 2024-01-01\ndraft: false\n---\n# First Post\n",
  );
  await fs.writeFile(
    path.join(root, "content/blog/2.second.md"),
    "---\ntitle: Second Post\ndate: 2024-02-01\ndraft: false\n---\n# Second Post\n",
  );
  await fs.writeFile(
    path.join(root, "content/docs/1.guide/2.install.md"),
    "---\ntitle: Install\ndescription: Setup guide\ndraft: false\n---\n# Install\n",
  );
  return root;
}

function createOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return {
    srcDir: "content",
    extensions: [".md", ".markdown", ".mdx"],
    frontmatter: true,
    collections: resolveCollectionsOptions({
      blog: { source: "blog/**/*.md", include: ["body"] },
      docs: "docs/**/*.md",
    }),
    ...overrides,
  } as ResolvedOptions;
}

async function createPermalinkFixture(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-collections-validate-"));
  tempDirs.push(root);
  for (const [file, source] of Object.entries(files)) {
    const destination = path.join(root, "content", file);
    await fs.mkdir(path.dirname(destination), { recursive: true });
    await fs.writeFile(destination, source);
  }
  return root;
}

function formatPermalinkValue(value: unknown): string {
  return typeof value === "string" ? value : "(missing)";
}
