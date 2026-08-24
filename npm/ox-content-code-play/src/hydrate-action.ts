import { decodePayload } from "./payload";
import type { PlayPayload, RunResult } from "./types";

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
