import * as fs from "node:fs/promises";
import * as path from "node:path";
import {
  planRedirectFiles,
  resolveRedirectsOptions,
  type RedirectFilePlan,
  type RedirectPageInput,
} from "./redirects";
import type { RedirectProvider, RedirectsOptions, ResolvedRedirectsOptions } from "./types";

export interface CustomHostRedirectRoute {
  path: string;
  aliases?: readonly string[];
  redirect?: string;
}

export interface PlanRedirectOutputsInput {
  redirects?: boolean | RedirectsOptions | Record<string, string> | ResolvedRedirectsOptions | null;
  routes?: readonly CustomHostRedirectRoute[];
  occupiedPaths?: readonly string[];
  base?: string;
  env?: NodeJS.ProcessEnv;
}

export type PlannedRedirectOutput =
  | {
      kind: "html";
      path: string;
      from: string;
      to: string;
      contents: string;
    }
  | {
      kind: "provider";
      provider: RedirectProvider;
      path: "_redirects";
      contents: string;
    }
  | {
      kind: "headers";
      path: "_headers";
      contents: string;
    }
  | {
      kind: "json";
      path: "redirects.json";
      contents: string;
    };

export interface RedirectOutputsPlan {
  outputs: PlannedRedirectOutput[];
}

export interface WriteRedirectOutputsInput extends PlanRedirectOutputsInput {
  outDir: string;
}

export interface WriteRedirectOutputsResult {
  files: string[];
  outputs: PlannedRedirectOutput[];
}

export function planRedirectOutputs(input: PlanRedirectOutputsInput): RedirectOutputsPlan {
  const options = resolvePublicRedirectOptions(input.redirects, input.env);
  const plan = planRedirectFiles({
    options,
    base: input.base,
    pages: redirectPages(input.routes, input.occupiedPaths),
  });
  const outputs: PlannedRedirectOutput[] = plan.files.map((file) => htmlOutput(file));

  if (plan.netlify && options.provider) {
    outputs.push({
      kind: "provider",
      provider: options.provider,
      path: "_redirects",
      contents: plan.netlify,
    });
  }
  if (plan.headers) {
    outputs.push({ kind: "headers", path: "_headers", contents: plan.headers });
  }
  if (plan.json) {
    outputs.push({ kind: "json", path: "redirects.json", contents: plan.json });
  }

  return { outputs };
}

export async function writeRedirectOutputs(
  input: WriteRedirectOutputsInput,
): Promise<WriteRedirectOutputsResult> {
  const plan = planRedirectOutputs(input);
  if (plan.outputs.length === 0) {
    return { files: [], outputs: [] };
  }

  await fs.mkdir(input.outDir, { recursive: true });
  const files: string[] = [];
  for (const output of plan.outputs) {
    const outputPath = outputFile(input.outDir, output.path);
    if (output.kind === "html") {
      if (await pathExists(outputPath)) {
        continue;
      }
      await fs.mkdir(path.dirname(outputPath), { recursive: true });
    }
    await fs.writeFile(outputPath, output.contents, "utf8");
    files.push(outputPath);
  }

  return { files, outputs: plan.outputs };
}

function resolvePublicRedirectOptions(
  value: PlanRedirectOutputsInput["redirects"],
  env: NodeJS.ProcessEnv | undefined,
): ResolvedRedirectsOptions {
  if (value && typeof value === "object" && "enabled" in value) {
    return value as ResolvedRedirectsOptions;
  }
  return resolveRedirectsOptions(value ?? undefined, env);
}

function redirectPages(
  routes: readonly CustomHostRedirectRoute[] | undefined,
  occupiedPaths: readonly string[] | undefined,
): RedirectPageInput[] {
  return [
    ...(routes ?? []).map((route) => ({
      dest: route.path,
      aliases: route.aliases,
      redirect: route.redirect,
    })),
    ...(occupiedPaths ?? []).map((dest) => ({ dest })),
  ];
}

function htmlOutput(file: RedirectFilePlan): PlannedRedirectOutput {
  return {
    kind: "html",
    path: file.relativePath,
    from: file.from,
    to: file.to,
    contents: file.html,
  };
}

async function pathExists(file: string): Promise<boolean> {
  try {
    await fs.access(file);
    return true;
  } catch {
    return false;
  }
}

function outputFile(outDir: string, relativePath: string): string {
  const root = path.resolve(outDir);
  const file = path.resolve(root, relativePath);
  const relative = path.relative(root, file);
  if (relative === "" || relative === ".." || relative.startsWith(`..${path.sep}`)) {
    throw new Error(`Redirect output path ${JSON.stringify(relativePath)} escapes outDir.`);
  }
  return file;
}
