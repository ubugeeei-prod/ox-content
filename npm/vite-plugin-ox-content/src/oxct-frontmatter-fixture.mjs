import { mkdtemp, mkdir, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

export async function frontmatterFixture() {
  const root = await realpath(await mkdtemp(join(tmpdir(), "oxct-frontmatter-")));
  await mkdir(join(root, "content/posts"), { recursive: true });
  const entry = new URL("../dist/index.mjs", import.meta.url).href;
  const marker = join(root, "executed.txt");
  const config = `import { oxContent, defineFrontmatterSchemas } from ${JSON.stringify(entry)};
import { appendFileSync } from "node:fs";
appendFileSync(${JSON.stringify(marker)}, "executed\\n");
console.log("project log 日本語");
const schema = { "~standard": { version: 1, vendor: "test", async validate(value) {
  if (value.title === "slow") await new Promise(resolve => setTimeout(resolve, 150));
  if (typeof value.title !== "string" || ["bad", "slow"].includes(value.title))
    return { issues: [{ message: "Title must be an accepted string", path: ["title"] }] };
  return { value: { ...value, draft: value.draft ?? false } };
} } };
export default { plugins: oxContent({ srcDir: "content", highlight: false, ssg: false,
  frontmatterSchemas: defineFrontmatterSchemas({ "posts/**/*.md": { schema,
    jsonSchema: { type: "object", required: ["title"], properties: {
      title: { type: "string", description: "Accepted title" },
      draft: { type: "boolean", default: false },
      category: { enum: ["guide", "news"] },
      author: { type: "object", properties: { name: { type: "string" } } }
    } }
  } }) }) };
`;
  const file = join(root, "content/posts/a.md");
  const configFile = join(root, "vite.config.mjs");
  await writeFile(file, "---\ntitle: bad\n---\n# Body\n");
  await writeFile(configFile, config);
  return { root, file, config, configFile, marker, uri: pathToFileURL(file).href };
}
