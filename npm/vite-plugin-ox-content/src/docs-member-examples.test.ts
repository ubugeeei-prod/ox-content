import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vite-plus/test";
import { extractDocs, generateMarkdown, resolveDocsOptions, writeDocs } from "./docs";
import type { ExtractedDocs } from "./types";

const source = `
/** An API.
 * @example interfaceExample()
 */
export interface Api {
  /** @example callExample() */
  (name: string): void;
  /** @example constructExample() */
  new (name: string): Api;
  /** @example indexExample() */
  [key: string]: unknown;
  /** A method.
   * @example methodExample()
   * @example secondMethodExample()
   */
  method(): void;
  /** @example propertyExample() */
  prop: string;
}
/** A client.
 * @example classExample()
 */
export class Client {
  /** @example constructorExample() */
  constructor() {}
  /** @example getterExample() */
  get value(): string { return ""; }
  /** @example setterExample() */
  set value(value: string) {}
}
/** Available modes. */
export enum Mode {
  /** @example enumExample() */
  Default
}
/** A function.
 * @returns The result.
 * @example functionExample()
 */
export function run(): number { return 0; }
`;

it("preserves member examples from extraction to both renderers and docs.json", async () => {
  const root = await mkdtemp(join(tmpdir(), "ox-content-member-examples-"));
  try {
    const file = join(root, "entry.ts");
    await writeFile(file, source);
    const options = resolveDocsOptions({
      entryPoints: [{ path: file, name: "entry" }],
      pathStrategy: "typedoc",
      renderStyle: "markdown",
    })!;
    const docs = await extractDocs([], options);
    const examples = docs[0]?.entries.flatMap(
      (entry) => entry.members?.flatMap((member) => member.examples ?? []) ?? [],
    );
    const members = [
      "call",
      "construct",
      "index",
      "method",
      "secondMethod",
      "property",
      "constructor",
      "getter",
      "setter",
      "enum",
    ].map((name) => `${name}Example()`);
    expect(examples?.sort()).toEqual([...members].sort());

    for (const renderStyle of ["markdown", "html"] as const) {
      const pages = generateMarkdown(docs, { ...options, renderStyle });
      const api = pages["entry/interfaces/Api.md"]!;
      const client = pages["entry/classes/Client.md"]!;
      const mode = pages["entry/enumerations/Mode.md"]!;
      for (const [page, example, firstMember] of [
        [api, "interfaceExample()", "Call Signature"],
        [client, "classExample()", "Constructors"],
      ] as const) {
        expect(page).toContain(firstMember);
        expect(page).toContain(example);
        expect(page.indexOf(example)).toBeLessThan(page.indexOf(firstMember));
      }
      for (const example of members) {
        const page =
          example === "enumExample()"
            ? mode
            : ["constructorExample()", "getterExample()", "setterExample()"].includes(example)
              ? client
              : api;
        expect(page.split(example)).toHaveLength(2);
      }
      expect(api.indexOf("methodExample()")).toBeLessThan(api.indexOf("secondMethodExample()"));
      if (renderStyle === "markdown") {
        expect(api).toContain("#### Examples");
        expect(api).toContain("### prop Examples");
      }
      const fn = pages["entry/functions/run.md"]!;
      expect(fn).toContain("Returns");
      expect(fn).toContain("functionExample()");
      expect(fn.indexOf("Returns")).toBeLessThan(fn.indexOf("functionExample()"));
    }

    await writeDocs({}, root, docs, options);
    const json = JSON.parse(await readFile(join(root, "docs.json"), "utf8")) as {
      modules: ExtractedDocs[];
      summary: { examples: number };
    };
    expect(json.summary.examples).toBe(members.length + 3);
    expect(
      json.modules[0]?.entries
        .find((entry) => entry.name === "Api")
        ?.members?.find((member) => member.name === "method")?.examples,
    ).toEqual(["methodExample()", "secondMethodExample()"]);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
