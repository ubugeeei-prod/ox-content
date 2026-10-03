import stringWidth from "string-width";
import wrapAnsi from "wrap-ansi";
import { stripVTControlCharacters } from "node:util";

export const plain = stripVTControlCharacters;
export const clean = (text) => String(text).replace(/[\x00-\x08\x0b-\x1f\x7f-\x9f]/g, "");
const themes = {
  nord: {
    accent: "136;192;208",
    muted: "129;161;193",
    code: "163;190;140",
    warning: "235;203;139",
  },
  light: { accent: "0;96;128", muted: "96;104;120", code: "30;112;54", warning: "144;80;0" },
  mono: {
    accent: "255;255;255",
    muted: "160;160;160",
    code: "220;220;220",
    warning: "190;190;190",
  },
};
export const themeNames = Object.keys(themes);
export function styles(theme = "nord", color = true) {
  const palette = themes[theme];
  if (!palette) throw new Error(`Unknown theme: ${theme}`);
  const paint = (code) => (text) => (color ? `\x1b[${code}m${text}\x1b[0m` : String(text));
  return {
    accent: paint(`38;2;${palette.accent}`),
    muted: paint(`38;2;${palette.muted}`),
    code: paint(`38;2;${palette.code}`),
    warning: paint(`38;2;${palette.warning}`),
    bold: paint("1"),
    italic: paint("3"),
    dim: paint("2"),
    selected: paint("7"),
  };
}
export const width = stringWidth;
export const wrap = (text, columns) =>
  wrapAnsi(text, Math.max(1, columns), { hard: true, trim: false }).split("\n");
const segmenter = new Intl.Segmenter(undefined, { granularity: "grapheme" });
export function clip(text, columns) {
  if (width(text) <= columns) return text;
  // Chrome-style cursor sequences never come from content: preserve only our SGR colors.
  let result = "",
    used = 0;
  for (const part of String(text).split(/(\x1b\[[\d;]*m)/)) {
    if (/^\x1b\[/.test(part)) {
      result += part;
      continue;
    }
    for (const { segment } of segmenter.segment(part)) {
      const size = width(segment);
      if (used + size > columns) return result + "\x1b[0m";
      used += size;
      result += segment;
    }
  }
  return result;
}
export function pad(text, columns) {
  const clipped = clip(text, Math.max(0, columns));
  return clipped + " ".repeat(Math.max(0, columns - width(clipped)));
}
