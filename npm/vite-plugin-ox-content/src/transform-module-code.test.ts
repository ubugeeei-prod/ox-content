import { describe, expect, it } from "vite-plus/test";
import { attachModuleCode } from "./transform-module-code";

function content() {
  return {
    code: "",
    html: "<h1>雪🌸 &quot;quoted&quot;</h1>\n",
    frontmatter: { title: "Original", nested: { value: 1 } },
    toc: [{ depth: 1, text: "Original", slug: "original", children: [] }],
    imports: [],
    exports: ["answer"],
    components: ["Island"],
    crossReferences: [],
    citations: [],
    bibliography: [],
  };
}

describe("transform module generation", () => {
  it("keeps deferred module exports consistent with the original transform snapshot", async () => {
    const input = content();
    const expected = attachModuleCode(content(), "docs/雪.md").code;
    const result = attachModuleCode(input, "docs/雪.md", true);
    input.html = "changed HTML";
    input.frontmatter.nested.value = 2;
    input.toc[0]!.text = "changed heading";
    input.components.push("OtherIsland");

    expect(result.code).toBe(expected);
    const module = await import(
      `data:text/javascript;charset=utf-8,${encodeURIComponent(result.code)}`
    );
    expect(module.html).toBe(content().html);
    expect(module.frontmatter).toEqual(content().frontmatter);
    expect(module.toc).toEqual(content().toc);
    expect(module.components).toEqual(["Island"]);
    expect(module.exports).toEqual(["answer"]);
  });

  it("preserves enumerable code when the host copies or serializes a transform", () => {
    const expected = attachModuleCode(content(), "doc.md");
    const result = attachModuleCode(content(), "doc.md", true);
    expect({ ...result }).toEqual(expected);
    expect(JSON.stringify(result)).toBe(JSON.stringify(expected));
  });

  it("lets hosts replace code before or after its first read", () => {
    const result = attachModuleCode(content(), "doc.md", true);
    result.code = "replacement";
    expect(result.code).toBe("replacement");
    result.code = "";
    expect(result.code).toBe("");
    expect(Reflect.deleteProperty(result, "code")).toBe(true);
  });
});
