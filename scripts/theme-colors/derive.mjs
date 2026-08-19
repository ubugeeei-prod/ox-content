// Colour derivation for the @ox-content/theme-color-* generator.
//
// Split out of generate.mjs so the emitter stays a file writer and the parts
// with actual reasoning — contrast normalisation, token expansion — sit
// together and can be read on their own.

const mix = (color, amount, into = "transparent") =>
  `color-mix(in srgb, ${color} ${amount}%, ${into})`;

function rgbTriplet(hex) {
  const value = hex.replace("#", "").slice(0, 6);
  const int = Number.parseInt(value, 16);
  return `${(int >> 16) & 255}, ${(int >> 8) & 255}, ${int & 255}`;
}

/** Relative luminance, used only to decide light-on-dark vs dark-on-light. */
function luminance(hex) {
  const [r, g, b] = rgbTriplet(hex)
    .split(", ")
    .map((n) => {
      const v = Number(n) / 255;
      return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
    });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

const toHex = (n) =>
  Math.round(Math.max(0, Math.min(255, n)))
    .toString(16)
    .padStart(2, "0");

function contrast(a, b) {
  const [x, y] = [luminance(a), luminance(b)].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
}

/**
 * Nudges an accent toward black or white until it clears `target` against the
 * background it will be read on.
 *
 * Schemes name their accent at the lightness the original design used it — Ayu's
 * orange, Snow's sky blue — which is fine for a fill but lands around 2:1 as link
 * text on a light page. Hue is preserved; only lightness moves, and only as far
 * as it has to.
 */
function ensureContrast(color, bg, target) {
  if (contrast(color, bg) >= target) return color;
  const toward = luminance(bg) > 0.4 ? 0 : 255;
  const base = rgbTriplet(color).split(", ").map(Number);
  for (let t = 0.04; t <= 1; t += 0.04) {
    const mixed = "#" + base.map((c) => toHex(c + (toward - c) * t)).join("");
    if (contrast(mixed, bg) >= target) return mixed;
  }
  return toward === 0 ? "#000000" : "#ffffff";
}

function backdrop(hex) {
  const [r, g, b] = rgbTriplet(hex)
    .split(", ")
    .map((n) => Math.round(Number(n) * 0.35));
  return `rgba(${r}, ${g}, ${b}, 0.62)`;
}

/**
 * Expands the 15 authored colors of one mode into the full token surface.
 *
 * `other` is the opposite mode's colors, needed because a scheme's code block
 * does not always match its page: several light schemes keep a dark code
 * surface. Syntax colors are picked from whichever mode was tuned for the code
 * background's own lightness, otherwise light-mode accents land on a dark code
 * block and the highlighting is unreadable.
 */
function tokensFor(c, other, mode) {
  // Pick the accent set that is actually readable on this scheme's code
  // background, by measuring it. A luminance bucket is too crude: Commander's
  // page is CGA cyan and its code panel is CGA blue, and a threshold puts both
  // on the same side while the two need opposite accents.
  const legibility = (set) =>
    ["red", "green", "yellow", "blue", "magenta", "cyan"].reduce(
      (sum, key) => sum + contrast(set[key], c.codeBg),
      0,
    );
  const code = legibility(c) >= legibility(other) ? c : other;
  return {
    "color-code-line-highlight": mix(c.blue, 16),
    "color-code-line-warning": mix(c.yellow, 18),
    "color-code-line-warning-border": c.yellow,
    "color-code-line-error": mix(c.red, 20),
    "color-code-line-error-border": c.red,
    "color-code-line-add": mix(c.green, 16),
    "color-code-line-add-border": c.green,
    "color-code-line-remove": mix(c.red, 14),
    "color-code-line-remove-border": c.red,
    "color-code-line-focus": mix(c.primary, 16),
    "color-code-line-dim": mode === "dark" ? "0.35" : "0.45",
    "color-code-title-bg": mix(c.codeBg, 92, c.codeText),
    "color-code-title-text": mix(c.codeText, 88, c.codeBg),
    "color-code-title-border": mix(c.codeText, 22),
    "color-code-line-number": mix(c.codeText, 48),
    "color-code-frame-border": mix(c.border, 92),
    "color-backdrop": backdrop(c.bg),
    "surface-glass": mix(c.bgAlt, 62, c.bg),
    "surface-line": mix(c.border, 72, c.bg),
    "brand-violet": c.magenta,
    "brand-cyan": c.cyan,
    "brand-lime": c.green,
    "brand-coral": c.red,
    "brand-navy": mix(c.text, 70, c.bg),
    "brand-violet-rgb": rgbTriplet(c.magenta),
    "brand-cyan-rgb": rgbTriplet(c.cyan),
    "brand-lime-rgb": rgbTriplet(c.green),
    "brand-coral-rgb": rgbTriplet(c.red),
    // Semantic aliases skins read for accents they must not hard-code.
    "accent-a": c.primary,
    "accent-b": c.magenta,
    "accent-c": c.cyan,
    "accent-warm": c.yellow,
    "accent-cool": c.blue,
    // Ink variants: the same hues tightened to AA against the page. Skins use
    // these wherever an accent carries text — either as the text colour itself,
    // or as a fill that page-coloured text sits on — while the plain accents
    // stay at their authored vividness for decoration.
    "accent-a-ink": ensureContrast(c.primary, c.bg, 4.5),
    "accent-b-ink": ensureContrast(c.magenta, c.bg, 4.5),
    "accent-c-ink": ensureContrast(c.cyan, c.bg, 4.5),
    "accent-warm-ink": ensureContrast(c.yellow, c.bg, 4.5),
    "accent-cool-ink": ensureContrast(c.blue, c.bg, 4.5),
    "accent-coral-ink": ensureContrast(c.red, c.bg, 4.5),
    // Shiki renders through `createCssVariablesTheme`, so syntax colors are
    // just custom properties and follow the scheme with no extra config.
    "shiki-foreground": c.codeText,
    "shiki-background": c.codeBg,
    "shiki-token-comment": mix(c.codeText, 55, c.codeBg),
    "shiki-token-punctuation": mix(c.codeText, 72, c.codeBg),
    "shiki-token-keyword": code.magenta,
    "shiki-token-string": code.green,
    "shiki-token-string-expression": code.green,
    "shiki-token-constant": code.yellow,
    "shiki-token-function": code.blue,
    "shiki-token-parameter": code.red,
    "shiki-token-link": code.cyan,
  };
}

export { mix, rgbTriplet, luminance, contrast, ensureContrast, tokensFor };
