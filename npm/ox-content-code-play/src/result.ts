import { withStdioText } from "./stdio";
import { emptyTiming } from "./timing";
import type { RunResult, RunStatus } from "./types";

export function errorMessage(error: unknown): string {
  return error instanceof Error && error.message ? error.message : String(error);
}

export function friendlyTransportMessage(error: unknown): string {
  const message = errorMessage(error);
  if (isOfflineError(error)) {
    return "The executor is offline or unreachable from this page (for example, CORS). Set endpoints to a host that allows browser POST, or use the Vite dev proxy.";
  }
  return message;
}

export function transportFailureStatus(error: unknown): RunStatus {
  return isOfflineError(error) ? "offline" : "error";
}

export function errorResult(
  message: string,
  source = "code-play",
  status: RunStatus = "error",
): RunResult {
  return withStdioText({
    status,
    stdio: [],
    diagnostics: [
      {
        message,
        severity: status === "cancelled" ? "info" : "error",
        source,
      },
    ],
    provenance: {},
    timing: emptyTiming(),
  });
}

function isOfflineError(error: unknown): boolean {
  const message = errorMessage(error);
  const name =
    error && typeof error === "object" && "name" in error ? String(error.name) : undefined;
  const typeError = error instanceof TypeError || name === "TypeError";
  if (name === "MissingTransportError") {
    return true;
  }
  if (
    typeError &&
    /failed to fetch|networkerror|load failed|network request failed/i.test(message)
  ) {
    return true;
  }
  return /\boffline\b|no code play transport|network request failed/i.test(message);
}
