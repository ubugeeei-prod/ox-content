import { describe, expect, it } from "vite-plus/test";
import { collectGitHubRepos, isSafeGitHubRepo, transformGitHub } from "./plugins/github";
import { transformBuiltinEmbeds } from "./plugins";
import { collectOgpUrls, isSafeOgpUrl, transformOgp } from "./plugins/ogp";

describe("builtin embed input hardening", () => {
  it("accepts only safe GitHub repo references", async () => {
    expect(isSafeGitHubRepo("ubugeeei/ox-content")).toBe(true);
    expect(isSafeGitHubRepo("../secret")).toBe(false);
    expect(isSafeGitHubRepo("owner/repo?tab=readme")).toBe(false);

    await expect(
      collectGitHubRepos(
        '<GitHub repo="ubugeeei/ox-content"></GitHub><GitHub repo="../secret"></GitHub>',
      ),
    ).resolves.toEqual(["ubugeeei/ox-content"]);

    const html = await transformGitHub(
      '<GitHub repo="../secret"></GitHub>',
      new Map([["../secret", null]]),
    );
    expect(html).toContain('href="#"');
  });

  it("accepts only public http OGP URLs", async () => {
    expect(isSafeOgpUrl("https://example.com/post")).toBe(true);
    expect(isSafeOgpUrl("http://127.0.0.1/admin")).toBe(false);
    expect(isSafeOgpUrl("http://[::1]/admin")).toBe(false);
    expect(isSafeOgpUrl("https://fcdomain.example/post")).toBe(true);
    expect(isSafeOgpUrl("javascript:alert(1)")).toBe(false);

    await expect(
      collectOgpUrls(
        '<OgCard url="https://example.com/post"></OgCard><OgCard url="http://127.0.0.1/admin"></OgCard>',
      ),
    ).resolves.toEqual(["https://example.com/post"]);

    const html = await transformOgp(
      '<OgCard url="javascript:alert(1)"></OgCard>',
      new Map([["javascript:alert(1)", null]]),
    );
    expect(html).toContain('href="#"');
  });

  it("runs GitHub embeds through the shared builtin transform", async () => {
    const html = await transformBuiltinEmbeds('<GitHub repo="../secret"></GitHub>', {
      github: {},
    });

    expect(html).toContain("ox-github-card");
    expect(html).toContain('href="#"');
  });

  it("can disable builtin embeds", async () => {
    const input = '<GitHub repo="../secret"></GitHub>';
    await expect(
      transformBuiltinEmbeds(input, {
        github: false,
      }),
    ).resolves.toBe(input);
  });
});
