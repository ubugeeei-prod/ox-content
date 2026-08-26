import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { resolveTheme, themeToNapi } from "./theme";
import {
  flattenThemeFont,
  flattenThemeFonts,
  namedFontVarsCss,
  planSelfHostedFaces,
  writeSelfHostedThemeFonts,
} from "./theme-fonts";
import { googleCssUrl, parseGoogleCss } from "./theme-fonts-acquire";

const fixtureFont = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "ox-test-font.woff2");
const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })));
});

async function tempDir(prefix: string): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

describe("flattenThemeFont", () => {
  it("keeps the string stack form unchanged", () => {
    expect(flattenThemeFont("Inter, sans-serif", "sans-serif")).toBe("Inter, sans-serif");
  });

  it("quotes families with spaces and appends a generic fallback", () => {
    expect(flattenThemeFont({ family: "DM Mono" }, "monospace")).toBe('"DM Mono", monospace');
  });
});

describe("flattenThemeFonts", () => {
  it("mixes an object sans face with a string mono stack", () => {
    expect(
      flattenThemeFonts({
        sans: { family: "Inter", provider: "google", selfHost: true },
        mono: "DM Mono, monospace",
      }),
    ).toEqual({ sans: "Inter, sans-serif", mono: "DM Mono, monospace" });
  });
});

describe("namedFontVarsCss", () => {
  it("exposes kebab-case CSS variables for named families", () => {
    expect(namedFontVarsCss({ named: { code: { family: "JetBrains Mono" } } })).toContain(
      '--octc-font-code: "JetBrains Mono", sans-serif;',
    );
  });

  it("rejects a named key that would break out of a declaration", () => {
    expect(() => namedFontVarsCss({ named: { "bad: red; }": "x" } })).toThrow(
      /Invalid theme font name/,
    );
  });
});

describe("themeToNapi fonts", () => {
  it("flattens object fonts and injects local stylesheet plus preload tags", () => {
    const napi = themeToNapi(
      resolveTheme({
        fonts: {
          sans: {
            family: "Inter",
            provider: "local",
            path: "./fixtures/ox-test-font.woff2",
            weights: [400],
            selfHost: true,
            preload: true,
          },
          named: {
            code: { family: "JetBrains Mono", provider: "google", weights: [400], selfHost: true },
          },
        },
      }),
      undefined,
      "/docs/",
    );

    expect(napi.fonts?.sans).toBe("Inter, sans-serif");
    expect(typeof napi.fonts?.mono).toBe("string");
    expect(napi.css).toContain('--octc-font-code: "JetBrains Mono", sans-serif;');
    expect(napi.embed?.head).toContain("/docs/__ox_fonts__/fonts.css");
    expect(napi.embed?.head).toContain("/docs/__ox_fonts__/inter-400-normal-latin.woff2");
    expect(napi.embed?.head).not.toContain("fonts.googleapis.com");
    expect(napi.embed?.head).not.toContain("fonts.gstatic.com");
  });

  it("does not inject font links for string stacks", () => {
    const napi = themeToNapi(resolveTheme({ fonts: { sans: "Inter, sans-serif" } }));
    expect(napi.embed?.head ?? "").not.toContain("__ox_fonts__");
  });
});

