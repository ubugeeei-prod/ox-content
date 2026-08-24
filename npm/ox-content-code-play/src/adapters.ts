import { runFramework } from "./framework";
import { runGo } from "./go";
import { runJavaScript } from "./javascript";
import { runRemote } from "./remote";
import { runRust } from "./rust";
import { runTypeScript, typecheckTypeScript } from "./typescript";
import type { AdapterRequest, AdapterResult } from "./types";

export async function executeAdapter(request: AdapterRequest): Promise<AdapterResult> {
  if (!request.enabled.execute) {
    return capabilityDisabled(request, "execute");
  }
  switch (request.definition.backend) {
    case "javascript":
      return runJavaScript(request);
    case "typescript":
      return runTypeScript(request);
    case "framework":
      return runFramework(request);
    case "rust-playground":
      return runRust(request, "execute");
    case "go-playground":
      return runGo(request, "execute");
    case "remote":
      return runRemote(request);
    default:
      return capabilityDisabled(request, "execute");
  }
}

export async function typecheckAdapter(request: AdapterRequest): Promise<AdapterResult> {
  if (!request.enabled.typecheck || !request.definition.capabilities.typecheck) {
    return capabilityDisabled(request, "typecheck");
  }
  switch (request.definition.backend) {
    case "typescript":
      return typecheckTypeScript(request);
    case "rust-playground":
      return runRust(request, "typecheck");
    case "go-playground":
      return runGo(request, "typecheck");
    default:
      return capabilityDisabled(request, "typecheck");
  }
}

function capabilityDisabled(
  request: AdapterRequest,
  action: "execute" | "typecheck",
): AdapterResult {
  return {
    status: "unsupported",
    stdio: [],
    diagnostics: [
      {
        message: `${request.definition.name} ${action} is not enabled.`,
        severity: "error",
        source: "code-play",
      },
    ],
    provenance: {},
    timing: { totalMs: 0, phases: [] },
  };
}
