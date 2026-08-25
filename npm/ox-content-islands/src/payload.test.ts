import { describe, expect, it } from "vite-plus/test";
import { readIslandSlotHtml, stripIslandPayloadScript, unwrapIslandProps } from "./payload";

describe("unwrapIslandProps", () => {
  it("flattens the Rust MDX payload", () => {
    expect(
      unwrapIslandProps({
        props: { tone: "info", count: 42 },
        expressions: { title: "foo" },
        spreads: ["...rest"],
      }),
    ).toEqual({ tone: "info", count: 42 });
  });

  it("leaves the regex-path flat props object unchanged", () => {
    expect(unwrapIslandProps({ tone: "info", active: true })).toEqual({
      tone: "info",
      active: true,
    });
  });
});

describe("stripIslandPayloadScript", () => {
  it("removes the leading JSON payload script", () => {
    expect(
      stripIslandPayloadScript(
        '<script type="application/json">{"props":{"tone":"info"}}</script><p>Hi</p>',
      ),
    ).toBe("<p>Hi</p>");
  });
});

describe("readIslandSlotHtml", () => {
  it("prefers data-ox-content from the regex path", () => {
    expect(
      readIslandSlotHtml({
        dataset: { oxContent: "Read **docs**." },
        innerHTML: "<p>ignored</p>",
      }),
    ).toBe("Read **docs**.");
  });

  it("strips the Rust payload script from inner HTML", () => {
    expect(
      readIslandSlotHtml({
        dataset: {},
        innerHTML: '<script type="application/json">{"props":{}}</script><strong>ok</strong>',
      }),
    ).toBe("<strong>ok</strong>");
  });
});
