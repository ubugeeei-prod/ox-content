import type { StdioBuffer } from "./stdio";

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

export async function executeInSandboxIframe(
  code: string,
  timeoutMs: number,
  stdio: StdioBuffer,
): Promise<unknown> {
  if (typeof document === "undefined" || typeof window === "undefined") {
    throw new Error("JavaScript sandbox iframe needs a document.");
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
      frame.remove();
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
    frame.srcdoc = buildJavaScriptSandboxDocument(code, messageId);
    document.body.append(frame);
  });
}
