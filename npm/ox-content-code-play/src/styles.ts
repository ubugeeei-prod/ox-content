export const CODE_PLAY_STYLES = `
.ox-code-play {
  border: 1px solid var(--octc-color-border, color-mix(in srgb, currentColor 16%, transparent));
  border-radius: 12px;
  background: var(--octc-color-bg-alt, var(--octc-color-bg, Canvas));
  color: var(--octc-color-text, CanvasText);
  overflow: hidden;
  margin: 1.25rem 0;
}
.ox-code-play__toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
  padding: 0.6rem 0.8rem;
  border-bottom: 1px solid var(--octc-color-border, color-mix(in srgb, currentColor 12%, transparent));
}
.ox-code-play__lang {
  font: 600 0.8rem/1.2 ui-sans-serif, system-ui, sans-serif;
  margin-right: auto;
  color: var(--octc-color-text, CanvasText);
}
.ox-code-play__toolbar button {
  appearance: none;
  border: 1px solid var(--octc-color-border, color-mix(in srgb, currentColor 20%, transparent));
  background: color-mix(in srgb, var(--octc-color-text, CanvasText) 8%, transparent);
  color: var(--octc-color-text, CanvasText);
  border-radius: 8px;
  padding: 0.25rem 0.75rem;
  font: 600 0.75rem/1.4 ui-sans-serif, system-ui, sans-serif;
  cursor: pointer;
}
.ox-code-play__toolbar button:disabled { opacity: 0.55; cursor: progress; }
.ox-code-play__runtime {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  align-items: center;
  padding: 0.55rem 0.8rem;
  border-bottom: 1px solid var(--octc-color-border, color-mix(in srgb, currentColor 10%, transparent));
  background: color-mix(in srgb, var(--octc-color-text, CanvasText) 4%, transparent);
}
.ox-code-play__runtime-chip {
  display: inline-grid;
  gap: 0.1rem;
  min-width: 7.5rem;
  padding: 0.35rem 0.5rem;
  border: 1px solid var(--octc-color-border, color-mix(in srgb, currentColor 14%, transparent));
  border-radius: 8px;
  background: var(--octc-color-bg, Canvas);
}
.ox-code-play__runtime-chip span {
  font: 600 0.62rem/1.1 ui-sans-serif, system-ui, sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  opacity: 0.62;
}
.ox-code-play__runtime-chip strong {
  font: 650 0.78rem/1.2 ui-sans-serif, system-ui, sans-serif;
}
.ox-code-play__runtime-chip--ok strong { color: var(--octc-color-primary, var(--octc-accent, #4f46e5)); }
.ox-code-play__runtime-chip--warn strong { color: var(--octc-warning, #b54708); }
.ox-code-play__runtime-chip--muted strong { opacity: 0.72; }
.ox-code-play .ox-code { margin: 0; }
.ox-code-play__source pre { margin: 0; border: 0; border-radius: 0; }
.ox-code-play__tabs {
  display: flex;
  gap: 0.25rem;
  padding: 0.4rem 0.7rem 0;
}
.ox-code-play__tabs button {
  appearance: none;
  border: 0;
  background: transparent;
  color: var(--octc-color-text, CanvasText);
  padding: 0.35rem 0.55rem;
  border-radius: 8px 8px 0 0;
  font: 600 0.75rem/1.2 ui-sans-serif, system-ui, sans-serif;
  cursor: pointer;
}
.ox-code-play__tabs button[aria-selected="true"] {
  background: color-mix(in srgb, var(--octc-color-text, CanvasText) 10%, transparent);
}
.ox-code-play__panel { padding: 0.7rem 0.8rem 0.9rem; color: var(--octc-color-text, CanvasText); }
.ox-code-play__panel pre {
  background: transparent !important;
  color: inherit !important;
  padding: 0 !important;
  margin: 0 !important;
  border: 0 !important;
  border-radius: 0 !important;
}
.ox-code-play__empty { margin: 0; opacity: 0.7; font-size: 0.85rem; }
.ox-code-play__stdio { font: 12px/1.45 ui-monospace, SFMono-Regular, monospace; }
.ox-code-play__stdio-line { display: grid; grid-template-columns: 8.5rem 1fr; gap: 0.6rem; white-space: pre-wrap; }
.ox-code-play__stdio-line--stderr { color: var(--octc-danger, #b42318); }
.ox-code-play__stdio-line--stdin { opacity: 0.75; }
.ox-code-play__stdio-meta { opacity: 0.6; }
.ox-code-play__config { display: grid; gap: 0.55rem; }
.ox-code-play__field { display: grid; gap: 0.25rem; font-size: 0.85rem; }
.ox-code-play__field input, .ox-code-play__field select {
  font: inherit;
  padding: 0.3rem 0.45rem;
  border-radius: 8px;
  border: 1px solid var(--octc-color-border, color-mix(in srgb, currentColor 18%, transparent));
  background: var(--octc-color-bg, Canvas);
  color: var(--octc-color-text, CanvasText);
}
.ox-code-play__provenance { display: grid; gap: 0.6rem; margin: 0; }
.ox-code-play__provenance dt { font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.04em; opacity: 0.65; }
.ox-code-play__provenance dd { margin: 0.15rem 0 0; }
.ox-code-play__phase { margin: 0.45rem 0; }
.ox-code-play__phase-meta { display: flex; justify-content: space-between; font-size: 0.8rem; }
.ox-code-play__phase-bar {
  height: 0.35rem;
  margin-top: 0.25rem;
  border-radius: 999px;
  background: var(--octc-color-primary, var(--octc-accent, #4f46e5));
}
.ox-code-play__timing-total { margin: 0 0 0.4rem; font-weight: 600; }
.ox-code-play__diags { margin: 0 0 0.6rem; padding-left: 1.1rem; }
.ox-code-play__diag--error { color: var(--octc-danger, #b42318); }
.ox-code-play__diag--warning { color: var(--octc-warning, #b54708); }
.ox-code-play--compact .ox-code-play__tabs { display: none; }
.ox-code-play--compact .ox-code-play__panel[data-panel]:not([data-panel="stdio"]):not([data-panel="stderr"]) { display: none; }
`;
