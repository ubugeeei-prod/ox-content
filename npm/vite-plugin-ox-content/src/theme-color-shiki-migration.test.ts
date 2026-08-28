import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { resolveTheme, themeToNapi, type ThemeConfig } from "./theme";
import { renderThemeTokenCss } from "./theme-tokens";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "../../..");
const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));

const REQUIRED_SHIKI_IDS = [
  "kanagawa-lotus",
  "kanagawa-wave",
  "kanagawa-dragon",
  "github-light",
  "github-light-default",
  "github-light-high-contrast",
  "github-dark",
  "github-dark-default",
  "github-dark-dimmed",
  "github-dark-high-contrast",
  "catppuccin-latte",
  "catppuccin-frappe",
  "catppuccin-macchiato",
  "catppuccin-mocha",
  "gruvbox-light-hard",
  "gruvbox-light-medium",
  "gruvbox-light-soft",
  "gruvbox-dark-hard",
  "gruvbox-dark-medium",
  "gruvbox-dark-soft",
  "rose-pine-dawn",
  "rose-pine",
  "rose-pine-moon",
  "vitesse-light",
  "vitesse-dark",
  "vitesse-black",
  "ayu-light",
  "ayu-mirage",
  "ayu-dark",
  "dracula",
  "dracula-soft",
  "horizon",
  "horizon-bright",
  "material-theme-lighter",
  "material-theme",
  "material-theme-darker",
  "material-theme-ocean",
  "material-theme-palenight",
  "one-light",
  "one-dark-pro",
  "night-owl-light",
  "night-owl",
  "solarized-light",
  "solarized-dark",
] as const;

const SYNTAX_KEYS = [
  "syntax-token-comment",
  "syntax-token-punctuation",
  "syntax-token-keyword",
  "syntax-token-string",
  "syntax-token-constant",
  "syntax-token-function",
  "syntax-token-parameter",
  "syntax-token-link",
] as const;

type Mode = {
  syntax?: Partial<Record<SyntaxName, string>>;
};

type PaletteVariant = {
  exportName: string;
  light?: Mode;
  dark?: Mode;
};

type Palette = {
  id: string;
  title: string;
  light: Mode;
  dark: Mode;
  variants?: PaletteVariant[];
  shiki?: {
    source: {
      package: string;
      version: string;
      license: string;
      themeIds: string[];
    };
    migrations: {
      id: string;
      exportName: string;
      mode: "light" | "dark";
      pairing: string;
      notes?: string;
    }[];
  };
};

type PaletteCatalog = {
  palettes: Palette[];
};

type SyntaxName =
  | "comment"
  | "punctuation"
  | "keyword"
  | "string"
  | "constant"
  | "function"
  | "parameter"
  | "link";

type ThemeModule = Record<string, ThemeConfig>;

const catalog = JSON.parse(
  readFileSync(join(repoRoot, "scripts/theme-colors/palettes.json"), "utf8"),
) as PaletteCatalog;

const shikiPalettes = catalog.palettes.filter((palette) => palette.shiki);
const migrations = shikiPalettes.flatMap((palette) =>
  (palette.shiki?.migrations ?? []).map((migration) => ({ palette, migration })),
);

function variantFor(palette: Palette, exportName: string): PaletteVariant | undefined {
  return (palette.variants ?? []).find((variant) => variant.exportName === exportName);
}

function expectedSyntax(
  palette: Palette,
  exportName: string,
  mode: "light" | "dark",
): Partial<Record<SyntaxName, string>> | undefined {
  return (variantFor(palette, exportName)?.[mode] ?? palette[mode]).syntax;
}

async function loadThemeModule(palette: Palette): Promise<ThemeModule> {
  const source = join(packageRoot, `../theme-color/${palette.id}/src/index.ts`);
  return (await import(pathToFileURL(source).href)) as ThemeModule;
}

