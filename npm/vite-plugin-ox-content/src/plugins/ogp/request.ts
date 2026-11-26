/**
 * Open Graph page request: public hosts only, redirects checked hop by hop,
 * one deadline for the whole exchange, and a bounded body.
 */

import { lookup as dnsLookup } from "node:dns/promises";
import { isBlockedFeedAddress } from "../../blog-feed-url";
import { isSafeOgpUrl } from "./url";

/** Bytes read from a page. Open Graph tags live in `<head>`, so the rest is dropped. */
export const OGP_MAX_BYTES = 1_048_576;
export const OGP_MAX_REDIRECTS = 5;

export type OgpLookup = (hostname: string) => Promise<string[]>;

let installedLookup: OgpLookup | undefined;

/** Test hook. Production builds use the system resolver. */
export function installOgpLookup(lookup: OgpLookup | undefined): void {
  installedLookup = lookup;
}

export type OgpPageResult =
  | { kind: "ok"; html: string }
  | { kind: "status"; status: number }
  | { kind: "blocked" };

export async function requestOgpPage(
  url: string,
  init: { timeout: number; headers: Record<string, string> },
): Promise<OgpPageResult> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), init.timeout);
  try {
    let current = url;
    for (let hop = 0; hop <= OGP_MAX_REDIRECTS; hop += 1) {
      if (!(await isPublicTarget(current))) {
        return { kind: "blocked" };
      }
      const response = await fetch(current, {
        headers: init.headers,
        redirect: "manual",
        signal: controller.signal,
      });
      if (isRedirect(response.status)) {
        const location = response.headers?.get("location");
        await response.body?.cancel().catch(() => undefined);
        if (!location) {
          return { kind: "status", status: response.status };
        }
        current = new URL(location, current).href;
        continue;
      }
      if (!response.ok) {
        return { kind: "status", status: response.status };
      }
      return { kind: "ok", html: await readHead(response, OGP_MAX_BYTES) };
    }
    return { kind: "blocked" };
  } finally {
    clearTimeout(timer);
  }
}

async function isPublicTarget(url: string): Promise<boolean> {
  if (!isSafeOgpUrl(url)) {
    return false;
  }
  const hostname = new URL(url).hostname.replace(/^\[|\]$/g, "");
  let addresses: string[];
  try {
    addresses = await (installedLookup ?? systemLookup)(hostname);
  } catch {
    // An unresolvable name cannot be connected to; let fetch report it.
    return true;
  }
  return !addresses.some((address) => isBlockedFeedAddress(address));
}

async function systemLookup(hostname: string): Promise<string[]> {
  const records = await dnsLookup(hostname, { all: true, verbatim: true });
  return records.map((record) => record.address);
}

function isRedirect(status: number): boolean {
  return status === 301 || status === 302 || status === 303 || status === 307 || status === 308;
}

async function readHead(response: Response, maxBytes: number): Promise<string> {
  const reader = response.body?.getReader();
  if (!reader) {
    return (await response.text()).slice(0, maxBytes);
  }
  const chunks: Uint8Array[] = [];
  let total = 0;
  while (total < maxBytes) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    const chunk = value.subarray(0, maxBytes - total);
    chunks.push(chunk);
    total += chunk.byteLength;
  }
  await reader.cancel().catch(() => undefined);
  const bytes = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return new TextDecoder("utf-8").decode(bytes);
}