describe("writeSelfHostedThemeFonts", () => {
  it("copies a local fixture and emits @font-face without remote URLs", async () => {
    const outDir = await tempDir("ox-fonts-local-");
    const fixture = await readFile(fixtureFont);
    const written = await writeSelfHostedThemeFonts({
      fonts: {
        sans: {
          family: "Ox Test",
          provider: "local",
          path: fixtureFont,
          weights: [400],
          selfHost: true,
          preload: true,
        },
        named: {
          code: {
            family: "Ox Test",
            provider: "local",
            path: fixtureFont,
            weights: [400],
            selfHost: true,
          },
        },
      },
      outDir,
      root: outDir,
    });

    const cssPath = join(outDir, "__ox_fonts__", "fonts.css");
    const fontPath = join(outDir, "__ox_fonts__", "ox-test-400-normal-latin.woff2");
    expect(written).toEqual(expect.arrayContaining([cssPath, fontPath]));
    expect(await readFile(fontPath)).toEqual(fixture);
    const css = await readFile(cssPath, "utf8");
    expect(css).toContain('font-family: "Ox Test"');
    expect(css).toContain("url(./ox-test-400-normal-latin.woff2)");
    expect(css).not.toContain("fonts.googleapis.com");
    expect(css).not.toContain("fonts.gstatic.com");
    expect(css).not.toContain("http");
  });

  it("resolves an @fontsource-style files directory without network", async () => {
    const root = await tempDir("ox-fonts-fontsource-");
    const outDir = join(root, "dist");
    const pkg = join(root, "node_modules", "@fontsource", "ox-test", "files");
    await mkdir(pkg, { recursive: true });
    await writeFile(join(pkg, "ox-test-latin-400-normal.woff2"), await readFile(fixtureFont));

    await writeSelfHostedThemeFonts({
      fonts: {
        mono: {
          family: "Ox Test",
          provider: "local",
          path: "@fontsource/ox-test",
          weights: [400],
          selfHost: true,
        },
      },
      outDir,
      root,
    });

    expect(await readFile(join(outDir, "__ox_fonts__", "ox-test-400-normal-latin.woff2"))).toEqual(
      await readFile(fixtureFont),
    );
  });

  it("is a no-op when every family is a string or selfHost is off", async () => {
    const outDir = await tempDir("ox-fonts-skip-");
    const written = await writeSelfHostedThemeFonts({
      fonts: {
        sans: "Inter, sans-serif",
        mono: { family: "DM Mono", provider: "google" },
      },
      outDir,
      root: outDir,
    });
    expect(written).toEqual([]);
  });

  it("downloads Google faces through the injected fetch and reuses the cache", async () => {
    const root = await tempDir("ox-fonts-google-");
    const outDir = join(root, "dist");
    const cacheDir = join(root, "cache");
    const css = `/* latin */
@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 400;
  src: url(https://fonts.gstatic.com/s/inter/v18/fake.woff2) format('woff2');
  unicode-range: U+0000-00FF;
}`;
    const payload = new Uint8Array([1, 2, 3, 4]);
    let fetches = 0;
    const fetchFn: typeof fetch = async (input) => {
      fetches += 1;
      const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
      if (url.startsWith("https://fonts.googleapis.com/css2")) {
        return new Response(css);
      }
      if (url === "https://fonts.gstatic.com/s/inter/v18/fake.woff2") {
        return new Response(payload);
      }
      throw new Error(`unexpected download ${url}`);
    };
    const fonts = {
      sans: { family: "Inter", provider: "google" as const, weights: [400], selfHost: true },
    };

    await writeSelfHostedThemeFonts({ fonts, outDir, root, cacheDir, fetch: fetchFn });
    const first = fetches;
    await writeSelfHostedThemeFonts({
      fonts,
      outDir: join(root, "dist-2"),
      root,
      cacheDir,
      fetch: async () => {
        throw new Error("cache should prevent network");
      },
    });

    expect(first).toBe(2);
    expect(await readFile(join(outDir, "__ox_fonts__", "inter-400-normal-latin.woff2"))).toEqual(
      Buffer.from(payload),
    );
    const emitted = await readFile(join(outDir, "__ox_fonts__", "fonts.css"), "utf8");
    expect(emitted).toContain("unicode-range: U+0000-00FF");
    expect(emitted).not.toContain("fonts.gstatic.com");
  });
});

describe("google css helpers", () => {
  it("builds a CSS2 URL and parses subset comments", () => {
    const [face] = planSelfHostedFaces({
      sans: { family: "Inter", provider: "google", weights: [600], selfHost: true },
    });
    expect(googleCssUrl(face!)).toContain("family=Inter:wght@600");
    expect(
      parseGoogleCss(
        "/* latin */\n@font-face { font-weight: 600; src: url(https://fonts.gstatic.com/s/x.woff2); }",
      ),
    ).toEqual([
      {
        subset: "latin",
        weight: 600,
        style: "normal",
        url: "https://fonts.gstatic.com/s/x.woff2",
        unicodeRange: undefined,
      },
    ]);
  });
});
