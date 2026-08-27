/**
 * Island Architecture Runtime
 *
 * Framework-agnostic Island controller using pure Vanilla JavaScript.
 * No framework dependencies - works with Vue, React, Svelte, or plain JS.
 */

import { unwrapIslandProps } from "./payload";
import type {
  HydrateFunction,
  InitIslandsOptions,
  IslandConfig,
  IslandInstance,
  IslandController,
  LoadStrategy,
} from "./types";

const defaultOptions: Required<
  Omit<InitIslandsOptions, "onHydrateStart" | "onHydrateEnd" | "onHydrateError">
> &
  InitIslandsOptions = {
  rootMargin: "200px",
  threshold: 0,
  idleTimeout: 200,
  selector: "[data-ox-island]",
};

/**
 * Parse island configuration from element attributes.
 */
function parseIslandConfig(element: HTMLElement): IslandConfig {
  const component = element.dataset.oxIsland || "";
  const load = (element.dataset.oxLoad as LoadStrategy) || "eager";
  const mediaQuery = element.dataset.oxMedia;

  let props: Record<string, unknown> = {};
  try {
    const propsJson = element.dataset.oxProps;
    if (propsJson) {
      props = unwrapIslandProps(JSON.parse(propsJson));
    }
  } catch (e) {
    console.warn("[ox-islands] Failed to parse props for", element, e);
  }

  return {
    id: element.id || `island-${Math.random().toString(36).slice(2, 9)}`,
    component,
    load,
    mediaQuery,
    props,
    ssr: element.dataset.oxSsr === "true",
  };
}

/**
 * Observe element visibility using IntersectionObserver.
 */
function observeVisibility(
  element: HTMLElement,
  callback: () => void,
  options: { rootMargin: string; threshold: number },
): () => void {
  const observer = new IntersectionObserver(
    (entries) => {
      if (entries[0].isIntersecting) {
        observer.disconnect();
        callback();
      }
    },
    {
      rootMargin: options.rootMargin,
      threshold: options.threshold,
    },
  );

  observer.observe(element);

  return () => observer.disconnect();
}

/**
 * Observe media query changes.
 */
function observeMedia(query: string, callback: () => void): () => void {
  const mql = matchMedia(query);

  if (mql.matches) {
    callback();
    return () => {};
  }

  const handler = () => {
    if (mql.matches) {
      mql.removeEventListener("change", handler);
      callback();
    }
  };

  mql.addEventListener("change", handler);

  return () => mql.removeEventListener("change", handler);
}

/**
 * Schedule callback for browser idle time.
 */
function scheduleIdle(callback: () => void, timeout: number): () => void {
  if ("requestIdleCallback" in window) {
    const id = requestIdleCallback(callback, { timeout });
    return () => cancelIdleCallback(id);
  }

  // Fallback for Safari and older browsers
  const id = setTimeout(callback, timeout);
  return () => clearTimeout(id);
}

/**
 * Initialize islands with a hydration function.
 *
 * This is the main entry point for the island system.
 * Pass a hydrate function that knows how to mount your components.
 *
 * @example Vue
 * ```ts
 * import { initIslands } from '@ox-content/islands';
 * import { createApp, h } from 'vue';
 * import Counter from './Counter.vue';
 *
 * const components = { Counter };
 *
 * initIslands((el, props) => {
 *   const name = el.dataset.oxIsland!;
 *   const Component = components[name];
 *   if (!Component) return;
 *
 *   const app = createApp({ render: () => h(Component, props) });
 *   app.mount(el);
 *
 *   return () => app.unmount();
 * });
 * ```
 *
 * @example React
 * ```ts
 * import { initIslands } from '@ox-content/islands';
 * import { createRoot } from 'react-dom/client';
 * import Counter from './Counter';
 *
 * const components = { Counter };
 *
 * initIslands((el, props) => {
 *   const name = el.dataset.oxIsland!;
 *   const Component = components[name];
 *   if (!Component) return;
 *
 *   const root = createRoot(el);
 *   root.render(<Component {...props} />);
 *
 *   return () => root.unmount();
 * });
 * ```
 *
 * @example Vanilla JS
 * ```ts
 * import { initIslands } from '@ox-content/islands';
 *
 * initIslands((el, props) => {
 *   const name = el.dataset.oxIsland!;
 *
 *   if (name === 'Counter') {
 *     let count = props.initial || 0;
 *     const button = el.querySelector('button')!;
 *     const handler = () => {
 *       count++;
 *       button.textContent = String(count);
 *     };
 *     button.addEventListener('click', handler);
 *     return () => button.removeEventListener('click', handler);
 *   }
 * });
 * ```
 */
