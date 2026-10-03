import { parentPort, workerData } from "node:worker_threads";
import { watch } from "node:fs";
import { basename, dirname, resolve } from "node:path";
import { loadFrontmatterProject } from "./oxct-frontmatter-project.mjs";

let project;
let epoch = 0;
let watchers = [];
let timer;
let loading;
let reloadAgain = false;
const configFile = resolve(workerData.cwd, workerData.config ?? "vite.config.ts");
// Keep recovery observable even when the initial config cannot be evaluated.
try {
  const watcher = watch(dirname(configFile), (_, file) => {
    if (
      file &&
      (workerData.config
        ? String(file) !== basename(configFile)
        : !/^(vite|ox-content)\.config\./.test(String(file)))
    )
      return;
    clearTimeout(timer);
    timer = setTimeout(() => void reload(), 120);
  });
  watcher.on("error", (error) => console.error(error));
} catch (error) {
  console.error(error);
}

async function reload() {
  if (loading) {
    reloadAgain = true;
    return loading;
  }
  loading = (async () => {
    const current = ++epoch;
    try {
      project = await loadFrontmatterProject(workerData);
      parentPort.postMessage({
        type: "ready",
        epoch: current,
        root: project.sourceRoot,
        shapes: project.shapes,
      });
      for (const watcher of watchers) watcher.close();
      watchers = project.dependencies.map((file) => {
        // Watch the parent so atomic editor saves remain observable.
        const name = basename(file);
        const directory = dirname(file);
        const watcher = watch(directory, (_, changed) => {
          if (changed && String(changed) !== name) return;
          clearTimeout(timer);
          timer = setTimeout(() => void reload(), 120);
        });
        watcher.on("error", (error) => console.error(error));
        return watcher;
      });
    } catch (error) {
      project = undefined;
      parentPort.postMessage({ type: "error", epoch: current, message: String(error) });
    }
  })();
  try {
    await loading;
  } finally {
    loading = undefined;
    if (reloadAgain) {
      reloadAgain = false;
      void reload();
    }
  }
}

parentPort.on("message", async (request) => {
  if (request.type === "reload") {
    await reload();
    return;
  }
  if (request.type !== "validate") return;
  if (loading) await loading;
  const current = epoch;
  try {
    const result = project ? await project.check(request.source, request.file) : undefined;
    // Validation outputs can contain functions or custom instances. The editor only
    // needs diagnostics; keep validated page data inside the worker.
    const diagnostics = result?.diagnostics.map(({ path, ...issue }) => issue);
    parentPort.postMessage({
      type: "result",
      id: request.id,
      epoch: current,
      result: result ? { matched: result.matched, diagnostics } : undefined,
    });
  } catch (error) {
    parentPort.postMessage({
      type: "result",
      id: request.id,
      epoch: current,
      result: {
        matched: true,
        diagnostics: [{ line: 1, column: 1, endLine: 1, endColumn: 2, message: String(error) }],
      },
    });
  }
});
await reload();
