/**
 * Safe-URL checks for configured external blog feeds.
 */

const CONTROL_CHARS = /[\n\r\t\0]/;

export function isSafeFeedUrl(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed || CONTROL_CHARS.test(trimmed)) {
    return false;
  }
  try {
    const url = new URL(trimmed);
    if (url.protocol !== "https:") {
      return false;
    }
    if (url.username || url.password) {
      return false;
    }
    return !isBlockedFeedHost(url.hostname);
  } catch {
    return false;
  }
}

export function canonicalizeFeedItemUrl(value: string): string | undefined {
  if (!isSafeFeedUrl(value)) {
    return undefined;
  }
  const url = new URL(value.trim());
  url.hash = "";
  url.username = "";
  url.password = "";
  if (url.port === "443") {
    url.port = "";
  }
  url.hostname = url.hostname.toLowerCase();
  let href = url.href;
  if (url.pathname !== "/" && href.endsWith("/")) {
    href = href.slice(0, -1);
  }
  return href;
}

export function isBlockedFeedHost(hostname: string): boolean {
  const host = hostname.toLowerCase().replace(/^\[|\]$/g, "");
  if (!host || host === "localhost" || host.endsWith(".localhost") || host.endsWith(".local")) {
    return true;
  }
  if (host.includes(":")) {
    return isBlockedIPv6(host);
  }
  return isIPv4(host) ? isBlockedIPv4(host) : false;
}

export function isBlockedFeedAddress(address: string): boolean {
  const value = address.toLowerCase().replace(/^\[|\]$/g, "");
  if (value.includes(":")) {
    return isBlockedIPv6(value);
  }
  return isIPv4(value) ? isBlockedIPv4(value) : true;
}

function isIPv4(value: string): boolean {
  const parts = value.split(".");
  if (parts.length !== 4) {
    return false;
  }
  return parts.every((part) => {
    const n = Number(part);
    return Number.isInteger(n) && n >= 0 && n <= 255 && String(n) === part;
  });
}

function isBlockedIPv4(ip: string): boolean {
  const parts = ip.split(".").map(Number);
  const [a, b] = parts;
  return (
    a === 0 ||
    a === 10 ||
    a === 127 ||
    (a === 169 && b === 254) ||
    (a === 172 && b >= 16 && b <= 31) ||
    (a === 192 && b === 168)
  );
}

function isBlockedIPv6(ip: string): boolean {
  if (ip === "::" || ip === "::1") {
    return true;
  }
  if (ip.startsWith("::ffff:")) {
    const mapped = ip.slice("::ffff:".length);
    return isIPv4(mapped) ? isBlockedIPv4(mapped) : true;
  }
  const first = Number.parseInt(ip.split(":")[0] ?? "", 16);
  if (!Number.isFinite(first)) {
    return true;
  }
  if (first >= 0xfe80 && first <= 0xfebf) {
    return true;
  }
  return (first & 0xfe00) === 0xfc00;
}
