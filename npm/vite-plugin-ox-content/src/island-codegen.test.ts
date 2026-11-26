import { describe, expect, it } from "vite-plus/test";
import type { ResolvedDocumentComponentImport } from "./document-imports";
import { renderIslandComponentImports } from "./island-codegen";

function binding(
  overrides: Partial<ResolvedDocumentComponentImport>,
): ResolvedDocumentComponentImport {
  return {
    localName: "Counter",
    specifier: "./Counter.js",
    resolvedPath: "/repo/docs/Counter.js",
    importPathRelativeToDocument: "./Counter.js",
    imported: "default",
    kind: "default",
    ...overrides,
  };
}

function render(bindings: ResolvedDocumentComponentImport[]): string {
  return renderIslandComponentImports(
    bindings.map((entry) => entry.localName),
    {
      globalComponents: {},
      localBindings: new Map(bindings.map((entry) => [entry.localName, entry])),
      documentPath: "/repo/docs/page.mdx",
      root: "/repo",
    },
  );
}

describe("renderIslandComponentImports", () => {
  it("keeps plain document imports unchanged", () => {
    expect(
      render([
        binding({}),
        binding({ localName: "Plot", imported: "Chart", kind: "named" }),
        binding({ localName: "Chart", imported: "Chart", kind: "named" }),
      ]),
    ).toBe(
      [
        "import Counter from './Counter.js';",
        "import { Chart as Plot } from './Counter.js';",
        "import { Chart } from './Counter.js';",
      ].join("\n"),
    );
  });

  it("quotes module paths that contain quote characters", () => {
    const path = "./Counter.js';globalThis.ran=1;'";
    const code = render([binding({ importPathRelativeToDocument: path })]);

    expect(code).toBe("import Counter from './Counter.js\\';globalThis.ran=1;\\'';");
  });

  it("writes string import names as string literals", () => {
    const imported = "Named } from './Counter.js'; globalThis.ran=1; import { Named";
    const code = render([binding({ localName: "Counter2", imported, kind: "named" })]);

    expect(code).toBe(
      "import { 'Named } from \\'./Counter.js\\'; globalThis.ran=1; import { Named' as Counter2 } from './Counter.js';",
    );
  });

  it("skips bindings whose local name is not an identifier", () => {
    const code = render([binding({ localName: "A } from 'x'; run(); import { B" })]);

    expect(code).toBe("");
  });
});
