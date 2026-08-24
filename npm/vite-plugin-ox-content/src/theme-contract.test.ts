import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "../../..");
const PLUGIN_PEER = "@ox-content/vite-plugin";

/** Catalog schemes must ship a full light/dark palette. ThemeColors stays optional for site overrides. */
const PALETTE_KEYS = [
  "primary",
  "primaryHover",
  "background",
  "backgroundAlt",
  "text",
  "textMuted",
  "border",
  "codeBackground",
  "codeText",
] as const;

const SHIKI_BASE = ["shiki-foreground", "shiki-background"] as const;
const HARD_CODED_COLOR = /#(?:[0-9a-f]{3,8})\b|\brgba?\(|\bhsla?\(/i;

type PkgJson = {
  name?: string;
  files?: string[];
  exports?: Record<string, unknown>;
  peerDependencies?: Record<string, string>;
  peerDependenciesMeta?: Record<string, { optional?: boolean }>;
};

type ThemePkg = { dir: string; json: PkgJson };

function listPackages(familyDir: string): ThemePkg[] {
  const root = join(repoRoot, familyDir);
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    if (!entry.isDirectory()) {
      return [];
    }
    const dir = join(root, entry.name);
    const manifest = join(dir, "package.json");
    if (!existsSync(manifest)) {
      return [];
    }
    return [{ dir, json: JSON.parse(readFileSync(manifest, "utf8")) as PkgJson }];
  });
}

function collectSrc(dir: string): string[] {
  const src = join(dir, "src");
  if (!existsSync(src)) {
    return [];
  }
  const files: string[] = [];
  const walk = (current: string) => {
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      const path = join(current, entry.name);
      if (entry.isDirectory()) {
        walk(path);
      } else if (/\.[cm]?[jt]s$/.test(entry.name) && !entry.name.endsWith(".d.ts")) {
        files.push(path);
      }
    }
  };
  walk(src);
  return files;
}

function sliceObject(source: string, openBrace: number): string {
  let depth = 0;
  let quote: string | undefined;
  for (let i = openBrace; i < source.length; i++) {
    const ch = source[i];
    const prev = source[i - 1];
    if (quote) {
      if (ch === quote && prev !== "\\") {
        quote = undefined;
      }
      continue;
    }
    if (ch === '"' || ch === "'" || ch === "`") {
      quote = ch;
      continue;
    }
    if (ch === "{") {
      depth++;
    } else if (ch === "}") {
      depth--;
      if (depth === 0) {
        return source.slice(openBrace + 1, i);
      }
    }
  }
  throw new Error("unbalanced object literal");
}

