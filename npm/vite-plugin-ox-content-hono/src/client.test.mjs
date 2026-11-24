import { jsx, useState } from "hono/jsx";
import { parseHTML } from "linkedom";
import { afterEach, expect, test, vi } from "vite-plus/test";
import { initHonoIslands } from "./client.js";
import { useFrontmatter } from "./context.js";

function setupDocument(html) {
  const { window, document } = parseHTML(`<html><body>${html}</body></html>`);
  vi.stubGlobal("window", window);
  vi.stubGlobal("document", document);
  vi.stubGlobal("Node", window.Node);
  vi.stubGlobal("Element", window.Element);
  vi.stubGlobal("requestAnimationFrame", (callback) =>
    setTimeout(() => callback(performance.now()), 0),
  );
  return document;
}

function marker(title, load = "eager") {
  return `<script type="application/json" data-ox-hono-context>${JSON.stringify({
    frontmatter: { title },
    load,
  })}</script>`;
}

afterEach(() => vi.unstubAllGlobals());

test("remounts with selected frontmatter and children props after the SSR marker is gone", async () => {
  const document = setupDocument(
    `<div data-ox-island="Example" data-ox-module="/example" data-ox-ssr="true"
      data-ox-props='{"children":"Passed in props"}'>${marker("Selected")}</div>`,
  );
  const options = {
    modules: {
      "/example": async () => ({
        default: ({ children }) => jsx("p", null, useFrontmatter().title + ": " + children),
      }),
    },
  };
  const element = document.querySelector("[data-ox-island]");
  for (let pass = 0; pass < 2; pass += 1) {
    const controller = initHonoIslands(options);
    try {
      await vi.waitFor(() => expect(controller.instances[0]?.hydrated).toBe(true));
      expect(element.textContent).toBe("Selected: Passed in props");
      expect(element.querySelector("[data-ox-hono-context]")).toBeNull();
    } finally {
      controller.destroy();
    }
  }
});

test("schedules the live nested wrapper after the parent replaces its static slot", async () => {
  const slot = `<div data-ox-island="Inner" data-ox-module="/inner" data-ox-ssr="true">
    <button>SSR child</button>${marker("Nested", "visible")}</div>`;
  const document = setupDocument(
    `<div data-ox-island="Outer" data-ox-module="/outer" data-ox-ssr="true">
      <article>${slot}</article>${marker("Outer")}</div>`,
  );
  const outer = document.querySelector("[data-ox-island=Outer]");
  outer.dataset.oxContent = slot;
  const original = outer.querySelector("[data-ox-island=Inner]");
  let observer;
  vi.stubGlobal(
    "IntersectionObserver",
    class {
      constructor(callback) {
        this.callback = callback;
        observer = this;
      }
      observe(element) {
        this.element = element;
      }
      disconnect() {}
    },
  );
  const loadInner = vi.fn(async () => ({
    default() {
      const [count, setCount] = useState(0);
      return jsx(
        "button",
        { onClick: () => setCount(count + 1) },
        useFrontmatter().title,
        " ",
        count,
      );
    },
  }));
  const controller = initHonoIslands({
    modules: {
      "/outer": async () => ({ default: ({ children }) => jsx("article", null, children) }),
      "/inner": loadInner,
    },
  });
  try {
    await vi.waitFor(() => expect(observer?.element).toBeDefined());
    const live = outer.querySelector("[data-ox-island=Inner]");
    expect(observer.element).toBe(live);
    expect(live).not.toBe(original);
    expect(loadInner).not.toHaveBeenCalled();
    observer.callback([{ isIntersecting: true }]);
    await vi.waitFor(() => expect(controller.instances[1]?.hydrated).toBe(true));
    const button = live.querySelector("button");
    expect(button.textContent).toBe("Nested 0");
    button.click();
    await vi.waitFor(() => expect(button.textContent).toBe("Nested 1"));
  } finally {
    controller.destroy();
  }
  expect(outer.children).toHaveLength(0);
});
