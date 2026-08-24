import { withStdioText } from "./stdio";
import { emptyTiming } from "./timing";
import type { RunResult, RunStatus } from "./types";

export function errorMessage(error: unknown): string {
  return error instanceof Error && error.message ? error.message : String(error);
}

export function errorResult(
  message: string,
  source = "code-play",
  status: RunStatus = "error",
): RunResult {
  return withStdioText({
    status,
    stdio: [],
    diagnostics: [{ message, severity: "error", source }],
    provenance: {},
    timing: emptyTiming(),
  });
}
