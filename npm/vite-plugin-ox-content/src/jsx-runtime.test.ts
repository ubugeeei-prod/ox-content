import { describe, it, expect } from "vite-plus/test";
import {
  jsx,
  jsxs,
  Fragment,
  renderToString,
  raw,
  when,
  each,
  type JSXChild,
  type JSXNode,
} from "./jsx-runtime";

describe("jsx-runtime", () => {
  describe("jsx — intrinsic elements", () => {
    it("renders a tag with text children, escaping the text", () => {
      expect(jsx("p", { children: "a < b & c" }).__html).toBe("<p>a &lt; b &amp; c</p>");
    });

    it("renders a tag with no children as an empty element", () => {
      expect(jsx("div", {}).__html).toBe("<div></div>");
    });

    it("renders numeric children without escaping", () => {
      expect(jsx("span", { children: 42 }).__html).toBe("<span>42</span>");
    });

    it("joins array children", () => {
      expect(
        jsx("ul", { children: [jsx("li", { children: "a" }), jsx("li", { children: "b" })] })
          .__html,
      ).toBe("<ul><li>a</li><li>b</li></ul>");
    });

    it("inlines nested node children without re-escaping their html", () => {
      expect(jsx("div", { children: raw("<strong>x</strong>") }).__html).toBe(
        "<div><strong>x</strong></div>",
      );
    });

    it("omits boolean/null/undefined children", () => {
      expect(jsx("div", { children: true }).__html).toBe("<div></div>");
      expect(jsx("div", { children: false }).__html).toBe("<div></div>");
      expect(jsx("div", { children: null }).__html).toBe("<div></div>");
    });
  });

  describe("jsx — attributes", () => {
    it("renders and escapes attribute values", () => {
      expect(jsx("a", { href: 'x"y', children: "link" }).__html).toBe(
        '<a href="x&quot;y">link</a>',
      );
    });

    it("maps className -> class and htmlFor -> for", () => {
      expect(jsx("label", { className: "c", htmlFor: "id", children: "L" }).__html).toBe(
        '<label class="c" for="id">L</label>',
      );
    });

    it("kebab-cases data-* and aria-* attributes", () => {
      expect(jsx("div", { dataFooBar: "1", ariaLabel: "x" }).__html).toBe(
        '<div data-foo-bar="1" aria-label="x"></div>',
      );
    });

    it("renders boolean attributes as bare names when truthy and omits them when falsy", () => {
      expect(jsx("input", { disabled: true }).__html).toBe("<input disabled />");
      expect(jsx("input", { disabled: false }).__html).toBe("<input />");
    });

    it("skips undefined, null, and false attribute values", () => {
      expect(jsx("div", { title: undefined, lang: null, contentEditable: false }).__html).toBe(
        "<div></div>",
      );
    });

    it("skips the internal key and ref props", () => {
      expect(jsx("div", { key: "k", ref: "r", id: "real" }).__html).toBe('<div id="real"></div>');
    });

    it("serializes a style object to a kebab-cased declaration string", () => {
      expect(jsx("div", { style: { backgroundColor: "red", fontSize: 12 } }).__html).toBe(
        '<div style="background-color:red;font-size:12"></div>',
      );
    });
  });

  describe("jsx — void elements", () => {
    it("self-closes void elements and ignores children", () => {
      expect(jsx("br", {}).__html).toBe("<br />");
      expect(jsx("img", { src: "a.png", alt: "x" }).__html).toBe('<img src="a.png" alt="x" />');
    });
  });

  describe("jsx — function components", () => {
    it("invokes a function component with merged props and children", () => {
      const Card = (props: Record<string, unknown>): JSXNode =>
        jsx("section", {
          children: [jsx("h2", { children: String(props.title) }), props.children as JSXChild],
        });
      expect(jsx(Card, { title: "T", children: raw("<p>body</p>") }).__html).toBe(
        "<section><h2>T</h2><p>body</p></section>",
      );
    });
  });

  describe("jsxs", () => {
    it("behaves identically to jsx", () => {
      expect(jsxs("ul", { children: [jsx("li", { children: "x" })] }).__html).toBe(
        jsx("ul", { children: [jsx("li", { children: "x" })] }).__html,
      );
    });
  });

  describe("Fragment", () => {
    it("renders children without a wrapper element", () => {
      expect(
        Fragment({ children: [jsx("span", { children: "a" }), jsx("span", { children: "b" })] })
          .__html,
      ).toBe("<span>a</span><span>b</span>");
    });

    it("renders empty for no children", () => {
      expect(Fragment({}).__html).toBe("");
    });
  });

  describe("renderToString", () => {
    it("returns the node's html", () => {
      expect(renderToString(jsx("p", { children: "hi" }))).toBe("<p>hi</p>");
    });
  });

  describe("raw", () => {
    it("passes html through unescaped", () => {
      expect(raw("<em>x</em> & y").__html).toBe("<em>x</em> & y");
    });
  });

  describe("when", () => {
    it("returns the content when the condition is true", () => {
      expect(when(true, raw("<p>shown</p>")).__html).toBe("<p>shown</p>");
    });

    it("returns empty when the condition is false", () => {
      expect(when(false, raw("<p>hidden</p>")).__html).toBe("");
    });
  });

  describe("each", () => {
    it("maps items and concatenates their html, passing the index", () => {
      expect(
        each(["a", "b"], (item, index) => jsx("li", { children: `${index}:${item}` })).__html,
      ).toBe("<li>0:a</li><li>1:b</li>");
    });

    it("renders empty for an empty array", () => {
      expect(each([], () => raw("x")).__html).toBe("");
    });
  });
});
