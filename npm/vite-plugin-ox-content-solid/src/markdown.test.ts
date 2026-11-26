import { describe, expect, it } from "vite-plus/test";
import { injectIslandMarkers, scanComponents } from "./markdown";

function decodeAttr(value: string): string {
  return value
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&");
}

describe("injectIslandMarkers", () => {
  it("writes island props and content so the attribute decodes to the source", () => {
    const source = `<Counter label="a &quot; b's"><b>&lt;img src=x&gt; & 'q'</b></Counter>`;
    const scanned = scanComponents(source, { Counter: "./Counter.tsx" });
    const html = injectIslandMarkers(`<p>${scanned.content}</p>`, scanned.islands);

    const props = /data-ox-props='([^']*)'/.exec(html)?.[1];
    const content = /data-ox-content='([^']*)'/.exec(html)?.[1];
    expect(props).toBeDefined();
    expect(content).toBeDefined();
    expect(JSON.parse(decodeAttr(props!))).toEqual({ label: "a &quot; b's" });
    expect(decodeAttr(content!)).toBe("<b>&lt;img src=x&gt; & 'q'</b>");
    expect(content).not.toContain("<");
  });
});
