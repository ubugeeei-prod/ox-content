import { createRequire } from "node:module";
import { runInThisContext } from "node:vm";
import { afterEach, describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };
import {
  applyThemeBootstrap,
  createThemeBootstrapScript,
  renderThemeBootstrapScript,
  resolveThemeBootstrapState,
  setThemeBootstrapPreference,
} from "./theme-bootstrap";

const require = createRequire(import.meta.url);
const originalDocument = Object.getOwnPropertyDescriptor(globalThis, "document");
const originalLocalStorage = Object.getOwnPropertyDescriptor(globalThis, "localStorage");
const originalMatchMedia = Object.getOwnPropertyDescriptor(globalThis, "matchMedia");

afterEach(() => {
  restoreGlobal("document", originalDocument);
  restoreGlobal("localStorage", originalLocalStorage);
  restoreGlobal("matchMedia", originalMatchMedia);
});

describe("theme bootstrap public API", () => {
  it("declares a standalone package subpath", () => {
    const exported = (packageJson.exports as unknown as Record<string, PackageConditionalExport>)[
      "./theme-bootstrap"
    ];
    expect(exported.import.types).toBe("./dist/theme-bootstrap.d.mts");
    expect(exported.import.default).toBe("./dist/theme-bootstrap.mjs");
    expect(exported.require.types).toBe("./dist/theme-bootstrap.d.cts");
    expect(exported.require.default).toBe("./dist/theme-bootstrap.cjs");

    const entries: string[] = require("../vite.config.ts").default.pack.entry;
    expect(entries).toContain("src/theme-bootstrap.ts");
  });

  it("lets explicit light and dark preferences win over the OS", () => {
    expect(resolveThemeBootstrapState("light", true)).toEqual({
      preference: "light",
      theme: "light",
      source: "storage",
    });
    expect(resolveThemeBootstrapState("dark", false)).toEqual({
      preference: "dark",
      theme: "dark",
      source: "storage",
    });
  });

  it("uses the configured fallback when storage is absent or invalid", () => {
    expect(resolveThemeBootstrapState(null, true, { defaultPreference: "system" })).toEqual({
      preference: "system",
      theme: "dark",
      source: "fallback",
    });
    expect(resolveThemeBootstrapState("sepia", true, { defaultPreference: "light" })).toEqual({
      preference: "light",
      theme: "light",
      source: "fallback",
    });
  });

  it("survives throwing localStorage and still applies root class and data", () => {
    const dom = installDom({ throwsGet: true, prefersDark: true });

    const state = applyThemeBootstrap({
      storageKey: "preferred-theme",
      rootSelector: "#app",
      darkClass: "is-dark",
      lightClass: "is-light",
      themeAttribute: "data-mode",
    });

    expect(state).toMatchObject({ preference: "system", theme: "dark", source: "fallback" });
    expect(dom.root.attributes.get("data-mode")).toBe("dark");
    expect(dom.root.classes.has("is-dark")).toBe(true);
    expect(dom.root.classes.has("is-light")).toBe(false);
  });

  it("shares the same root and storage contract with later theme transitions", () => {
    const dom = installDom({ stored: "light", prefersDark: false });

    const state = setThemeBootstrapPreference("dark", {
      storageKey: "theme",
      darkClass: "is-dark",
      themeAttribute: "data-theme",
    });

    expect(state.theme).toBe("dark");
    expect(dom.storage.get("theme")).toBe("dark");
    expect(dom.documentElement.attributes.get("data-theme")).toBe("dark");
    expect(dom.documentElement.classes.has("is-dark")).toBe(true);
  });

  it("serializes custom configuration without injectable closing script tags", () => {
    const script = createThemeBootstrapScript({
      storageKey: 'theme"></script><script>alert(1)</script>',
      rootSelector: 'html[data-x="</script>"]',
    });
    const tag = renderThemeBootstrapScript(
      { storageKey: "theme" },
      { id: 'theme"</script>', nonce: 'nonce"value' },
    );

    expect(script.toLowerCase()).not.toContain("</script>");
    expect(script).toContain("\\u003C/script");
    expect(tag).toContain('id="theme&quot;&lt;/script&gt;"');
    expect(tag).toContain('nonce="nonce&quot;value"');
    expect(tag.toLowerCase().replace("</script>", "")).not.toContain("</script>");
  });

  it("the emitted bootstrap applies before stylesheets run", () => {
    const dom = installDom({ stored: "dark", prefersDark: false });
    runInThisContext(
      createThemeBootstrapScript({ darkClass: "dark", themeAttribute: "data-theme" }),
    );

    expect(dom.documentElement.attributes.get("data-theme")).toBe("dark");
    expect(dom.documentElement.classes.has("dark")).toBe(true);
  });
});

interface PackageConditionalExport {
  import: {
    types: string;
    default: string;
  };
  require: {
    types: string;
    default: string;
  };
}

function installDom(input: { stored?: string | null; throwsGet?: boolean; prefersDark: boolean }) {
  const storage = new Map<string, string>();
  if (input.stored != null) {
    storage.set("theme", input.stored);
  }
  const documentElement = elementStub();
  const root = elementStub();
  const document = {
    documentElement,
    querySelector(selector: string) {
      return selector === "#app" ? root : documentElement;
    },
  };
  defineGlobal("document", document);
  defineGlobal("localStorage", {
    getItem(key: string) {
      if (input.throwsGet) {
        throw new Error("blocked");
      }
      return storage.get(key) ?? null;
    },
    setItem(key: string, value: string) {
      storage.set(key, value);
    },
  });
  defineGlobal("matchMedia", () => ({ matches: input.prefersDark }));
  return { documentElement, root, storage };
}

function elementStub() {
  const attributes = new Map<string, string>();
  const classes = new Set<string>();
  return {
    attributes,
    classes,
    setAttribute(name: string, value: string) {
      attributes.set(name, value);
    },
    classList: {
      toggle(name: string, force: boolean) {
        if (force) {
          classes.add(name);
        } else {
          classes.delete(name);
        }
        return force;
      },
    },
  };
}

function defineGlobal(name: string, value: unknown): void {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    value,
  });
}

function restoreGlobal(name: string, descriptor: PropertyDescriptor | undefined): void {
  if (descriptor) {
    Object.defineProperty(globalThis, name, descriptor);
  } else {
    Reflect.deleteProperty(globalThis, name);
  }
}
