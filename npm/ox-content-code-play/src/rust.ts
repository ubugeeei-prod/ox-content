import { StdioBuffer } from "./stdio";
import { PhaseTracker } from "./timing";
import type { AdapterRequest, Diagnostic, RunResult } from "./types";

interface RustPlaygroundResponse {
  success?: boolean;
  stdout?: string;
  stderr?: string;
  error?: string;
}

export async function runRust(
  request: AdapterRequest,
  mode: "execute" | "typecheck",
): Promise<RunResult> {
  const tracker = new PhaseTracker();
  const stdio = new StdioBuffer(tracker.startedAt);
  const crateType = resolveCrateType(request.code, String(request.config.crateType ?? "auto"));
  const body = {
    channel: request.config.channel ?? "stable",
    mode: request.config.mode ?? "debug",
    edition: String(request.config.edition ?? "2024"),
    crateType,
    tests: false,
    code: request.code,
    backtrace: false,
  };

  tracker.start(
    mode === "typecheck" ? "typecheck" : "compile",
    mode === "typecheck" ? "Typecheck" : "Compile",
  );
  const response = await request.transport.request({
    url: request.endpoints.rust,
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  tracker.start("collect", "Collect output");

  const parsed = parseResponse(response.text);
  const diagnostics = parseRustcDiagnostics(parsed.stderr ?? parsed.error ?? "");
  if (parsed.stdout) {
    stdio.push("stdout", parsed.stdout);
  }
  if (parsed.stderr) {
    stdio.push("stderr", parsed.stderr);
  }
  if (parsed.error && !parsed.stderr) {
    stdio.push("stderr", parsed.error);
  }

  const success = parsed.success === true && response.ok;
  const compileFailed = diagnostics.some((item) => item.severity === "error") || !success;
  tracker.stop();

  return {
    status: compileFailed ? "error" : "ok",
    stdio: stdio.snapshot(),
    diagnostics,
    provenance: {
      compile: {
        host: hostFromUrl(request.endpoints.rust),
        runtime: "rustc",
        version: String(request.config.channel ?? "stable"),
        target: crateType,
      },
      execute:
        mode === "execute"
          ? {
              host: hostFromUrl(request.endpoints.rust),
              runtime: "rust-playground",
              sandbox: "playground",
            }
          : undefined,
    },
    timing: tracker.report(),
  };
}

function resolveCrateType(code: string, configured: string): "bin" | "lib" {
  if (configured === "bin" || configured === "lib") {
    return configured;
  }
  return /\bfn\s+main\s*\(/.test(code) ? "bin" : "lib";
}

function parseResponse(text: string): RustPlaygroundResponse {
  try {
    return JSON.parse(text) as RustPlaygroundResponse;
  } catch {
    return { success: false, error: text };
  }
}

export function parseRustcDiagnostics(stderr: string): Diagnostic[] {
  const diagnostics: Diagnostic[] = [];
  const pattern = /^(error|warning|note)(?:\[([^\]]+)\])?:\s+(.*)$/gm;
  for (const match of stderr.matchAll(pattern)) {
    const severity = match[1] === "warning" ? "warning" : match[1] === "note" ? "info" : "error";
    diagnostics.push({
      message: match[3] ?? "",
      severity,
      source: match[2] ? `rustc ${match[2]}` : "rustc",
    });
  }
  return diagnostics;
}

function hostFromUrl(url: string): string {
  try {
    return new URL(url, "https://code-play.local").host || url;
  } catch {
    return url;
  }
}
