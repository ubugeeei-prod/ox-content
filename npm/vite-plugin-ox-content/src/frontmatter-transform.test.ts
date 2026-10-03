import { resolve } from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { z } from "zod";
import { resolveOptions } from "./resolve-options";
import { transformMarkdown } from "./transform";

describe("frontmatter schema build integration", () => {
  it("rejects invalid source and applies defaults and transforms to the resulting page data", async () => {
    const options = resolveOptions({
      srcDir: "content",
      highlight: false,
      ssg: false,
      frontmatterSchemas: {
        "posts/*.md": z.object({
          title: z.string().transform((value) => value.toUpperCase()),
          draft: z.boolean().default(false),
        }),
      },
    });
    const file = resolve("content/posts/a.md");
    await expect(
      transformMarkdown("---\ntitle: 42\n---\n", `${file}?raw`, options),
    ).rejects.toThrow(`${file}:2:8`);
    await expect(transformMarkdown("---\ntitle: 42\n---\n# Body", file, options)).rejects.toThrow(
      `${file}:2:8`,
    );
    expect(
      (await transformMarkdown("---\ntitle: hello\n---\n# Body", file, options)).frontmatter,
    ).toEqual({ title: "HELLO", draft: false });
    expect(
      (await transformMarkdown("---\ntitle: 42\n---\n# Body", resolve("content/other.md"), options))
        .frontmatter,
    ).toEqual({ title: 42 });
  });
});
