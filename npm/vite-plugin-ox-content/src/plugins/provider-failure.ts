/**
 * Why a build-time embed lookup did not produce metadata.
 *
 * Every fetcher used to collapse these into one message carrying a status code
 * or an error string, so a rate limit read the same as a deleted resource and a
 * disconnected machine read the same as a typo. The card that ships is the same
 * either way; the difference matters to whoever reads the build log.
 */

export type EmbedFailure =
  /** The provider answered and refused: auth, or a rate limit. */
  | { kind: "unavailable"; status: number }
  /** The provider answered and the thing is not there. */
  | { kind: "missing"; status: number }
  /** The request was made and failed: timeout, 5xx, a broken response. */
  | { kind: "fetch-failed"; reason: string }
  /** No network was reachable, so nothing was really attempted. */
  | { kind: "offline"; reason: string };

/** Node's fetch reports a missing network through `cause.code`. */
const OFFLINE_CODES = new Set([
  "ENOTFOUND",
  "EAI_AGAIN",
  "ECONNREFUSED",
  "ENETUNREACH",
  "ENETDOWN",
  "EHOSTUNREACH",
]);

export function classifyStatus(status: number): EmbedFailure {
  if (status === 404 || status === 410) return { kind: "missing", status };
  if (status === 401 || status === 403 || status === 429) return { kind: "unavailable", status };
  return { kind: "fetch-failed", reason: `HTTP ${status}` };
}

export function classifyError(error: unknown): EmbedFailure {
  if (error instanceof Error) {
    if (error.name === "AbortError") return { kind: "fetch-failed", reason: "timed out" };
    const code = errorCode(error);
    if (code && OFFLINE_CODES.has(code)) return { kind: "offline", reason: code };
    return { kind: "fetch-failed", reason: error.message };
  }
  return { kind: "fetch-failed", reason: "unknown error" };
}

/** `cause` is where undici puts the syscall error; older runtimes use `code`. */
function errorCode(error: Error): string | undefined {
  const cause = (error as { cause?: unknown }).cause;
  if (cause && typeof cause === "object" && "code" in cause) {
    const code = (cause as { code?: unknown }).code;
    if (typeof code === "string") return code;
  }
  const own = (error as { code?: unknown }).code;
  return typeof own === "string" ? own : undefined;
}

export function describeFailure(failure: EmbedFailure): string {
  switch (failure.kind) {
    case "missing":
      return `the resource does not exist (${failure.status})`;
    case "unavailable":
      return failure.status === 429
        ? "the provider is rate limiting this build (429)"
        : `the provider refused the request (${failure.status})`;
    case "offline":
      return `no network (${failure.reason})`;
    case "fetch-failed":
      return failure.reason;
  }
}

/**
 * An offline build fails every lookup for the same reason, so it says so once.
 * One line per embed would bury the single fact that matters.
 */
let offlineReported = false;

export function resetEmbedFailureReporting(): void {
  offlineReported = false;
}

export function warnEmbedFailure(
  provider: string,
  target: string,
  failure: EmbedFailure,
  fallback: string,
): void {
  if (failure.kind === "offline") {
    if (offlineReported) return;
    offlineReported = true;
    console.warn(
      `[ox-content:embeds] No network (${failure.reason}); every embed lookup this build renders ${fallback}.`,
    );
    return;
  }
  console.warn(
    `[ox-content:embeds] ${provider} lookup for ${target} failed: ${describeFailure(failure)}; rendering ${fallback}.`,
  );
}
