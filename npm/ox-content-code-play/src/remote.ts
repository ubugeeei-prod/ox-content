import { StdioBuffer } from "./stdio";
import { PhaseTracker } from "./timing";
import type { AdapterRequest, AdapterResult } from "./types";

interface PistonExecuteResponse {
  compile?: { stdout?: string; stderr?: string; code?: number };
  run?: { stdout?: string; stderr?: string; code?: number; signal?: string | null };
  message?: string;
}

export async function runRemote(request: AdapterRequest): Promise<AdapterResult> {
  const tracker = new PhaseTracker();
  const stdio = new StdioBuffer(tracker.startedAt);
  const endpoint = request.enabled.endpoint;
  const language = request.definition.remote?.pistonLanguage ?? request.definition.id;

  if (!endpoint) {
    tracker.stop();
    return {
      status: "unsupported",
      stdio: [],
      diagnostics: [
        {
          message:
            `${request.definition.name} execution needs a configured HTTP executor. ` +
            `Pass languages.${request.definition.id}.endpoint (Piston-compatible).`,
          severity: "error",
          source: "code-play",
        },
      ],
      provenance: {},
      timing: tracker.report(),
    };
  }

  tracker.start("queue", "Queue");
  tracker.start("compile", "Compile / execute");
  const response = await request.transport.request({
    url: joinEndpoint(endpoint, "execute"),
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      language,
      version: request.config.version ?? request.definition.remote?.pistonVersion ?? "*",
      files: [{ content: request.code }],
    }),
  });
  tracker.start("collect", "Collect output");

  const parsed = parseResponse(response.text);
  if (parsed.compile?.stdout) {
    stdio.push("stdout", parsed.compile.stdout);
  }
  if (parsed.compile?.stderr) {
    stdio.push("stderr", parsed.compile.stderr);
  }
  if (parsed.run?.stdout) {
    stdio.push("stdout", parsed.run.stdout);
  }
  if (parsed.run?.stderr) {
    stdio.push("stderr", parsed.run.stderr);
  }
  if (parsed.message && !parsed.run && !parsed.compile) {
    stdio.push("stderr", `${parsed.message}\n`);
  }

  const compileFailed = (parsed.compile?.code ?? 0) !== 0;
  const runFailed = (parsed.run?.code ?? 0) !== 0 || Boolean(parsed.run?.signal);
  const failed =
    !response.ok || compileFailed || runFailed || Boolean(parsed.message && !parsed.run);
  tracker.stop();

  return {
    status: failed ? "error" : "ok",
    stdio: stdio.snapshot(),
    diagnostics: failed
      ? [
          {
            message:
              parsed.message ??
              parsed.run?.stderr ??
              parsed.compile?.stderr ??
              "Remote execution failed.",
            severity: "error",
            source: language,
          },
        ]
      : [],
    provenance: {
      compile: parsed.compile
        ? { host: hostFromUrl(endpoint), runtime: language, sandbox: "piston" }
        : undefined,
      execute: { host: hostFromUrl(endpoint), runtime: language, sandbox: "piston" },
    },
    timing: tracker.report(),
  };
}

function parseResponse(text: string): PistonExecuteResponse {
  try {
    return JSON.parse(text) as PistonExecuteResponse;
  } catch {
    return { message: text };
  }
}

function joinEndpoint(endpoint: string, action: string): string {
  const trimmed = endpoint.replace(/\/+$/, "");
  return trimmed.endsWith(action) ? trimmed : `${trimmed}/${action}`;
}

function hostFromUrl(url: string): string {
  try {
    return new URL(url, "https://code-play.local").host || url;
  } catch {
    return url;
  }
}
