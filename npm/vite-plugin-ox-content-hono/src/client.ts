import { jsx, useState, type Child } from "hono/jsx";
import { render, flushSync } from "hono/jsx/dom";
import {
  createHtmlHostLazyHydrate,
  type CreateHtmlHostLazyHydrateInput,
} from "@ox-content/islands/html-host";
import { initIslands } from "@ox-content/islands/runtime";
import { FrontmatterContext, type Frontmatter } from "./context.js";
import { componentProps, type IslandContext } from "./shared.js";

type InitIslandsOptions = NonNullable<Parameters<typeof initIslands>[1]>;
type IslandController = ReturnType<typeof initIslands>;

// Keep the context on the wrapper even if HMR replaces this module.
const contextKey = Symbol.for("@ox-content/vite-plugin-hono/island-context");
const batchAttribute = "data-ox-hono-batch";
let nextBatch = 0;

function readContext(
  element: HTMLElement & { [contextKey]?: IslandContext },
): IslandContext | undefined {
  const marker = Array.from(element.children).find((node) =>
    node.matches("script[data-ox-hono-context]"),
  );
  if (marker) element[contextKey] = JSON.parse(marker.textContent ?? "{}");
  return element[contextKey];
}

export type HonoClientOptions = Pick<CreateHtmlHostLazyHydrateInput, "modules" | "onError">;

/** Convert only top-level static slot nodes; their inner markup stays host-owned. */
function slotChildren(html: string) {
  const template = document.createElement("template");
  template.innerHTML = html;
  return Array.from(template.content.childNodes).flatMap<Child>((node) => {
    if (node.nodeType === Node.TEXT_NODE) return [node.textContent ?? ""];
    if (!(node instanceof Element)) return [];
    const props: Record<string, unknown> = {};
    for (const attr of node.attributes) {
      props[attr.name] = [
        "checked",
        "selected",
        "disabled",
        "multiple",
        "readonly",
        "required",
      ].includes(attr.name)
        ? true
        : attr.value;
    }
    props.dangerouslySetInnerHTML = { __html: node.innerHTML };
    return [jsx(node.localName, props)];
  });
}

/** Mounts into the SSR wrapper when Ox Content selects the island's strategy. */
export function createHonoHydrate(options: HonoClientOptions) {
  return createHtmlHostLazyHydrate({
    ...options,
    frameworkName: "Hono JSX",
    eventName: "ox-content-hono:error",
    // The renderer reads the context before Hono replaces the SSR children.
    preserveElementContents: true,
    async render({ component, props, element, slotHtml }) {
      if (typeof component !== "function")
        throw new TypeError("Hono islands must export a function component");
      const Component = component;
      const frontmatter: Frontmatter = readContext(element)?.frontmatter ?? {};
      if (!frontmatter || Array.isArray(frontmatter) || typeof frontmatter !== "object") {
        throw new TypeError("Invalid Hono island frontmatter");
      }
      const children = slotHtml === undefined ? props.children : slotChildren(slotHtml);
      // Updating the same root runs Hono's effect/ref cleanup. Calling render()
      // again would create a separate root and only replace the DOM children.
      let unmount = () => {};
      function MountBoundary() {
        const [active, setActive] = useState(true);
        unmount = () => flushSync(() => setActive(false));
        return active
          ? jsx(
              FrontmatterContext.Provider,
              { value: frontmatter },
              jsx(Component, { ...componentProps(props), children }),
            )
          : null;
      }
      render(jsx(MountBoundary, {}), element);
      // Hono schedules initial effects in requestAnimationFrame. Complete the
      // mount after that frame so even immediate disposal can clean them up.
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
      return () => unmount();
    },
  });
}

export function initHonoIslands(
  options: HonoClientOptions & { options?: InitIslandsOptions },
): IslandController {
  const hydrate = createHonoHydrate(options);
  const selector = options.options?.selector ?? "[data-ox-hono]";
  const controllers: IslandController[] = [];
  let destroyed = false;

  function initialize(root: Document | HTMLElement) {
    for (const element of root.querySelectorAll<HTMLElement>("[data-ox-island]")) {
      const context = readContext(element);
      if (!context) continue;
      element.dataset.oxHono = "";
      element.dataset.oxLoad = context.load;
      if (context.media) element.dataset.oxMedia = context.media;
    }
    // A parent replaces its SSR children. Schedule only the outermost wrappers
    // in this subtree and discover the live children once their parent mounts.
    const elements = Array.from(root.querySelectorAll<HTMLElement>(selector)).filter((element) => {
      const ancestor = element.parentElement?.closest(selector);
      return !ancestor || ancestor === root || !root.contains(ancestor);
    });
    if (elements.length === 0 && root !== document) return;

    // The islands runtime queries the document once per controller.
    const batch = String(++nextBatch);
    for (const element of elements) element.setAttribute(batchAttribute, batch);
    try {
      const controller = initIslands(hydrate, {
        ...options.options,
        selector: `[${batchAttribute}="${batch}"]`,
        onHydrateEnd(element, config) {
          options.options?.onHydrateEnd?.(element, config);
          if (!destroyed) initialize(element);
        },
      });
      if (destroyed) controller.destroy();
      else controllers.push(controller);
    } finally {
      for (const element of elements) element.removeAttribute(batchAttribute);
    }
  }

  initialize(document);
  return {
    get instances() {
      return controllers.flatMap((controller) => controller.instances);
    },
    hydrate(element) {
      if (destroyed) return;
      const controller = controllers.find((item) =>
        item.instances.some((instance) => instance.element === element),
      );
      (controller ?? controllers[0]).hydrate(element);
    },
    destroy() {
      destroyed = true;
      for (const controller of controllers.reverse()) controller.destroy();
      controllers.length = 0;
    },
  };
}
