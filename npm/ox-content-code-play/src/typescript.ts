import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { executeScript } from "./javascript";
import { StdioBuffer } from "./stdio";
import { stripTypeScript } from "./strip-typescript";
import { PhaseTracker } from "./timing";
import type { AdapterRequest, Diagnostic, RunResult } from "./types";

const execFileAsync = promisify(execFile);

export async function typecheckTypeScript(request: AdapterRequest): Promise<RunResult> {
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

export async function runTypeScript(request: AdapterRequest): Promise<RunResult> {
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
          runtime: hasNodeVm() ? "node:vm" : "function",
          sandbox: hasNodeVm() ? "vm" : "function",
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
        execute: { host: "local", runtime: hasNodeVm() ? "node:vm" : "function" },
      },
      timing: tracker.report(),
    };
  }
}

async function typecheckViaEndpoint(
  request: AdapterRequest,
  tracker: PhaseTracker,
): Promise<RunResult> {
  const response = await request.transport.request({
    url: request.endpoints.typecheck ?? "",
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ language: "typescript", code: request.code, config: request.config }),
  });
  tracker.stop();
  try {
    return JSON.parse(response.text) as RunResult;
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
      provenance: {
        compile: { host: hostFromUrl(request.endpoints.typecheck ?? ""), runtime: "tsgo" },
      },
      timing: tracker.report(),
    };
  }
}

export async function typecheckWithTsgo(code: string, command = "tsgo"): Promise<Diagnostic[]> {
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

function hasNodeVm(): boolean {
  return typeof process !== "undefined" && Boolean(process.versions?.node);
}

function hostFromUrl(url: string): string {
  try {
    return new URL(url, "https://code-play.local").host || url;
  } catch {
    return url;
  }
}
