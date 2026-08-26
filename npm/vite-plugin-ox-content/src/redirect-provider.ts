import type { RedirectProvider } from "./types";

const CONFLICT_WARNING =
  '[ox-content] Conflicting redirect provider environment variables (Cloudflare and Netlify). Set redirects.provider explicitly to "netlify" or "cloudflare". The host _redirects file will not be written.';

/**
 * Resolves the `_redirects` host from an explicit option or CI env.
 *
 * Pass `env` so tests can inject values without mutating `process.env`.
 */
export function resolveRedirectProvider(
  explicit: RedirectProvider | undefined,
  env: NodeJS.ProcessEnv,
): { provider?: RedirectProvider; warning?: string } {
  if (explicit) {
    return { provider: explicit };
  }
  const cloudflare = env.CF_PAGES === "1" || env.WORKERS_CI === "1";
  const netlify = env.NETLIFY === "true";
  if (cloudflare && netlify) {
    return { warning: CONFLICT_WARNING };
  }
  if (cloudflare) {
    return { provider: "cloudflare" };
  }
  if (netlify) {
    return { provider: "netlify" };
  }
  return {};
}
