import { withStdioText } from "./stdio";
import { emptyTiming } from "./timing";
import type { RunResult, RunStatus } from "./types";

export function errorMessage(error: unknown): string {
  return error instanceof Error && error.message ? error.message : String(error);
}

export function friendlyTransportMessage(error: unknown): string {
  const message = errorMessage(error);
  const typeError =
    error instanceof TypeError || (error instanceof Error && error.name === "TypeError");
  if (
    typeError &&
    /failed to fetch|networkerror|load failed|network request failed/i.test(message)
  ) {
    return "The executor could not be reached from this page (often CORS). Set endpoints to a host that allows browser POST, or use the Vite dev proxy.";
  }
  return message;
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
