import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { expect, test, type Page } from "@playwright/test";
import type { ThemeConfig } from "../../src/theme";
import { renderThemeTokenCss } from "../../src/theme-tokens";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "../../../..");
const npmRoot = join(repoRoot, "npm");

const SYNTAX_KEYS = [
  "syntax-foreground",
  "syntax-background",
  "syntax-token-comment",
  "syntax-token-punctuation",
  "syntax-token-keyword",
  "syntax-token-string",
  "syntax-token-constant",
  "syntax-token-function",
  "syntax-token-parameter",
  "syntax-token-link",
] as const;

type Palette = {
  id: string;
  shiki?: {
    migrations: {
      id: string;
      exportName: string;
    }[];
  };
};

type PaletteCatalog = {
  palettes: Palette[];
};

type ThemeModule = Record<string, ThemeConfig>;

const catalog = JSON.parse(
  readFileSync(join(repoRoot, "scripts/theme-colors/palettes.json"), "utf8"),
) as PaletteCatalog;

const migrationExports = catalog.palettes.flatMap((palette) =>
  [...new Set((palette.shiki?.migrations ?? []).map((migration) => migration.exportName))].map(
    (exportName) => ({ palette, exportName }),
  ),
);

test("renders every Shiki migration preset through browser light and dark CSS variables", async ({
  page,
}) => {
  await page.setViewportSize({ width: 840, height: 560 });

  for (const { palette, exportName } of migrationExports) {
    const module = await loadThemeModule(palette);
    const theme = module[exportName];
    expect(theme, `${palette.id} ${exportName}`).toBeDefined();

    for (const mode of ["light", "dark"] as const) {
      await renderFixture(page, theme, mode);
      const values = await collectSyntaxVariables(page);
      const expected = mode === "dark" ? theme.darkTokens : theme.tokens;

      for (const key of SYNTAX_KEYS) {
        const expectedValue = expected?.[key];
        expect(expectedValue, `${palette.id} ${exportName} ${mode} ${key}`).toBeDefined();
        expect(normalizeCssValue(values[key]), `${palette.id} ${exportName} ${mode} ${key}`).toBe(
          normalizeCssValue(expectedValue ?? ""),
        );
      }
    }
  }
});

async function loadThemeModule(palette: Palette): Promise<ThemeModule> {
  const source = join(npmRoot, `theme-color/${palette.id}/src/index.ts`);
  return (await import(pathToFileURL(source).href)) as ThemeModule;
}

async function renderFixture(page: Page, theme: ThemeConfig, mode: "light" | "dark") {
  await page.setContent(
    `<!doctype html>
<html data-theme="${mode}">
  <head>
    <meta charset="utf-8">
    <style>${renderThemeTokenCss(theme)}</style>
    <style>
      body {
        margin: 0;
        background: var(--octc-syntax-background);
        color: var(--octc-syntax-foreground);
      }
      code {
        display: block;
        width: 640px;
        margin: 24px;
        padding: 18px;
        border: 1px solid var(--octc-syntax-token-punctuation);
        font: 16px/1.6 ui-monospace, SFMono-Regular, Menlo, monospace;
      }
      .comment { color: var(--octc-syntax-token-comment); }
      .keyword { color: var(--octc-syntax-token-keyword); }
      .string { color: var(--octc-syntax-token-string); }
      .constant { color: var(--octc-syntax-token-constant); }
      .function { color: var(--octc-syntax-token-function); }
      .parameter { color: var(--octc-syntax-token-parameter); }
      .link { color: var(--octc-syntax-token-link); }
    </style>
  </head>
  <body>
    <code>
      <span class="comment">// theme fixture</span>
      <span class="keyword">const</span>
      <span class="parameter">preset</span>
      <span class="keyword">=</span>
      <span class="function">theme</span>(<span class="string">"shiki"</span>,
      <span class="constant">44</span>,
      <a class="link" href="#">docs</a>)
    </code>
  </body>
</html>`,
    { waitUntil: "load" },
  );
}

async function collectSyntaxVariables(
  page: Page,
): Promise<Record<(typeof SYNTAX_KEYS)[number], string>> {
  return page.evaluate((keys) => {
    const rootStyle = getComputedStyle(document.documentElement);
    return Object.fromEntries(
      keys.map((key) => [key, rootStyle.getPropertyValue(`--octc-${key}`).trim()]),
    ) as Record<(typeof keys)[number], string>;
  }, SYNTAX_KEYS);
}

function normalizeCssValue(value: string) {
  return value.replace(/\s+/g, " ").trim();
}
