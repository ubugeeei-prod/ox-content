import { formatConsoleArgs, StdioBuffer } from "./stdio";
import { nowMs, PhaseTracker } from "./timing";
import type { AdapterRequest, Diagnostic, RunResult } from "./types";

export async function runJavaScript(request: AdapterRequest): Promise<RunResult> {
  const tracker = new PhaseTracker();
  tracker.start("execute", "Execute");
  const stdio = new StdioBuffer(tracker.startedAt);
  const provenance = {
    execute: {
      host: "local",
      runtime: hasNodeVm() ? "node:vm" : "function",
      sandbox: hasNodeVm() ? "vm" : "function",
    },
  };

  try {
    const value = await executeScript(request.code, request.timeoutMs, stdio);
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
): Promise<unknown> {
  const consoleLike = {
    log: (...args: unknown[]) => stdio.push("stdout", formatConsoleArgs(args)),
    info: (...args: unknown[]) => stdio.push("stdout", formatConsoleArgs(args)),
    warn: (...args: unknown[]) => stdio.push("stderr", formatConsoleArgs(args)),
    error: (...args: unknown[]) => stdio.push("stderr", formatConsoleArgs(args)),
  };

  if (hasNodeVm()) {
    const vm = await import("node:vm");
    const context = vm.createContext({ console: consoleLike });
    return vm.runInContext(code, context, { timeout: timeoutMs, displayErrors: true });
  }

  const started = nowMs();
  const run = new Function("console", `"use strict";\n${code}`);
  const value = run(consoleLike);
  if (nowMs() - started > timeoutMs) {
    throw Object.assign(new Error("JavaScript execution timed out."), {
      code: "ERR_SCRIPT_EXECUTION_TIMEOUT",
    });
  }
  return value;
}

function hasNodeVm(): boolean {
  return typeof process !== "undefined" && Boolean(process.versions?.node);
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
  if (error instanceof Error) {
    return { message: error.message, severity: "error", source: "javascript" };
  }
  return { message: String(error), severity: "error", source: "javascript" };
}
