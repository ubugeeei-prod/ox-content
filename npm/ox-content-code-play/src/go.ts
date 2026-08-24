import { StdioBuffer } from "./stdio";
import { PhaseTracker } from "./timing";
import type { AdapterRequest, AdapterResult, Diagnostic } from "./types";

interface GoPlaygroundEvent {
  Message?: string;
  Kind?: string;
  Delay?: number;
}

interface GoPlaygroundResponse {
  Errors?: string;
  Events?: GoPlaygroundEvent[];
  VetErrors?: string;
}

export async function runGo(
  request: AdapterRequest,
  mode: "execute" | "typecheck",
): Promise<AdapterResult> {
  const tracker = new PhaseTracker();
  const stdio = new StdioBuffer(tracker.startedAt);
  const params = new URLSearchParams({
    version: "2",
    body: request.code,
    withVet: request.config.withVet === false ? "false" : "true",
  });

  tracker.start(
    mode === "typecheck" ? "typecheck" : "compile",
    mode === "typecheck" ? "Typecheck" : "Compile",
  );
  const response = await request.transport.request({
    url: request.endpoints.go,
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: params.toString(),
    signal: request.signal,
  });
  tracker.start("collect", "Collect output");

  const parsed = parseResponse(response.text);
  const diagnostics = [
    ...parseGoErrors(parsed.Errors ?? "", "go"),
    ...parseGoErrors(parsed.VetErrors ?? "", "vet"),
  ];
  for (const event of parsed.Events ?? []) {
    const stream = event.Kind === "stderr" ? "stderr" : "stdout";
    stdio.push(stream, event.Message ?? "");
  }
  if (parsed.Errors) {
    stdio.push("stderr", parsed.Errors);
  }

  tracker.stop();
  const failed = diagnostics.some((item) => item.severity === "error") || !response.ok;
  return {
    status: failed ? "error" : "ok",
    stdio: stdio.snapshot(),
    diagnostics,
    provenance: {
      compile: {
        host: hostFromUrl(request.endpoints.go),
        runtime: "go",
      },
      execute:
        mode === "execute"
          ? {
              host: hostFromUrl(request.endpoints.go),
              runtime: "go-playground",
              sandbox: "playground",
            }
          : undefined,
    },
    timing: tracker.report(),
  };
}

function parseResponse(text: string): GoPlaygroundResponse {
  try {
    return JSON.parse(text) as GoPlaygroundResponse;
  } catch {
    return { Errors: text };
  }
}

export function parseGoErrors(output: string, source: string): Diagnostic[] {
  if (!output.trim()) {
    return [];
  }
  return output
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const match = /^(?:prog\.go:)?(\d+)(?::(\d+))?:\s*(.*)$/.exec(line);
      return {
        message: match?.[3] ?? line,
        severity: "error" as const,
        line: match?.[1] ? Number(match[1]) : undefined,
        column: match?.[2] ? Number(match[2]) : undefined,
        source,
      };
    });
}

function hostFromUrl(url: string): string {
  try {
    return new URL(url, "https://code-play.local").host || url;
  } catch {
    return url;
  }
}
