/**
 * Deterministic blog reading-time estimates.
 */

const LATIN_WORDS_PER_MINUTE = 200;
const CJK_CHARS_PER_MINUTE = 500;

export function readingTimeMinutes(markdown: string): number {
  const body = stripInlineCode(stripFences(stripFrontmatter(markdown)));
  let latin = 0;
  let cjk = 0;
  let latinRun = false;
  for (const char of body) {
    const code = char.codePointAt(0) ?? 0;
    if (isCjkCodePoint(code)) {
      cjk += 1;
      latinRun = false;
      continue;
    }
    if (isLatinWordChar(code)) {
      if (!latinRun) {
        latin += 1;
        latinRun = true;
      }
      continue;
    }
    if (char === "'" || char === "\u2019") {
      continue;
    }
    latinRun = false;
  }
  if (latin === 0 && cjk === 0) {
    return 0;
  }
  return Math.max(1, Math.ceil(latin / LATIN_WORDS_PER_MINUTE + cjk / CJK_CHARS_PER_MINUTE));
}

function stripFrontmatter(markdown: string): string {
  if (!markdown.startsWith("---")) {
    return markdown;
  }
  const match = markdown.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/);
  return match ? markdown.slice(match[0].length) : markdown;
}

function stripFences(text: string): string {
  return text.replace(/```[\s\S]*?(?:```|$)/g, " ");
}

function stripInlineCode(text: string): string {
  return text.replace(/`[^`\n]*`/g, " ");
}

function isCjkCodePoint(code: number): boolean {
  return (
    (code >= 0x3040 && code <= 0x30ff) ||
    (code >= 0x31f0 && code <= 0x31ff) ||
    (code >= 0x3400 && code <= 0x4dbf) ||
    (code >= 0x4e00 && code <= 0x9fff) ||
    (code >= 0xf900 && code <= 0xfaff) ||
    (code >= 0xac00 && code <= 0xd7af) ||
    (code >= 0x1100 && code <= 0x11ff)
  );
}

function isLatinWordChar(code: number): boolean {
  return (
    (code >= 0x30 && code <= 0x39) ||
    (code >= 0x41 && code <= 0x5a) ||
    (code >= 0x61 && code <= 0x7a)
  );
}
