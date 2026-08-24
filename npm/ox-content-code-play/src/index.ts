/**
 * `@ox-content/code-play` — opt-in on-demand sample execution for Ox Content.
 *
 * @packageDocumentation
 */

export { createCodePlay } from "./client";
export type { CodePlayClient, CodePlayClientOptions } from "./client";
export { listLanguages, resolveLanguage, LANGUAGE_CATALOG } from "./catalog";
export { resolveCodePlayOptions, DEFAULT_ENDPOINTS, DEFAULT_VIEWERS } from "./config";
export type { RawCodePlayOptions, ResolvedCodePlayOptions } from "./config";
export { codePlay } from "./plugin";
export type { CodePlayPluginOptions } from "./plugin";
export { CodePlaySession } from "./session";
export { joinStream, selectStream, withStdioText } from "./stdio";
export { createFetchTransport, createMemoryTransport } from "./transport";
export { bootCodePlay } from "./boot";
export { hydrateCodePlay, mountCodePlay } from "./hydrate";
export { renderPlayUi } from "./ui";
export {
  renderConfigHtml,
  renderDiagnosticsHtml,
  renderProvenanceHtml,
  renderStderrHtml,
  renderStdioHtml,
  renderTimingHtml,
} from "./viewers";
export { parsePlayFences, parseCodePlayTags, rewritePlayFences, stripPlayMeta } from "./markdown";
export { enhancePlayHtml, enhanceGeneratedModule } from "./html";
export { encodePayload, decodePayload } from "./payload";
export { PhaseTracker } from "./timing";
export { CODE_PLAY_STYLES } from "./styles";
export type {
  AdapterResult,
  CodePlayPreset,
  ConfigField,
  Diagnostic,
  LanguageDefinition,
  LanguageEnable,
  PlayPayload,
  Provenance,
  RunResult,
  SessionInput,
  StdioEvent,
  TimingReport,
  ViewerFlags,
} from "./types";
