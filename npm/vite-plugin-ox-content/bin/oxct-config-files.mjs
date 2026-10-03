import { mkdir, readFile, writeFile, rename, rm, stat } from "node:fs/promises";
import { dirname } from "node:path";
import { randomUUID } from "node:crypto";
import { applyEdits, modify, parse } from "jsonc-parser";

export async function readOptional(file) {
  try {
    return await readFile(file, "utf8");
  } catch (error) {
    if (error.code === "ENOENT") return undefined;
    throw error;
  }
}

export async function planJsonEdit(file, edits) {
  const original = await readOptional(file);
  const errors = [];
  const value = parse(original ?? "{}", errors, { allowTrailingComma: true });
  if (errors.length || !value || typeof value !== "object" || Array.isArray(value))
    throw new Error(`Invalid configuration: ${file}. Fix it before running setup.`);
  let content = original ?? "{}\n";
  for (const [path, next] of edits) {
    content = applyEdits(
      content,
      modify(content, path, next, {
        formattingOptions: { insertSpaces: true, tabSize: 2, eol: "\n" },
      }),
    );
  }
  return { file, original, content: content.endsWith("\n") ? content : content + "\n" };
}

export async function writePlan(plan) {
  if (plan.original === plan.content) return false;
  await mkdir(dirname(plan.file), { recursive: true });
  if ((await readOptional(plan.file)) !== plan.original)
    throw new Error(`Configuration changed during setup: ${plan.file}. Run setup again.`);
  if (plan.original !== undefined) {
    const backup = `${plan.file}.oxct-${randomUUID()}.bak`;
    await writeFile(backup, plan.original, { flag: "wx", mode: 0o600 });
  }
  if (plan.original === undefined) await writeFile(plan.file, plan.content, { flag: "wx" });
  else {
    const temp = `${plan.file}.oxct-${randomUUID()}.tmp`;
    try {
      const { mode } = await stat(plan.file);
      await writeFile(temp, plan.content, { flag: "wx", mode: mode & 0o777 });
      if ((await readOptional(plan.file)) !== plan.original)
        throw new Error(`Configuration changed during setup: ${plan.file}. Run setup again.`);
      await rename(temp, plan.file);
    } finally {
      await rm(temp, { force: true });
    }
  }
  return true;
}

export function parseConfig(content) {
  return parse(content ?? "{}");
}
