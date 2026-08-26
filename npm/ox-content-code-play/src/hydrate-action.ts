import { decodePayload } from "./payload";
import { errorMessage } from "./result";
import type { PlayPayload, RunAction, RunActionState, RunResult } from "./types";

export function readPlayPayload(encoded: string): PlayPayload | undefined {
  try {
    return decodePayload(encoded);
  } catch {
    return undefined;
  }
}

export async function runPlayAction(input: {
  action: () => Promise<RunResult>;
  setBusy: (busy: boolean) => void;
  onResult: (result: RunResult) => void;
  onError: (error: unknown) => void;
}): Promise<void> {
  input.setBusy(true);
  try {
    input.onResult(await input.action());
  } catch (error) {
    input.onError(error);
  } finally {
    input.setBusy(false);
  }
}

export function idleRunActionState(): RunActionState {
  return { phase: "idle" };
}

export function runningRunActionState(action: RunAction, startedAtMs = Date.now()): RunActionState {
  return { phase: "running", action, startedAtMs };
}

export function resultRunActionState(
  action: RunAction,
  result: RunResult,
  finishedAtMs = Date.now(),
): RunActionState {
  const phase =
    result.status === "offline"
      ? "offline"
      : result.status === "ok" || result.status === "cancelled"
        ? "result"
        : "error";
  return {
    phase,
    action,
    result,
    message: result.diagnostics[0]?.message,
    finishedAtMs,
  };
}

export function errorRunActionState(
  action: RunAction,
  error: unknown,
  finishedAtMs = Date.now(),
): RunActionState {
  return {
    phase: "error",
    action,
    message: errorMessage(error),
    finishedAtMs,
  };
}