export function initIslands(
  hydrate: HydrateFunction,
  options?: InitIslandsOptions,
): IslandController {
  const opts = { ...defaultOptions, ...options };
  const instances: IslandInstance[] = [];
  const cleanups: (() => void)[] = [];

  /**
   * Hydrate a single island element.
   */
  function hydrateIsland(element: HTMLElement, config: IslandConfig): void {
    // Find existing instance
    const instance = instances.find((i) => i.element === element);
    if (instance?.hydrated) return;

    try {
      opts.onHydrateStart?.(element, config);

      // Mark as loading
      element.classList.add("ox-island-loading");

      // Call the hydrate function
      const cleanup = hydrate(element, config.props);

      // Mark as hydrated
      element.dataset.oxHydrated = "true";
      element.classList.remove("ox-island-loading");

      // Update instance
      if (instance) {
        instance.cleanup = cleanup || undefined;
        instance.hydrated = true;
      } else {
        instances.push({
          element,
          config,
          cleanup: cleanup || undefined,
          hydrated: true,
        });
      }

      opts.onHydrateEnd?.(element, config);
    } catch (error) {
      element.classList.remove("ox-island-loading");
      element.classList.add("ox-island-error");
      element.dataset.oxError = error instanceof Error ? error.message : String(error);

      opts.onHydrateError?.(
        element,
        config,
        error instanceof Error ? error : new Error(String(error)),
      );
    }
  }

  /**
   * Schedule hydration based on load strategy.
   */
  function scheduleHydration(element: HTMLElement, config: IslandConfig): void {
    switch (config.load) {
      case "eager":
        hydrateIsland(element, config);
        break;

      case "idle": {
        const cancel = scheduleIdle(() => hydrateIsland(element, config), opts.idleTimeout);
        cleanups.push(cancel);
        break;
      }

      case "visible": {
        const cancel = observeVisibility(element, () => hydrateIsland(element, config), {
          rootMargin: opts.rootMargin,
          threshold: opts.threshold,
        });
        cleanups.push(cancel);
        break;
      }

      case "media": {
        if (config.mediaQuery) {
          const cancel = observeMedia(config.mediaQuery, () => hydrateIsland(element, config));
          cleanups.push(cancel);
        } else {
          // No media query specified, fall back to eager
          hydrateIsland(element, config);
        }
        break;
      }

      default:
        hydrateIsland(element, config);
    }
  }

  // Find and process all island elements
  const elements = document.querySelectorAll<HTMLElement>(opts.selector);

  elements.forEach((element) => {
    const config = parseIslandConfig(element);

    // Track instance
    instances.push({
      element,
      config,
      hydrated: false,
    });

    // Schedule hydration
    scheduleHydration(element, config);
  });

  // Return controller
  return {
    instances,

    hydrate(element: HTMLElement): void {
      const config = parseIslandConfig(element);
      hydrateIsland(element, config);
    },

    destroy(): void {
      // Cancel pending hydrations
      cleanups.forEach((cleanup) => cleanup());
      cleanups.length = 0;

      // Cleanup hydrated instances
      instances.forEach((instance) => {
        if (instance.cleanup) {
          instance.cleanup();
        }
      });
      instances.length = 0;
    },
  };
}

/**
 * Create a deferred hydration wrapper.
 *
 * Returns a function that can be called to hydrate islands later.
 * Useful for frameworks that need to register components first.
 *
 * @example
 * ```ts
 * const deferredInit = createDeferredInit();
 *
 * // Later, after components are ready
 * const components = await loadComponents();
 * deferredInit((el, props) => {
 *   const Component = components[el.dataset.oxIsland!];
 *   // ...
 * });
 * ```
 */
export function createDeferredInit(
  options?: InitIslandsOptions,
): (hydrate: HydrateFunction) => IslandController {
  return (hydrate: HydrateFunction) => initIslands(hydrate, options);
}

/**
 * Check if islands are supported in the current environment.
 */
export function isIslandsSupported(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof document !== "undefined" &&
    typeof IntersectionObserver !== "undefined" &&
    typeof MutationObserver !== "undefined"
  );
}
