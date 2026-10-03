import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { glob } from "glob";
import { loadPackageApi, loadResolvedOptions } from "./oxct-validate.mjs";

export async function loadFrontmatterProject(options) {
  const api = await loadPackageApi();
  const project = await loadResolvedOptions(options, api);
  const root = resolve(project.root, project.resolvedOptions.srcDir);
  const schemas = project.resolvedOptions.frontmatterSchemas;
  return {
    ...project,
    sourceRoot: root,
    schemas,
    api,
    shapes: Object.entries(schemas ?? {}).map(([pattern, schema]) => {
      let shape;
      try {
        shape = api.frontmatterJsonSchema(schema);
      } catch (error) {
        console.error(`[ox-content] Completion shape for ${pattern}: ${String(error)}`);
      }
      return { pattern, shape };
    }),
    async check(source, file) {
      const matched = !!api.selectFrontmatterSchema(schemas, file, root);
      const result = await api.checkFrontmatter(source, file, schemas, root);
      return { ...result, matched };
    },
  };
}

export async function runTypecheck(args) {
  if (args.includes("--help") || args.includes("-h")) {
    console.log(`oxct typecheck [files/globs] [--config <path>] [--format text|json]

Checks YAML frontmatter against source-relative frontmatterSchemas in the Vite config.
Executes the project configuration and Standard Schema validators, including async refinements.`);
    return;
  }
  const options = { cwd: process.cwd(), config: undefined, format: "text" };
  const patterns = [];
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (["--config", "-c", "--format"].includes(arg)) {
      const value = args[++i];
      if (!value || value.startsWith("-")) throw new Error(`Missing value for ${arg}`);
      options[arg === "--format" ? "format" : "config"] = value;
    } else if (arg.startsWith("-")) throw new Error(`Unknown typecheck option: ${arg}`);
    else patterns.push(arg);
  }
  if (!["text", "json"].includes(options.format)) throw new Error("Use --format text or json");
  // Project logs must not corrupt machine-readable output.
  const log = console.log;
  let project;
  try {
    console.log = (...values) => console.error(...values);
    project = await loadFrontmatterProject(options);
    const result = await checkProject(project, patterns, options);
    log(
      options.format === "json"
        ? JSON.stringify(result)
        : result.diagnostics.length
          ? ""
          : `[ox-content] Frontmatter typecheck passed for ${result.checked} document${result.checked === 1 ? "" : "s"}.`,
    );
    if (result.diagnostics.length) {
      if (options.format !== "json")
        console.error(project.api.formatFrontmatterDiagnostics(result.diagnostics));
      process.exitCode = 1;
    }
  } finally {
    console.log = log;
  }
}

async function checkProject(project, patterns, options) {
  if (!project.schemas || !Object.keys(project.schemas).length)
    throw new Error("No frontmatterSchemas configured in the project");
  const files = await glob(patterns.length ? patterns : "**/*.{md,mdc,mdx,markdown}", {
    cwd: patterns.length ? options.cwd : project.sourceRoot,
    absolute: true,
    nodir: true,
    dot: true,
    ignore: ["**/node_modules/**", "**/.git/**"],
  });
  const diagnostics = [];
  let checked = 0;
  for (const file of [...new Set(files)].sort()) {
    if (!project.api.selectFrontmatterSchema(project.schemas, file, project.sourceRoot)) continue;
    checked++;
    const result = await project.check(await readFile(file, "utf8"), file);
    diagnostics.push(...result.diagnostics);
  }
  if (!checked) throw new Error("No documents matched frontmatterSchemas");
  return { checked, diagnostics };
}
