import type { StdioBuffer } from "./stdio";
import { abortError } from "./transport";

export const JS_SANDBOX_FLAGS = "allow-scripts";

export function embedJson(value: unknown): string {
  return JSON.stringify(value).replace(/</g, "\\u003c");
}

export function buildJavaScriptSandboxDocument(code: string, messageId: string): string {
  return `<!doctype html><html><head><meta charset="utf-8"></head><body><script>
(function () {
  var id = ${embedJson(messageId)};
  var stdout = [];
  var stderr = [];
  function format(args) {
    return Array.prototype.map.call(args, function (value) {
      if (typeof value === "string") return value;
      if (value === undefined) return "undefined";
      if (value === null) return "null";
      try { return JSON.stringify(value); } catch (error) { return String(value); }
    }).join(" ") + "\\n";
  }
  var consoleLike = {
    log: function () { stdout.push(format(arguments)); },
    info: function () { stdout.push(format(arguments)); },
    warn: function () { stderr.push(format(arguments)); },
    error: function () { stderr.push(format(arguments)); }
  };
  try {
    var run = new Function("console", ${embedJson(`"use strict";\n${code}`)});
    var value = run(consoleLike);
    parent.postMessage({
      id: id,
      stdout: stdout,
      stderr: stderr,
      value: value === undefined ? undefined : String(value)
    }, "*");
  } catch (error) {
    var message = error && error.message ? String(error.message) : String(error);
    parent.postMessage({ id: id, stdout: stdout, stderr: stderr, error: message }, "*");
  }
})();
</script></body></html>`;
}

export function buildJavaScriptWorkerSource(): string {
  return `
(function () {
  function format(args) {
    return Array.prototype.map.call(args, function (value) {
      if (typeof value === "string") return value;
      if (value === undefined) return "undefined";
      if (value === null) return "null";
      try { return JSON.stringify(value); } catch (error) { return String(value); }
    }).join(" ") + "\\n";
  }
  self.onmessage = function (event) {
    var data = event.data || {};
    if (!data.id) return;
    var stdout = [];
    var stderr = [];
    var consoleLike = {
      log: function () { stdout.push(format(arguments)); },
      info: function () { stdout.push(format(arguments)); },
      warn: function () { stderr.push(format(arguments)); },
      error: function () { stderr.push(format(arguments)); }
    };
    try {
      var run = new Function("console", '"use strict";\\n' + String(data.code || ""));
      var value = run(consoleLike);
      self.postMessage({
        id: data.id,
        stdout: stdout,
        stderr: stderr,
        value: value === undefined ? undefined : String(value)
      });
    } catch (error) {
      var message = error && error.message ? String(error.message) : String(error);
      self.postMessage({ id: data.id, stdout: stdout, stderr: stderr, error: message });
    }
  };
})();
`;
}

export interface SandboxMessage {
  id?: string;
  stdout?: string[];
  stderr?: string[];
  value?: string;
  error?: string;
}

export function applySandboxStreams(stdio: StdioBuffer, message: SandboxMessage): void {
  for (const text of message.stdout ?? []) {
    stdio.push("stdout", text);
  }
  for (const text of message.stderr ?? []) {
    stdio.push("stderr", text);
  }
}

const WORKER_UNAVAILABLE_CODE = "ERR_SCRIPT_WORKER_UNAVAILABLE";

function workerUnavailableError(error: unknown): Error {
  const message =
    error instanceof Error && error.message
      ? error.message
      : "JavaScript worker sandbox is unavailable.";
  return Object.assign(new Error(message), { code: WORKER_UNAVAILABLE_CODE });
}

export function isSandboxWorkerUnavailable(error: unknown): boolean {
  return Boolean(
    error && typeof error === "object" && "code" in error && error.code === WORKER_UNAVAILABLE_CODE,
  );
}

export function canUseSandboxWorker(): boolean {
  return (
    typeof Worker !== "undefined" &&
    typeof Blob !== "undefined" &&
    typeof URL !== "undefined" &&
    typeof URL.createObjectURL === "function" &&
    typeof URL.revokeObjectURL === "function"
  );
}

