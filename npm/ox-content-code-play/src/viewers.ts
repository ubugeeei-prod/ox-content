import { escapeHtml } from "./escape";
import { selectStream } from "./stdio";
import type { ConfigField, Provenance, RunResult, StdioEvent, TimingReport } from "./types";

export function renderStdioHtml(events: StdioEvent[]): string {
  if (events.length === 0) {
    return `<p class="ox-code-play__stdio ox-code-play__empty">No stdio yet.</p>`;
  }
  const lines = events
    .map((event) => {
      const time = event.timestampMs.toFixed(1);
      return `<div class="ox-code-play__stdio-line ox-code-play__stdio-line--${event.stream}"><span class="ox-code-play__stdio-meta">${escapeHtml(event.stream)} +${escapeHtml(time)}ms</span><span class="ox-code-play__stdio-text">${escapeHtml(event.text)}</span></div>`;
    })
    .join("");
  return `<div class="ox-code-play__stdio" role="log">${lines}</div>`;
}

/** Dedicated stderr viewer: stderr chunks plus error/warning diagnostics. */
export function renderStderrHtml(result: RunResult | undefined): string {
  const chunks = selectStream(result?.stdio ?? [], "stderr");
  const diagnostics = (result?.diagnostics ?? []).filter(
    (diagnostic) => diagnostic.severity === "error" || diagnostic.severity === "warning",
  );
  if (chunks.length === 0 && diagnostics.length === 0) {
    return `<p class="ox-code-play__empty">No stderr.</p>`;
  }
  const stream = chunks.length === 0 ? "" : renderStdioHtml(chunks);
  if (diagnostics.length === 0) {
    return stream;
  }
  const items = diagnostics
    .map((diagnostic) => {
      const place = diagnostic.line
        ? `:${diagnostic.line}${diagnostic.column ? `:${diagnostic.column}` : ""}`
        : "";
      return `<li class="ox-code-play__diag ox-code-play__diag--${diagnostic.severity}">${escapeHtml(diagnostic.severity)}${escapeHtml(place)} ${escapeHtml(diagnostic.message)}</li>`;
    })
    .join("");
  return `${stream}<ul class="ox-code-play__diags">${items}</ul>`;
}

export function renderConfigHtml(schema: ConfigField[], config: Record<string, unknown>): string {
  if (schema.length === 0) {
    return `<p class="ox-code-play__empty">This language has no editable config.</p>`;
  }
  const fields = schema
    .map((field) => {
      const value = config[field.key] ?? field.default ?? "";
      const label = `<label class="ox-code-play__field"><span>${escapeHtml(field.label)}</span>${renderField(field, value)}</label>`;
      return field.description
        ? `${label}<p class="ox-code-play__field-help">${escapeHtml(field.description)}</p>`
        : label;
    })
    .join("");
  return `<form class="ox-code-play__config">${fields}</form>`;
}

export function renderProvenanceHtml(provenance: Provenance | undefined): string {
  if (!provenance?.compile && !provenance?.execute) {
    return `<p class="ox-code-play__empty">No provenance yet. Run or type-check a sample.</p>`;
  }
  return `<dl class="ox-code-play__provenance">${renderLocation("Compiled", provenance.compile)}${renderLocation("Executed", provenance.execute)}</dl>`;
}

export function renderTimingHtml(timing: TimingReport | undefined): string {
  if (!timing || timing.phases.length === 0) {
    return `<p class="ox-code-play__empty">No timing yet. Run or type-check a sample.</p>`;
  }
  const rows = timing.phases
    .map((phase) => {
      const width = timing.totalMs > 0 ? Math.max(2, (phase.durationMs / timing.totalMs) * 100) : 0;
      return `<div class="ox-code-play__phase"><div class="ox-code-play__phase-meta"><span>${escapeHtml(phase.label)}</span><span>${phase.durationMs.toFixed(1)}ms</span></div><div class="ox-code-play__phase-bar" style="width:${width.toFixed(1)}%"></div></div>`;
    })
    .join("");
  return `<div class="ox-code-play__timing"><p class="ox-code-play__timing-total">Total ${timing.totalMs.toFixed(1)}ms</p>${rows}</div>`;
}

export function renderDiagnosticsHtml(result: RunResult | undefined): string {
  if (!result?.diagnostics.length) {
    return "";
  }
  const items = result.diagnostics
    .map((diagnostic) => {
      const place = diagnostic.line
        ? `:${diagnostic.line}${diagnostic.column ? `:${diagnostic.column}` : ""}`
        : "";
      return `<li class="ox-code-play__diag ox-code-play__diag--${diagnostic.severity}">${escapeHtml(diagnostic.severity)}${escapeHtml(place)} ${escapeHtml(diagnostic.message)}</li>`;
    })
    .join("");
  return `<ul class="ox-code-play__diags">${items}</ul>`;
}

function renderField(field: ConfigField, value: unknown): string {
  const name = escapeHtml(field.key);
  if (field.type === "boolean") {
    return `<input type="checkbox" name="${name}" ${value ? "checked" : ""}>`;
  }
  if (field.type === "select") {
    const options = (field.options ?? [])
      .map(
        (option) =>
          `<option value="${escapeHtml(option.value)}" ${String(value) === option.value ? "selected" : ""}>${escapeHtml(option.label)}</option>`,
      )
      .join("");
    return `<select name="${name}">${options}</select>`;
  }
  const inputType = field.type === "number" ? "number" : "text";
  return `<input type="${inputType}" name="${name}" value="${escapeHtml(String(value))}">`;
}

function renderLocation(label: string, location: Provenance["compile"]): string {
  if (!location) {
    return "";
  }
  const details = [location.runtime, location.version, location.sandbox, location.target]
    .filter(Boolean)
    .join(" · ");
  return `<div><dt>${escapeHtml(label)}</dt><dd><strong>${escapeHtml(location.host)}</strong>${details ? ` <span>${escapeHtml(details)}</span>` : ""}</dd></div>`;
}
