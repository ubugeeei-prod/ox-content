/**
 * Island Architecture Types
 *
 * Framework-agnostic type definitions for the Island system.
 */

/**
 * Loading strategy for islands.
 *
 * - `eager`: Hydrate immediately on page load
 * - `idle`: Hydrate when the browser is idle (requestIdleCallback)
 * - `visible`: Hydrate when the element becomes visible (IntersectionObserver)
 * - `media`: Hydrate when a media query matches
 */
export type LoadStrategy = "eager" | "idle" | "visible" | "media";

/**
 * Island configuration extracted from data attributes.
 */
export interface IslandConfig {
  /** Unique island identifier */
  id: string;
  /** Component name to hydrate */
  component: string;
  /** Loading strategy */
  load: LoadStrategy;
  /** Media query for 'media' load strategy */
  mediaQuery?: string;
  /** Component props (JSON serialized in data-ox-props) */
  props: Record<string, unknown>;
  /** Whether the island wrapper already contains server-rendered component HTML. */
  ssr?: boolean;
}

/**
 * Hydration function signature.
 *
 * Called when an island should be hydrated.
 * Returns an optional cleanup function.
 */
export type HydrateFunction = (
  element: HTMLElement,
  props: Record<string, unknown>,
) => void | (() => void);

/**
 * Component registry mapping component names to hydrate functions.
 */
export type ComponentRegistry = Map<string, HydrateFunction>;

/**
 * Options for initializing islands.
 */
export interface InitIslandsOptions {
  /** Root margin for IntersectionObserver (visible strategy). Default: "200px" */
  rootMargin?: string;
  /** Threshold for IntersectionObserver. Default: 0 */
  threshold?: number;
  /** Timeout for idle callback fallback in ms. Default: 200 */
  idleTimeout?: number;
  /** Custom selector for finding islands. Default: "[data-ox-island]" */
  selector?: string;
  /** Called when an island starts hydrating */
  onHydrateStart?: (element: HTMLElement, config: IslandConfig) => void;
  /** Called when an island finishes hydrating */
  onHydrateEnd?: (element: HTMLElement, config: IslandConfig) => void;
  /** Called when hydration fails */
  onHydrateError?: (element: HTMLElement, config: IslandConfig, error: Error) => void;
}

/**
 * Island instance tracking.
 */
export interface IslandInstance {
  element: HTMLElement;
  config: IslandConfig;
  cleanup?: () => void;
  hydrated: boolean;
}

/**
 * Island controller returned by initIslands.
 */
export interface IslandController {
  /** All tracked island instances */
  instances: IslandInstance[];
  /** Manually hydrate a specific island */
  hydrate: (element: HTMLElement) => void;
  /** Destroy all islands and cleanup */
  destroy: () => void;
}
