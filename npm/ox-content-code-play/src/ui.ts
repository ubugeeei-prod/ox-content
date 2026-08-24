import { resolveLanguage } from "./catalog";
import { escapeHtml } from "./escape";
import type { CodePlayPreset, PlayPayload, RunResult } from "./types";
import {
  renderConfigHtml,
  renderDiagnosticsHtml,
  renderProvenanceHtml,
  renderStderrHtml,
  renderStdioHtml,
  renderTimingHtml,
} from "./viewers";

export interface UiState {
  payload: PlayPayload;
  result?: RunResult;
  busy?: boolean;
  panel?: "stdio" | "stderr" | "config" | "provenance" | "timing";
}

export function renderPlayUi(state: UiState): string {
  const definition = resolveLanguage(state.payload.language);
  const preset: CodePlayPreset = state.payload.ui === "headless" ? "headless" : state.payload.ui;
  if (preset === "headless") {
    return "";
  }
  const panel = state.panel ?? "stdio";
  const canTypecheck = state.payload.capabilities.typecheck;
  const tabs = renderTabs(state, panel);
  const viewers = state.payload.viewers;
  return `<div class="ox-code-play ox-code-play--${preset}" data-ox-code-play-ui>
  <div class="ox-code-play__toolbar">
    <span class="ox-code-play__lang">${escapeHtml(definition?.name ?? state.payload.language)}</span>
    <button type="button" data-ox-action="run"${state.busy ? " disabled" : ""}>Run</button>
    ${canTypecheck ? `<button type="button" data-ox-action="typecheck"${state.busy ? " disabled" : ""}>Typecheck</button>` : ""}
  </div>
  <div class="ox-code-play__source"></div>
  ${tabs}
  ${viewers.stdio ? `<div class="ox-code-play__panel" data-panel="stdio">${renderDiagnosticsHtml(state.result)}${renderStdioHtml(state.result?.stdio ?? [])}</div>` : ""}
  ${viewers.stderr ? `<div class="ox-code-play__panel" data-panel="stderr"${hidden(panel, "stderr", preset)}>${renderStderrHtml(state.result)}</div>` : ""}
  <div class="ox-code-play__panel" data-panel="config"${hidden(panel, "config", preset)}>${renderConfigHtml(definition?.configSchema ?? [], state.payload.config)}</div>
  <div class="ox-code-play__panel" data-panel="provenance"${hidden(panel, "provenance", preset)}>${renderProvenanceHtml(state.result?.provenance)}</div>
  <div class="ox-code-play__panel" data-panel="timing"${hidden(panel, "timing", preset)}>${renderTimingHtml(state.result?.timing)}</div>
</div>`;
}

function renderTabs(state: UiState, panel: string): string {
  if (state.payload.ui === "compact") {
    return "";
  }
  const viewers = state.payload.viewers;
  const buttons = [
    viewers.stdio ? tab("stdio", "stdio", panel) : "",
    viewers.stderr ? tab("stderr", "stderr", panel) : "",
    viewers.config ? tab("config", "config", panel) : "",
    viewers.provenance ? tab("provenance", "provenance", panel) : "",
    viewers.timing ? tab("timing", "timing", panel) : "",
  ]
    .filter(Boolean)
    .join("");
  return buttons ? `<div class="ox-code-play__tabs" role="tablist">${buttons}</div>` : "";
}

function tab(id: string, label: string, selected: string): string {
  return `<button type="button" role="tab" data-ox-panel="${id}" aria-selected="${selected === id ? "true" : "false"}">${label}</button>`;
}

function hidden(current: string, id: string, preset: CodePlayPreset): string {
  if (preset === "compact" && (id === "stdio" || id === "stderr")) {
    return "";
  }
  return current === id ? "" : " hidden";
}
