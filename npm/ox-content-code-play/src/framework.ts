import { escapeHtml } from "./escape";
import { PhaseTracker } from "./timing";
import type { AdapterRequest, FrameworkId, RunResult } from "./types";

const RUNTIMES: Record<FrameworkId, { specifier: string; cdn: string }> = {
  vue: { specifier: "vue", cdn: "https://esm.sh/vue@3" },
  react: { specifier: "react", cdn: "https://esm.sh/react@19" },
  svelte: { specifier: "svelte", cdn: "https://esm.sh/svelte@5" },
  solid: { specifier: "solid-js", cdn: "https://esm.sh/solid-js@1" },
};

export async function runFramework(request: AdapterRequest): Promise<RunResult> {
  const tracker = new PhaseTracker();
  tracker.start("compile", "Compile preview");
  const framework = request.definition.framework ?? "vue";
  const html = buildPreviewDocument(framework, request.code);
  tracker.stop();
  return {
    status: "ok",
    stdio: [],
    diagnostics: [],
    provenance: {
      compile: { host: "local", runtime: `${framework}-preview` },
      execute: { host: "iframe", runtime: framework, sandbox: "srcdoc" },
    },
    timing: tracker.report(),
    preview: { kind: "html", html },
  };
}

export function buildPreviewDocument(framework: FrameworkId, code: string): string {
  const runtime = RUNTIMES[framework];
  return `<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>${escapeHtml(framework)} preview</title>
  <script type="importmap">${JSON.stringify({ imports: { [runtime.specifier]: runtime.cdn } })}</script>
  <style>html,body{margin:0;padding:1rem;font:14px/1.5 system-ui,sans-serif;}</style>
</head>
<body>
  <div id="app"></div>
  <script type="module">
${indentPreview(code)}
  </script>
</body>
</html>
`;
}

function indentPreview(code: string): string {
  return code
    .split("\n")
    .map((line) => `    ${line}`)
    .join("\n");
}
