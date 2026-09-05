export type ThemeBootstrapPreference = "light" | "dark" | "system";
export type ThemeBootstrapResolvedTheme = "light" | "dark";
export type ThemeBootstrapSource = "storage" | "fallback";

export interface ThemeBootstrapOptions {
  /**
   * Storage key read before first paint. Set to `false` to disable storage.
   *
   * @default "theme"
   */
  storageKey?: string | false;
  /**
   * Preference used when storage is empty, invalid, or unavailable.
   *
   * @default "system"
   */
  defaultPreference?: ThemeBootstrapPreference;
  /**
   * Element selector receiving theme attributes and classes.
   *
   * @default ":root"
   */
  rootSelector?: string;
  /**
   * Class toggled on the root when the resolved theme is dark.
   *
   * @default "dark"
   */
  darkClass?: string | false;
  /**
   * Optional class toggled on the root when the resolved theme is light.
   *
   * @default false
   */
  lightClass?: string | false;
  /**
   * Attribute set to the resolved theme. Set to `false` to skip it.
   *
   * @default "data-theme"
   */
  themeAttribute?: string | false;
}

export interface ResolvedThemeBootstrapOptions {
  storageKey: string | false;
  defaultPreference: ThemeBootstrapPreference;
  rootSelector: string;
  darkClass: string | false;
  lightClass: string | false;
  themeAttribute: string | false;
}

export interface ThemeBootstrapState {
  preference: ThemeBootstrapPreference;
  theme: ThemeBootstrapResolvedTheme;
  source: ThemeBootstrapSource;
}

export interface RenderThemeBootstrapScriptOptions {
  /** CSP nonce for the inline bootstrap script. */
  nonce?: string;
  /** Optional script id for hosts that audit static head output. */
  id?: string;
}

const DEFAULT_STORAGE_KEY = "theme";
const DEFAULT_ROOT_SELECTOR = ":root";
const DEFAULT_DARK_CLASS = "dark";

/**
 * Resolve a stored theme preference to the root state the bootstrap applies.
 */
export function resolveThemeBootstrapState(
  storedPreference: string | null | undefined,
  prefersDark: boolean,
  options: ThemeBootstrapOptions = {},
): ThemeBootstrapState {
  const resolved = resolveThemeBootstrapOptions(options);
  const stored = normalizePreference(storedPreference);
  const preference = stored ?? resolved.defaultPreference;
  return {
    preference,
    theme: preference === "system" ? (prefersDark ? "dark" : "light") : preference,
    source: stored ? "storage" : "fallback",
  };
}

/**
 * Apply the resolved theme state to the configured root element.
 *
 * Hosts can reuse this inside `applyThemeTransition({ apply })` so the initial
 * bootstrap and later toggles share one storage/root contract.
 */
export function applyThemeBootstrap(options: ThemeBootstrapOptions = {}): ThemeBootstrapState {
  const resolved = resolveThemeBootstrapOptions(options);
  const state = resolveThemeBootstrapState(
    readStoredPreference(resolved.storageKey),
    prefersDarkColorScheme(),
    resolved,
  );
  applyThemeBootstrapState(state, resolved);
  return state;
}

/**
 * Persist a preference and apply it with the same root contract as the bootstrap.
 */
export function setThemeBootstrapPreference(
  preference: ThemeBootstrapPreference,
  options: ThemeBootstrapOptions = {},
): ThemeBootstrapState {
  const resolved = resolveThemeBootstrapOptions(options);
  writeStoredPreference(resolved.storageKey, preference);
  const state = resolveThemeBootstrapState(preference, prefersDarkColorScheme(), resolved);
  applyThemeBootstrapState(state, resolved);
  return state;
}

/**
 * Return the exact inline JavaScript body a static host can hash for CSP.
 */
export function createThemeBootstrapScript(options: ThemeBootstrapOptions = {}): string {
  const config = serializeJsonForScript(resolveThemeBootstrapOptions(options));
  return `(()=>{const c=${config};const n=v=>v==="light"||v==="dark"||v==="system"?v:null;const g=()=>{if(!c.storageKey)return null;try{return localStorage.getItem(c.storageKey)}catch{return null}};const m=()=>{try{return matchMedia("(prefers-color-scheme: dark)").matches===true}catch{return false}};const r=()=>{try{return document.querySelector(c.rootSelector)||document.documentElement}catch{return document.documentElement}};const p=n(g())||c.defaultPreference;const t=p==="system"?(m()?"dark":"light"):p;const e=r();if(!e)return;if(c.themeAttribute)e.setAttribute(c.themeAttribute,t);if(c.darkClass)e.classList.toggle(c.darkClass,t==="dark");if(c.lightClass)e.classList.toggle(c.lightClass,t==="light")})();`;
}

