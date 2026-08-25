const RUST_PAYLOAD_KEYS = new Set(["props", "expressions", "spreads"]);
const ISLAND_JSON_SCRIPT = /^\s*<script type="application\/json">[\s\S]*?<\/script>/;

/**
 * Flatten the Rust MDX island payload (`{ props, expressions, spreads }`)
 * into the literal props object hydrate functions expect. Legacy flat
 * `data-ox-props` objects from the Markdown regex path are returned as-is.
 */
export function unwrapIslandProps(parsed: unknown): Record<string, unknown> {
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    return {};
  }

  const record = parsed as Record<string, unknown>;
  const keys = Object.keys(record);
  if (
    keys.length > 0 &&
    keys.every((key) => RUST_PAYLOAD_KEYS.has(key)) &&
    record.props &&
    typeof record.props === "object" &&
    !Array.isArray(record.props)
  ) {
    return record.props as Record<string, unknown>;
  }

  return record;
}

/**
 * Slot markup for an island: `data-ox-content` when the regex path set it,
 * otherwise inner HTML with the Rust JSON payload script stripped.
 */
export function stripIslandPayloadScript(innerHTML: string): string {
  return innerHTML.replace(ISLAND_JSON_SCRIPT, "");
}

export function readIslandSlotHtml(element: Pick<HTMLElement, "dataset" | "innerHTML">): string {
  const fromAttr = element.dataset.oxContent;
  if (fromAttr) return fromAttr;
  return stripIslandPayloadScript(element.innerHTML);
}
