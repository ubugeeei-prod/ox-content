import { executeInSandboxIframe } from "./javascript-sandbox";
import { hasNodeVm } from "./runtime-host";
import { formatConsoleArgs, StdioBuffer } from "./stdio";
import { PhaseTracker } from "./timing";
import { abortError, isAbortError } from "./transport";
import type { AdapterRequest, AdapterResult, Diagnostic } from "./types";

export async function runJavaScript(request: AdapterRequest): Promise<AdapterResult> {
  const tracker = new PhaseTracker();
  tracker.start("execute", "Execute");
  const stdio = new StdioBuffer(tracker.startedAt);
  const provenance = {
    execute: {
      host: "local",
      runtime: hasNodeVm() ? "node:vm" : "iframe",
      sandbox: hasNodeVm() ? "vm" : "srcdoc",
    },
  };

  try {
    const value = await executeScript(request.code, request.timeoutMs, stdio, request.signal);
    tracker.stop();
    return {
      status: "ok",
      stdio: stdio.snapshot(),
      diagnostics: [],
      provenance,
      timing: tracker.report(),
      value: value === undefined ? undefined : String(value),
    };
  } catch (error) {
    if (isAbortError(error) || request.signal?.aborted) {
      throw error;
    }
    tracker.stop();
    const diagnostic = toDiagnostic(error);
    stdio.push("stderr", `${diagnostic.message}\n`);
    return {
      status: isTimeout(error) ? "timeout" : "error",
      stdio: stdio.snapshot(),
      diagnostics: [diagnostic],
      provenance,
      timing: tracker.report(),
    };
  }
}

export async function executeScript(
  code: string,
  timeoutMs: number,
  stdio: StdioBuffer,
  signal?: AbortSignal,
): Promise<unknown> {
  const consoleLike = {
    log: (...args: unknown[]) => stdio.push("stdout", formatConsoleArgs(args)),
    info: (...args: unknown[]) => stdio.push("stdout", formatConsoleArgs(args)),
    warn: (...args: unknown[]) => stdio.push("stderr", formatConsoleArgs(args)),
    error: (...args: unknown[]) => stdio.push("stderr", formatConsoleArgs(args)),
  };

  if (signal?.aborted) {
    throw abortError();
  }

  const runtime = javascriptExecuteRuntime(hasNodeVm(), typeof document !== "undefined");
  if (runtime === "vm") {
    const vm = await import("node:vm");
    const context = vm.createContext({ console: consoleLike });
    return vm.runInContext(code, context, { timeout: timeoutMs, displayErrors: true });
  }

  return executeInSandboxIframe(code, timeoutMs, stdio, signal);
}

export function javascriptExecuteRuntime(hasVm: boolean, hasDocument: boolean): "vm" | "iframe" {
  if (hasVm) {
    return "vm";
  }
  if (hasDocument) {
    return "iframe";
  }
  throw new Error("JavaScript execute needs node:vm or a document for the sandbox iframe.");
}

function isTimeout(error: unknown): boolean {
  return Boolean(
    error &&
    typeof error === "object" &&
    "code" in error &&
    error.code === "ERR_SCRIPT_EXECUTION_TIMEOUT",
  );
}

function toDiagnostic(error: unknown): Diagnostic {
  if (isErrorLike(error)) {
    return { message: error.message, severity: "error", source: "javascript" };
  }
  return { message: String(error), severity: "error", source: "javascript" };
}

function isErrorLike(error: unknown): error is { message: string } {
  return Boolean(
    error &&
    typeof error === "object" &&
    "message" in error &&
    typeof error.message === "string" &&
    error.message.length > 0,
  );
}
