import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { buildSsg, resolveSsgOptions } from "./ssg";
import { resolveTeamOptions } from "./team";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-team-"));
  tempDirs.push(root);
  const srcDir = path.join(root, "content");
  await fs.mkdir(srcDir, { recursive: true });
  for (const [relative, body] of Object.entries(files)) {
    const filePath = path.join(srcDir, relative);
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, body, "utf8");
  }
  return root;
}

describe("resolveTeamOptions", () => {
  it("omitted => false; true => true; {} => true", () => {
    expect(resolveTeamOptions(undefined).enabled).toBe(false);
    expect(resolveTeamOptions(false).enabled).toBe(false);
    expect(resolveSsgOptions(undefined).team?.enabled).toBe(false);
    expect(resolveSsgOptions(true).team?.enabled).toBe(false);
    expect(resolveSsgOptions({}).team?.enabled).toBe(false);

    expect(resolveTeamOptions(true)).toEqual({ enabled: true, members: [] });
    expect(resolveSsgOptions({ team: true }).team).toEqual({ enabled: true, members: [] });

    expect(resolveTeamOptions({})).toEqual({ enabled: true, members: [] });
    expect(resolveSsgOptions({ team: {} }).team).toEqual({ enabled: true, members: [] });
  });

  it("keeps configured members when the option is an object", () => {
    const members = [
      {
        name: "Ada Lovelace",
        role: "Mathematician",
        avatar: "https://cdn.example.com/ada.png",
        links: [{ label: "Website", href: "https://example.com/ada" }],
      },
    ];
    expect(resolveTeamOptions({ members })).toEqual({ enabled: true, members });
  });
});

describe("buildSsg team page", () => {
  it("ignores layout: team when the option is omitted", async () => {
    const root = await makeSite({
      "team.md": "---\nlayout: team\n---\n\n# People\n\nOrdinary body.\n",
    });
    const base = createDocsResolvedOptions();
    await buildSsg(createDocsResolvedOptions({ ssg: { ...base.ssg, siteName: "Docs" } }), root);

    const html = await fs.readFile(path.join(root, "dist", "team", "index.html"), "utf8");
    expect(html).toContain("Ordinary body.");
    expect(html).not.toContain("ox-team");
  });

  it("renders escaped member cards on layout: team when enabled", async () => {
    const root = await makeSite({
      "team.md": "---\nlayout: team\n---\n\n# People\n\nAbout the maintainers.\n",
    });
    const base = createDocsResolvedOptions();
    await buildSsg(
      createDocsResolvedOptions({
        ssg: {
          ...base.ssg,
          siteName: "Docs",
          team: resolveTeamOptions({
            members: [
              {
                name: "<img src=x onerror=alert(1)>",
                role: "Lead",
                avatar: "javascript:alert(1)",
                links: [
                  { label: "Safe", href: "https://example.com/ada" },
                  { label: "Bad", href: "javascript:alert(1)" },
                ],
              },
              {
                name: "Grace Hopper",
                avatar: "https://cdn.example.com/grace.png",
                links: [{ label: "Profile", href: "/people/grace" }],
              },
            ],
          }),
        },
      }),
      root,
    );

    const html = await fs.readFile(path.join(root, "dist", "team", "index.html"), "utf8");
    expect(html).toContain("ox-team");
    expect(html).toContain("About the maintainers.");
    expect(html).toContain("&lt;img src=x onerror=alert(1)&gt;");
    expect(html).not.toContain("<img src=x");
    expect(html).not.toContain("javascript:");
    expect(html).toContain('href="https://example.com/ada"');
    expect(html).not.toContain("Bad");
    expect(html).toContain("Grace Hopper");
    expect(html).toContain('src="https://cdn.example.com/grace.png"');
    expect(html).toContain('href="/people/grace"');
    expect(html).toMatch(/ox-content:css:team|ox-content-team-[0-9a-f]+\.css/);
  });
});