export async function executeInSandboxWorker(
  code: string,
  timeoutMs: number,
  stdio: StdioBuffer,
  signal?: AbortSignal,
): Promise<unknown> {
  if (!canUseSandboxWorker()) {
    throw workerUnavailableError(
      new Error("JavaScript worker sandbox needs Worker, Blob, and object URLs."),
    );
  }
  if (signal?.aborted) {
    throw abortError();
  }

  const messageId = `ox-code-play-${Math.random().toString(36).slice(2)}`;
  const url = URL.createObjectURL(
    new Blob([buildJavaScriptWorkerSource()], { type: "text/javascript" }),
  );

  let worker: Worker;
  try {
    worker = new Worker(url);
  } catch (error) {
    URL.revokeObjectURL(url);
    throw workerUnavailableError(error);
  }

  return new Promise((resolve, reject) => {
    let settled = false;
    const cleanup = () => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timer);
      signal?.removeEventListener("abort", onAbort);
      worker.onmessage = null;
      worker.onerror = null;
      worker.terminate();
      URL.revokeObjectURL(url);
    };
    const onAbort = () => {
      cleanup();
      reject(abortError());
    };
    const onMessage = (event: MessageEvent<SandboxMessage>) => {
      if (event.data?.id !== messageId) {
        return;
      }
      cleanup();
      applySandboxStreams(stdio, event.data);
      if (event.data.error) {
        reject(new Error(event.data.error));
        return;
      }
      resolve(event.data.value);
    };
    const onError = (event: ErrorEvent) => {
      cleanup();
      reject(
        workerUnavailableError(new Error(event.message || "JavaScript worker sandbox failed.")),
      );
    };
    const timer = setTimeout(() => {
      cleanup();
      reject(
        Object.assign(new Error("JavaScript execution timed out."), {
          code: "ERR_SCRIPT_EXECUTION_TIMEOUT",
        }),
      );
    }, timeoutMs);
    worker.onmessage = onMessage;
    worker.onerror = onError;
    signal?.addEventListener("abort", onAbort, { once: true });
    worker.postMessage({ id: messageId, code });
  });
}

export async function executeInSandboxIframe(
  code: string,
  timeoutMs: number,
  stdio: StdioBuffer,
  signal?: AbortSignal,
): Promise<unknown> {
  if (typeof document === "undefined" || typeof window === "undefined") {
    throw new Error("JavaScript sandbox iframe needs a document.");
  }
  if (signal?.aborted) {
    throw abortError();
  }
  const messageId = `ox-code-play-${Math.random().toString(36).slice(2)}`;
  return new Promise((resolve, reject) => {
    const frame = document.createElement("iframe");
    frame.setAttribute("sandbox", JS_SANDBOX_FLAGS);
    frame.setAttribute("title", "Code Play JavaScript sandbox");
    frame.hidden = true;
    const cleanup = () => {
      window.clearTimeout(timer);
      window.removeEventListener("message", onMessage);
      signal?.removeEventListener("abort", onAbort);
      frame.remove();
    };
    const onAbort = () => {
      cleanup();
      reject(abortError());
    };
    const onMessage = (event: MessageEvent<SandboxMessage>) => {
      if (event.source !== frame.contentWindow || event.data?.id !== messageId) {
        return;
      }
      cleanup();
      applySandboxStreams(stdio, event.data);
      if (event.data.error) {
        reject(new Error(event.data.error));
        return;
      }
      resolve(event.data.value);
    };
    const timer = window.setTimeout(() => {
      cleanup();
      reject(
        Object.assign(new Error("JavaScript execution timed out."), {
          code: "ERR_SCRIPT_EXECUTION_TIMEOUT",
        }),
      );
    }, timeoutMs);
    window.addEventListener("message", onMessage);
    signal?.addEventListener("abort", onAbort, { once: true });
    frame.srcdoc = buildJavaScriptSandboxDocument(code, messageId);
    document.body.append(frame);
  });
}