function themeConfigBodies(source: string): string[] {
  const bodies: string[] = [];
  const pattern = /:\s*ThemeConfig\s*=\s*\{/g;
  for (const match of source.matchAll(pattern)) {
    bodies.push(sliceObject(source, match.index + match[0].length - 1));
  }
  return bodies;
}

function namedObject(body: string, name: string): string | undefined {
  const match = new RegExp(`\\b${name}\\s*:\\s*\\{`).exec(body);
  return match ? sliceObject(body, match.index + match[0].length - 1) : undefined;
}

function objectKeys(body: string): string[] {
  return [...body.matchAll(/(?:^|[,{])\s*(?:["']([^"']+)["']|([A-Za-z_][\w-]*))\s*:/g)].map(
    (match) => match[1] ?? match[2] ?? "",
  );
}

function stringValues(body: string): string[] {
  return [...body.matchAll(/:\s*(["'`])((?:\\.|(?!\1).)*)\1/g)].map((match) => match[2] ?? "");
}

function missing(required: readonly string[], actual: string[]): string[] {
  return required.filter((key) => !actual.includes(key));
}

function configsIn(pkg: ThemePkg): string[] {
  return collectSrc(pkg.dir).flatMap((file) => themeConfigBodies(readFileSync(file, "utf8")));
}

const schemes = listPackages("npm/theme-color");
const skins = listPackages("npm/theme");

describe("theme package contract", () => {
  it("discovers published theme-color and theme packages from the filesystem", () => {
    expect(schemes.length).toBeGreaterThan(10);
    expect(skins.length).toBeGreaterThan(10);
  });

  it("exports a ThemeConfig with required light and dark palette keys from every theme-color package", () => {
    for (const pkg of schemes) {
      const configs = configsIn(pkg);
      expect(configs.length, pkg.json.name).toBeGreaterThan(0);
      for (const config of configs) {
        const colors = namedObject(config, "colors");
        const darkColors = namedObject(config, "darkColors");
        expect(colors, `${pkg.json.name} colors`).toBeTruthy();
        expect(darkColors, `${pkg.json.name} darkColors`).toBeTruthy();
        expect(missing(PALETTE_KEYS, objectKeys(colors ?? "")), `${pkg.json.name} colors`).toEqual(
          [],
        );
        expect(
          missing(PALETTE_KEYS, objectKeys(darkColors ?? "")),
          `${pkg.json.name} darkColors`,
        ).toEqual([]);
      }
    }
  });

  it("includes historical --octc-shiki-* syntax tokens on every color scheme", () => {
    for (const pkg of schemes) {
      for (const config of configsIn(pkg)) {
        for (const field of ["tokens", "darkTokens"] as const) {
          const tokens = namedObject(config, field);
          const keys = tokens ? objectKeys(tokens) : [];
          const label = `${pkg.json.name} ${field}`;
          expect(missing(SHIKI_BASE, keys), label).toEqual([]);
          expect(
            keys.filter((key) => key.startsWith("shiki-token-")).length,
            label,
          ).toBeGreaterThanOrEqual(4);
          for (const key of keys.filter((item) => item.startsWith("shiki-"))) {
            expect(key, `${label} ${key}`).toMatch(/^shiki-(foreground|background|token-)/);
          }
        }
      }
    }
  });

  it("does not let skins hard-code hex or rgb palette colors in ThemeConfig", () => {
    for (const pkg of skins) {
      const configs = configsIn(pkg);
      expect(configs.length, pkg.json.name).toBeGreaterThan(0);
      for (const config of configs) {
        for (const field of ["colors", "darkColors"] as const) {
          const palette = namedObject(config, field);
          if (!palette?.trim()) {
            continue;
          }
          // CSS variables and empty/omitted fields are allowed. Hex/rgb here
          // would mean the skin owns color instead of form.
          const hardcoded = stringValues(palette).filter((value) => HARD_CODED_COLOR.test(value));
          expect(hardcoded, `${pkg.json.name} ${field}`).toEqual([]);
        }
      }
    }
  });

  it("declares @ox-content/vite-plugin as a peer on every theme package", () => {
    for (const pkg of [...schemes, ...skins]) {
      const range = pkg.json.peerDependencies?.[PLUGIN_PEER];
      expect(range, pkg.json.name).toBeTruthy();
      const meta = pkg.json.peerDependenciesMeta?.[PLUGIN_PEER];
      expect(meta === undefined || meta.optional === true, pkg.json.name).toBe(true);
    }
  });

  it("records that 3.0.0 will tighten the vite-plugin peer range", () => {
    // Published line is still 2.90.x. The 3.0.0 release PR should require >=3
    // and update this assertion — do not bump peers in this contract PR.
    for (const pkg of [...schemes, ...skins]) {
      const range = pkg.json.peerDependencies?.[PLUGIN_PEER] ?? "";
      expect(range, pkg.json.name).not.toMatch(/>=\s*3(?:\.0){0,2}(?:\s|$)|^\^?3(?:\.|$)/);
    }
  });

  it("publishes a stable export shape for every theme and theme-color package", () => {
    for (const pkg of schemes) {
      expect(pkg.json.name, pkg.dir).toMatch(/^@ox-content\/theme-color-[a-z0-9-]+$/);
      expect(pkg.json.exports?.["."], pkg.json.name).toBeTruthy();
      expect(
        pkg.json.files?.some((file) => file === "dist" || file.startsWith("dist/")),
        pkg.json.name,
      ).toBe(true);
    }
    for (const pkg of skins) {
      expect(pkg.json.name, pkg.dir).toMatch(/^@ox-content\/theme-(?!color-)[a-z0-9-]+$/);
      expect(pkg.json.exports?.["."], pkg.json.name).toBeTruthy();
      expect(
        pkg.json.files?.some((file) => file === "dist" || file.startsWith("dist/")),
        pkg.json.name,
      ).toBe(true);
    }
  });

  it("declares the Theme Presets catalog official and documents authoring", () => {
    const page = readFileSync(join(repoRoot, "docs/content/theme-presets.md"), "utf8");
    expect(page).toMatch(/official\s+catalog/i);
    expect(page).toMatch(/## Authoring a package/);
    expect(page).toMatch(/ssg\.theme/);
    expect(page).toMatch(/--octc-shiki-\*/);
    expect(page).toMatch(/must not hard-code/i);
  });
});
