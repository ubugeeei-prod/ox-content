export function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, (char) => {
    switch (char) {
      case "&":
        return "&amp;";
      case "<":
        return "&lt;";
      case ">":
        return "&gt;";
      case '"':
        return "&quot;";
      default:
        return "&#39;";
    }
  });
}

export function decodeHtml(value: string): string {
  return value
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&#x([0-9a-fA-F]+);/g, (_, hex: string) => decodeCodePoint(hex, 16))
    .replace(/&#(\d+);/g, (_, dec: string) => decodeCodePoint(dec, 10))
    .replace(/&amp;/g, "&");
}

function decodeCodePoint(digits: string, radix: number): string {
  const code = Number.parseInt(digits, radix);
  return Number.isFinite(code) ? String.fromCodePoint(code) : "";
}

export function escapeAttribute(value: string): string {
  return escapeHtml(value).replace(/`/g, "&#96;");
}
