/**
 * ox-content Islands
 *
 * Framework-agnostic Island Architecture for partial hydration.
 *
 * @example
 * ```ts
 * import { initIslands } from '@ox-content/islands';
 *
 * // With Vue
 * import { createApp, h } from 'vue';
 * import Counter from './Counter.vue';
 *
 * initIslands((el, props) => {
 *   const app = createApp({ render: () => h(Counter, props) });
 *   app.mount(el);
 *   return () => app.unmount();
 * });
 * ```
 *
 * @packageDocumentation
 */

export type {
  LoadStrategy,
  IslandConfig,
  HydrateFunction,
  ComponentRegistry,
  InitIslandsOptions,
  IslandInstance,
  IslandController,
} from "./types";

export { initIslands, createDeferredInit, isIslandsSupported } from "./runtime";
export { unwrapIslandProps, stripIslandPayloadScript, readIslandSlotHtml } from "./payload";