/**
 * Render an inline first-paint bootstrap tag.
 *
 * Use `createThemeBootstrapScript()` when a deployment needs the exact body for
 * a CSP hash. Use `nonce` here when the host supplies per-request nonces.
 */
export function renderThemeBootstrapScript(
  options: ThemeBootstrapOptions = {},
  renderOptions: RenderThemeBootstrapScriptOptions = {},
): string {
  return `<script${renderScriptAttrs(renderOptions)}>${createThemeBootstrapScript(options)}</script>`;
}

export function resolveThemeBootstrapOptions(
  options: ThemeBootstrapOptions = {},
): ResolvedThemeBootstrapOptions {
  return {
    storageKey:
      options.storageKey === false
        ? false
        : typeof options.storageKey === "string" && options.storageKey.length > 0
          ? options.storageKey
          : DEFAULT_STORAGE_KEY,
    defaultPreference: normalizePreference(options.defaultPreference) ?? "system",
    rootSelector:
      typeof options.rootSelector === "string" && options.rootSelector.trim()
        ? options.rootSelector
        : DEFAULT_ROOT_SELECTOR,
    darkClass:
      options.darkClass === false
        ? false
        : typeof options.darkClass === "string" && options.darkClass.trim()
          ? options.darkClass
          : DEFAULT_DARK_CLASS,
    lightClass:
      options.lightClass === false || options.lightClass == null
        ? false
        : options.lightClass.trim() || false,
    themeAttribute:
      options.themeAttribute === false
        ? false
        : typeof options.themeAttribute === "string" && options.themeAttribute.trim()
          ? options.themeAttribute
          : "data-theme",
  };
}

function applyThemeBootstrapState(
  state: ThemeBootstrapState,
  options: ResolvedThemeBootstrapOptions,
): void {
  const root = resolveRoot(options.rootSelector);
  if (!root) {
    return;
  }
  if (options.themeAttribute) {
    root.setAttribute(options.themeAttribute, state.theme);
  }
  if (options.darkClass) {
    root.classList.toggle(options.darkClass, state.theme === "dark");
  }
  if (options.lightClass) {
    root.classList.toggle(options.lightClass, state.theme === "light");
  }
}

function readStoredPreference(storageKey: string | false): string | null {
  if (!storageKey) {
    return null;
  }
  try {
    return globalThis.localStorage?.getItem(storageKey) ?? null;
  } catch {
    return null;
  }
}

function writeStoredPreference(storageKey: string | false, preference: ThemeBootstrapPreference) {
  if (!storageKey) {
    return;
  }
  try {
    globalThis.localStorage?.setItem(storageKey, preference);
  } catch {
    // Storage can throw in private windows or locked-down embeds. The root state
    // still updates, which keeps the UI responsive and first-paint safe.
  }
}

function prefersDarkColorScheme(): boolean {
  try {
    return globalThis.matchMedia?.("(prefers-color-scheme: dark)").matches === true;
  } catch {
    return false;
  }
}

function resolveRoot(selector: string): Element | null {
  const doc = globalThis.document;
  if (!doc) {
    return null;
  }
  try {
    return doc.querySelector(selector) ?? doc.documentElement;
  } catch {
    return doc.documentElement;
  }
}

function normalizePreference(value: unknown): ThemeBootstrapPreference | null {
  return value === "light" || value === "dark" || value === "system" ? value : null;
}

function serializeJsonForScript(value: unknown): string {
  return JSON.stringify(value).replace(/[<>&\u2028\u2029]/g, (char) => {
    switch (char) {
      case "<":
        return "\\u003C";
      case ">":
        return "\\u003E";
      case "&":
        return "\\u0026";
      case "\u2028":
        return "\\u2028";
      case "\u2029":
        return "\\u2029";
      default:
        return char;
    }
  });
}

function renderScriptAttrs(options: RenderThemeBootstrapScriptOptions): string {
  const attrs: string[] = [];
  if (options.id) {
    attrs.push(`id="${escapeAttr(options.id)}"`);
  }
  if (options.nonce) {
    attrs.push(`nonce="${escapeAttr(options.nonce)}"`);
  }
  return attrs.length > 0 ? ` ${attrs.join(" ")}` : "";
}

function escapeAttr(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}
