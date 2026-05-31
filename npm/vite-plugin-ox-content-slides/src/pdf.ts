import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { ResolvedSlidePdfOptions } from "./pdf-options";
import { relativeUrlPath, startStaticServer } from "./pdf-server";

/**
 * HTML/PDF pair scheduled for deck export.
 */
export interface DeckPdfTask {
  htmlPath: string;
  pdfPath: string;
}

/**
 * Result collected from PDF generation.
 */
export interface DeckPdfResult {
  files: string[];
  errors: string[];
}

/**
 * Generates deck PDFs from previously written print-shell HTML files.
 */
export async function generateDeckPdfs(
  tasks: DeckPdfTask[],
  rootDir: string,
  options: ResolvedSlidePdfOptions,
): Promise<DeckPdfResult> {
  const files: string[] = [];
  const errors: string[] = [];

  if (!options.enabled || tasks.length === 0) {
    return { files, errors };
  }

  try {
    const { chromium } = await import("playwright");
    const server = await startStaticServer(rootDir);
    const browser = await chromium.launch({
      headless: true,
      args: [
        "--no-sandbox",
        "--disable-setuid-sandbox",
        "--disable-dev-shm-usage",
        "--disable-gpu",
      ],
    });

    try {
      const page = await browser.newPage();
      try {
        for (const task of tasks) {
          await page.goto(`${server.origin}/${relativeUrlPath(rootDir, task.htmlPath)}`, {
            waitUntil: "networkidle",
          });
          await page.waitForFunction(() => document.fonts.status === "loaded");
          await fs.mkdir(path.dirname(task.pdfPath), { recursive: true });
          await page.pdf({
            path: task.pdfPath,
            width: options.pageWidth,
            height: options.pageHeight,
            scale: options.scale,
            printBackground: true,
            preferCSSPageSize: true,
            margin: { top: "0", right: "0", bottom: "0", left: "0" },
          });
          files.push(task.pdfPath);
        }
      } finally {
        await page.close();
      }
    } finally {
      await browser.close();
      await server.close();
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    errors.push(`PDF export skipped because Chromium was not available: ${message}`);
  }

  return { files, errors };
}