describe("Shiki migration theme-color presets", () => {
  it("pins the upstream Shiki source metadata for every migration family", () => {
    for (const palette of shikiPalettes) {
      expect(palette.shiki?.source.package, palette.id).toBe("@shikijs/themes");
      expect(palette.shiki?.source.version, palette.id).toBe("4.4.3");
      expect(palette.shiki?.source.license, palette.id).toBe("MIT");
      expect(palette.shiki?.source.themeIds, palette.id).toEqual(
        palette.shiki?.migrations.map((migration) => migration.id),
      );
    }
  });

  it("covers the initial popular Shiki migration matrix exactly", () => {
    expect(migrations.map(({ migration }) => migration.id).sort()).toEqual(
      [...REQUIRED_SHIKI_IDS].sort(),
    );
  });

  it("documents every Shiki ID, package, and export in English and Japanese", () => {
    for (const localePath of [
      "docs/content/theme-presets.md",
      "docs/content/ja/theme-presets.md",
    ]) {
      const page = readFileSync(join(repoRoot, localePath), "utf8");
      expect(page, localePath).toContain("@shikijs/themes@4.4.3");
      expect(page, localePath).toContain("TextMate");
      for (const { palette, migration } of migrations) {
        expect(page, `${localePath} ${migration.id}`).toContain(`\`${migration.id}\``);
        expect(page, `${localePath} ${palette.id}`).toContain(
          `\`@ox-content/theme-color-${palette.id}\``,
        );
        expect(page, `${localePath} ${migration.exportName}`).toContain(
          `\`${migration.exportName}\``,
        );
      }
    }
  });

  it("exports every mapped ThemeConfig from its generated package source", async () => {
    for (const palette of shikiPalettes) {
      const module = await loadThemeModule(palette);
      const exportNames = [...new Set(palette.shiki?.migrations.map((item) => item.exportName))];
      for (const exportName of exportNames) {
        const theme = module[exportName];
        expect(theme, `${palette.id} ${exportName}`).toBeDefined();
        expect(theme.colors, `${palette.id} ${exportName} colors`).toBeDefined();
        expect(theme.darkColors, `${palette.id} ${exportName} darkColors`).toBeDefined();
        expect(theme.tokens, `${palette.id} ${exportName} tokens`).toBeDefined();
        expect(theme.darkTokens, `${palette.id} ${exportName} darkTokens`).toBeDefined();
      }
    }
  });

  it("packs every migration family in the publish dry-run", () => {
    const dryRunScript = readFileSync(join(repoRoot, "scripts/package-dry-run.mjs"), "utf8");

    for (const palette of shikiPalettes) {
      expect(dryRunScript, palette.id).toContain(`"npm/theme-color/${palette.id}"`);
    }
  });

  it("locks representative syntax tokens for named Shiki variants", async () => {
    for (const { palette, migration } of migrations) {
      const module = await loadThemeModule(palette);
      const theme = module[migration.exportName];
      const lightSyntax = expectedSyntax(palette, migration.exportName, "light");
      const darkSyntax = expectedSyntax(palette, migration.exportName, "dark");

      expect(theme, `${palette.id} ${migration.exportName}`).toBeDefined();
      for (const key of SYNTAX_KEYS) {
        const syntaxName = key.replace(/^syntax-token-/, "") as SyntaxName;
        const lightValue = lightSyntax?.[syntaxName];
        const darkValue = darkSyntax?.[syntaxName];
        if (lightValue) {
          expect(theme.tokens?.[key], `${migration.id} light ${key}`).toBe(lightValue);
        }
        if (darkValue) {
          expect(theme.darkTokens?.[key], `${migration.id} dark ${key}`).toBe(darkValue);
        }
      }
    }
  });

  it("renders every mapped export through bare-host token CSS and the SSG path", async () => {
    for (const { palette, migration } of migrations) {
      const module = await loadThemeModule(palette);
      const theme = module[migration.exportName];
      expect(theme, `${palette.id} ${migration.exportName}`).toBeDefined();

      const syntaxCss = renderThemeTokenCss(theme, {
        include: (name) => name.startsWith("syntax-"),
      });
      expect(syntaxCss, `${migration.id} bare CSS`).toContain("--octc-syntax-foreground:");
      expect(syntaxCss, `${migration.id} bare CSS`).toContain("--octc-syntax-token-keyword:");
      expect(syntaxCss, `${migration.id} bare CSS`).toContain(
        "@media (prefers-color-scheme: dark)",
      );

      const ssgCss = themeToNapi(resolveTheme(theme)).css ?? "";
      for (const declaration of syntaxCss.matchAll(/--octc-syntax[^:]+: [^;]+;/g)) {
        expect(ssgCss, `${migration.id} SSG CSS`).toContain(declaration[0]);
      }
    }
  });

  it("keeps the Vite plugin free of Shiki runtime dependencies", () => {
    const packageJson = JSON.parse(readFileSync(join(packageRoot, "package.json"), "utf8")) as {
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
    };

    expect(Object.keys(packageJson.dependencies ?? {})).not.toContain("shiki");
    expect(Object.keys(packageJson.dependencies ?? {})).not.toContain("@shikijs/themes");
  });
});
