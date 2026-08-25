import {
  canUseSandboxWorker,
  executeInSandboxIframe,
  executeInSandboxWorker,
  isSandboxWorkerUnavailable,
} from "./javascript-sandbox";
import { hasNodeVm } from "./runtime-host";
import { formatConsoleArgs, StdioBuffer } from "./stdio";
import { PhaseTracker } from "./timing";
import { abortError, isAbortError } from "./transport";
import type { AdapterRequest, AdapterResult, Diagnostic, RuntimeLocation } from "./types";

export type JavaScriptExecutionRuntime = "vm" | "worker" | "iframe";

export interface JavaScriptExecutionResult {
  value: unknown;
  runtime: JavaScriptExecutionRuntime;
}

export async function runJavaScript(request: AdapterRequest): Promise<AdapterResult> {
  const tracker = new PhaseTracker();
  tracker.start("execute", "Execute");
  const stdio = new StdioBuffer(tracker.startedAt);
  const runtime = currentJavaScriptRuntime();
  let executedRuntime = runtime;

  try {
    const result = await executeScriptWithRuntime(
      request.code,
      request.timeoutMs,
      stdio,
      request.signal,
      runtime,
    );
    executedRuntime = result.runtime;
    tracker.stop();
    return {
      status: "ok",
      stdio: stdio.snapshot(),
      diagnostics: [],
      provenance: {
        execute: javascriptRuntimeProvenance(executedRuntime),
      },
      timing: tracker.report(),
      value: result.value === undefined ? undefined : String(result.value),
    };
  } catch (error) {
    if (isAbortError(error) || request.signal?.aborted) {
      throw error;
    }
    executedRuntime = executionRuntimeFromError(error) ?? executedRuntime;
    tracker.stop();
    const diagnostic = toDiagnostic(error);
    stdio.push("stderr", `${diagnostic.message}\n`);
    return {
      status: isTimeout(error) ? "timeout" : "error",
      stdio: stdio.snapshot(),
      diagnostics: [diagnostic],
      provenance: {
        execute: javascriptRuntimeProvenance(executedRuntime),
      },
      timing: tracker.report(),
    };
  }
}

export async function executeScript(
  code: string,
  timeoutMs: number,
  stdio: StdioBuffer,
  signal?: AbortSignal,
  runtime = currentJavaScriptRuntime(),
): Promise<unknown> {
  const result = await executeScriptWithRuntime(code, timeoutMs, stdio, signal, runtime);
  return result.value;
}

export async function executeScriptWithRuntime(
  code: string,
  timeoutMs: number,
  stdio: StdioBuffer,
  signal?: AbortSignal,
  runtime = currentJavaScriptRuntime(),
): Promise<JavaScriptExecutionResult> {
  try {
    return {
      value: await executeScriptInRuntime(code, timeoutMs, stdio, signal, runtime),
      runtime,
    };
  } catch (error) {
    if (
      runtime === "worker" &&
      isSandboxWorkerUnavailable(error) &&
      typeof document !== "undefined"
    ) {
      try {
        return {
          value: await executeScriptInRuntime(code, timeoutMs, stdio, signal, "iframe"),
          runtime: "iframe",
        };
      } catch (fallbackError) {
        throw withExecutionRuntime(fallbackError, "iframe");
      }
    }
    throw withExecutionRuntime(error, runtime);
  }
}

async function executeScriptInRuntime(
  code: string,
  timeoutMs: number,
  stdio: StdioBuffer,
  signal: AbortSignal | undefined,
  runtime: JavaScriptExecutionRuntime,
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

  if (runtime === "vm") {
    const vm = await import("node:vm");
    const context = vm.createContext({ console: consoleLike });
    return vm.runInContext(code, context, { timeout: timeoutMs, displayErrors: true });
  }
  if (runtime === "worker") {
    return executeInSandboxWorker(code, timeoutMs, stdio, signal);
  }

  return executeInSandboxIframe(code, timeoutMs, stdio, signal);
}

export function currentJavaScriptRuntime(): JavaScriptExecutionRuntime {
  return javascriptExecuteRuntime(
    hasNodeVm(),
    canUseSandboxWorker(),
    typeof document !== "undefined",
  );
}

export function javascriptExecuteRuntime(
  hasVm: boolean,
  hasWorker: boolean,
  hasDocument: boolean,
): JavaScriptExecutionRuntime {
  if (hasVm) {
    return "vm";
  }
  if (hasWorker) {
    return "worker";
  }
  if (hasDocument) {
    return "iframe";
  }
  throw new Error(
    "JavaScript execute needs node:vm, a browser worker sandbox, or a document for the sandbox iframe.",
  );
}

export function javascriptRuntimeProvenance(runtime: JavaScriptExecutionRuntime): RuntimeLocation {
  if (runtime === "vm") {
    return { host: "local", runtime: "node:vm", sandbox: "vm" };
  }
  if (runtime === "worker") {
    return { host: "local", runtime: "web-worker", sandbox: "worker" };
  }
  return { host: "local", runtime: "iframe", sandbox: "srcdoc" };
}

function withExecutionRuntime(
  error: unknown,
  runtime: JavaScriptExecutionRuntime,
): Error & { executionRuntime: JavaScriptExecutionRuntime } {
  if (error instanceof Error) {
    return Object.assign(error, { executionRuntime: runtime });
  }
  if (isErrorLike(error)) {
    return Object.assign(error, { executionRuntime: runtime }) as Error & {
      executionRuntime: JavaScriptExecutionRuntime;
    };
  }
  return Object.assign(new Error(String(error)), { executionRuntime: runtime });
}

export function executionRuntimeFromError(error: unknown): JavaScriptExecutionRuntime | undefined {
  if (
    error &&
    typeof error === "object" &&
    "executionRuntime" in error &&
    isJavaScriptExecutionRuntime(error.executionRuntime)
  ) {
    return error.executionRuntime;
  }
  return undefined;
}

function isJavaScriptExecutionRuntime(value: unknown): value is JavaScriptExecutionRuntime {
  return value === "vm" || value === "worker" || value === "iframe";
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
