import { executeScript } from "./javascript";
import { hasNodeVm } from "./runtime-host";
import { StdioBuffer } from "./stdio";
import { stripTypeScript } from "./strip-typescript";
import { PhaseTracker } from "./timing";
import type { AdapterRequest, AdapterResult, Diagnostic, TransportResponse } from "./types";

export async function typecheckTypeScript(request: AdapterRequest): Promise<AdapterResult> {
  const tracker = new PhaseTracker();
  tracker.start("typecheck", "Typecheck");

  if (request.endpoints.typecheck && !hasNodeVm()) {
    return typecheckViaEndpoint(request, tracker);
  }

  try {
    const diagnostics = await typecheckWithTsgo(
      request.code,
      String(request.config.tsgoCommand ?? "tsgo"),
    );
    tracker.stop();
    return {
      status: diagnostics.some((item) => item.severity === "error") ? "error" : "ok",
      stdio: [],
      diagnostics,
      provenance: { compile: { host: "local", runtime: "tsgo" } },
      timing: tracker.report(),
    };
  } catch (error) {
    tracker.stop();
    const message = error instanceof Error ? error.message : String(error);
    return {
      status: message.includes("ENOENT") ? "unsupported" : "error",
      stdio: [],
      diagnostics: [
        {
          message: message.includes("ENOENT")
            ? "tsgo is not available. Install @typescript/native-preview or set languages.typescript.config.tsgoCommand."
            : message,
          severity: "error",
          source: "tsgo",
        },
      ],
      provenance: { compile: { host: "local", runtime: "tsgo" } },
      timing: tracker.report(),
    };
  }
}

export async function runTypeScript(request: AdapterRequest): Promise<AdapterResult> {
  const tracker = new PhaseTracker();
  const stdio = new StdioBuffer(tracker.startedAt);
  tracker.start("compile", "Strip types");
  const javascript = stripTypeScript(request.code);
  tracker.start("execute", "Execute");
  try {
    const value = await executeScript(javascript, request.timeoutMs, stdio);
    tracker.stop();
    return {
      status: "ok",
      stdio: stdio.snapshot(),
      diagnostics: [],
      provenance: {
        compile: { host: "local", runtime: "strip-types" },
        execute: {
          host: "local",
          runtime: hasNodeVm() ? "node:vm" : "iframe",
          sandbox: hasNodeVm() ? "vm" : "srcdoc",
        },
      },
      timing: tracker.report(),
      value: value === undefined ? undefined : String(value),
    };
  } catch (error) {
    tracker.stop();
    const message = error instanceof Error ? error.message : String(error);
    stdio.push("stderr", `${message}\n`);
    return {
      status: "error",
      stdio: stdio.snapshot(),
      diagnostics: [{ message, severity: "error", source: "javascript" }],
      provenance: {
        compile: { host: "local", runtime: "strip-types" },
        execute: {
          host: "local",
          runtime: hasNodeVm() ? "node:vm" : "iframe",
          sandbox: hasNodeVm() ? "vm" : "srcdoc",
        },
      },
      timing: tracker.report(),
    };
  }
}

async function typecheckViaEndpoint(
  request: AdapterRequest,
  tracker: PhaseTracker,
): Promise<AdapterResult> {
  const url = request.endpoints.typecheck ?? "";
  const response = await request.transport.request({
    url,
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ language: "typescript", code: request.code, config: request.config }),
  });
  tracker.stop();
  return adapterResultFromTypecheckResponse(response, url, tracker);
}

export function typecheckEndpointFailureMessage(status: number, text: string): string {
  if (status === 404 || status === 405) {
    return "Typecheck needs a reachable endpoints.typecheck. The Vite /__ox-code-play/typecheck proxy exists only during vite dev.";
  }
  const trimmed = text.trim();
  return trimmed || "Typecheck endpoint failed.";
}

export function adapterResultFromTypecheckResponse(
  response: TransportResponse,
  url: string,
  tracker: PhaseTracker,
): AdapterResult {
  if (!response.ok) {
    return {
      status: "error",
      stdio: [],
      diagnostics: [
        {
          message: typecheckEndpointFailureMessage(response.status, response.text),
          severity: "error",
          source: "tsgo",
        },
      ],
      provenance: { compile: { host: hostFromUrl(url), runtime: "tsgo" } },
      timing: tracker.report(),
    };
  }
  try {
    return JSON.parse(response.text) as AdapterResult;
  } catch {
    return {
      status: "error",
      stdio: [],
      diagnostics: [
        {
          message: response.text || "Typecheck endpoint failed.",
          severity: "error",
          source: "tsgo",
        },
      ],
      provenance: { compile: { host: hostFromUrl(url), runtime: "tsgo" } },
      timing: tracker.report(),
    };
  }
}

export async function typecheckWithTsgo(code: string, command = "tsgo"): Promise<Diagnostic[]> {
  const [{ mkdtemp, rm, writeFile }, { tmpdir }, { join }, { execFile }, { promisify }] =
    await Promise.all([
      import("node:fs/promises"),
      import("node:os"),
      import("node:path"),
      import("node:child_process"),
      import("node:util"),
    ]);
  const execFileAsync = promisify(execFile);
  const dir = await mkdtemp(join(tmpdir(), "ox-code-play-"));
  const file = join(dir, "snippet.ts");
  await writeFile(file, code);
  try {
    await execFileAsync(command, ["--noEmit", "--pretty", "false", "--strict", file], {
      cwd: dir,
      maxBuffer: 1024 * 1024,
    });
    return [];
  } catch (error) {
    const output = commandOutput(error);
    if ((error as { code?: string }).code === "ENOENT") {
      throw error;
    }
    return parseTsgoOutput(output);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

export function parseTsgoOutput(output: string): Diagnostic[] {
  const diagnostics: Diagnostic[] = [];
  const pattern =
    /^(?:.*[\\/])?snippet\.ts\((\d+),(\d+)\):\s+(error|warning|info)\s+TS\d+:\s+(.*)$/gm;
  for (const match of output.matchAll(pattern)) {
    diagnostics.push({
      message: match[4] ?? output,
      severity: match[3] === "warning" ? "warning" : match[3] === "info" ? "info" : "error",
      line: Number(match[1]),
      column: Number(match[2]),
      source: "tsgo",
    });
  }
  if (diagnostics.length === 0 && output.trim()) {
    diagnostics.push({ message: output.trim(), severity: "error", source: "tsgo" });
  }
  return diagnostics;
}

function commandOutput(error: unknown): string {
  if (!error || typeof error !== "object") {
    return String(error);
  }
  const value = error as { stdout?: unknown; stderr?: unknown; message?: unknown };
  return [value.stdout, value.stderr, value.message]
    .filter((part): part is string => typeof part === "string" && part.trim().length > 0)
    .join("\n")
    .trim();
}

function hostFromUrl(url: string): string {
  try {
    return new URL(url, "https://code-play.local").host || url;
  } catch {
    return url;
  }
}
