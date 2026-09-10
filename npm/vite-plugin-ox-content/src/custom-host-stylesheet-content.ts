import * as fs from "node:fs/promises";
import type {
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetContent,
  OxContentCustomHostStylesheetContentDiagnostic,
  OxContentCustomHostStylesheetContentInput,
  OxContentCustomHostStylesheetContentResult,
} from "./custom-host-types";
import { resolveOutputPath } from "./custom-host-utils";

export interface ResolveCustomHostStylesheetContentInput extends OxContentCustomHostStylesheetContentInput {
  build: boolean;
  outDir: string;
  resolveDevContent?: CustomHostDevStylesheetContentResolver;
}

export type CustomHostDevStylesheetContentResolver = (href: string) => Promise<string | undefined>;

export async function resolveCustomHostStylesheetContent(
  input: ResolveCustomHostStylesheetContentInput,
): Promise<OxContentCustomHostStylesheetContentResult> {
  const stylesheets: OxContentCustomHostStylesheetContent[] = [];
  const diagnostics: OxContentCustomHostStylesheetContentDiagnostic[] = [];
  const reads = new Map<string, Promise<string>>();

  for (const stylesheet of input.stylesheets) {
    if (stylesheet.content != null) {
      stylesheets.push(stylesheetContent(stylesheet, stylesheet.content));
      continue;
    }
    if (!input.build) {
      const content = await input.resolveDevContent?.(stylesheet.href);
      if (content != null) {
        stylesheets.push(stylesheetContent(stylesheet, content));
        continue;
      }
      diagnostics.push({
        code: "unavailable",
        href: stylesheet.href,
        moduleId: stylesheet.moduleId,
        message: `Stylesheet content for "${stylesheet.href}" is only available during custom-host builds.`,
      });
      continue;
    }
    if (!stylesheet.outputPath) {
      diagnostics.push({
        code: "missing-artifact",
        href: stylesheet.href,
        moduleId: stylesheet.moduleId,
        message: `No build artifact path was recorded for stylesheet "${stylesheet.href}".`,
      });
      continue;
    }

    const outputPath = resolveOutputPath(input.outDir, stylesheet.outputPath);
    try {
      let read = reads.get(outputPath);
      if (!read) {
        read = fs.readFile(outputPath, "utf8");
        reads.set(outputPath, read);
      }
      stylesheets.push(stylesheetContent(stylesheet, await read));
    } catch (error) {
      reads.delete(outputPath);
      diagnostics.push({
        code: "missing-artifact",
        href: stylesheet.href,
        moduleId: stylesheet.moduleId,
        message: `Failed to read stylesheet artifact "${stylesheet.outputPath}" for "${stylesheet.href}": ${errorMessage(error)}`,
      });
    }
  }

  return { stylesheets, diagnostics };
}

function stylesheetContent(
  stylesheet: OxContentCustomHostStylesheet,
  content: string,
): OxContentCustomHostStylesheetContent {
  return {
    stylesheet,
    href: stylesheet.href,
    moduleId: stylesheet.moduleId,
    content,
  };
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
