import * as fs from "node:fs/promises";
import * as path from "node:path";
import { generateOgImages } from "@ox-content/vite-plugin";
import type { ResolvedSlidesPluginOptions, SlideDeckData } from "./internal-types";
import { buildSlideDecks } from "./decks";
import { renderDeckPrintHtml, renderRouteHtml } from "./html";
import { loadNapiModule } from "./napi";
import { generateDeckPdfs } from "./pdf";

async function writeGeneratedHtml(outputPath: string, html: string): Promise<string> {
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(outputPath, html, "utf-8");
  return outputPath;
}

function collectOgEntries(
  decks: SlideDeckData[],
): Array<{ slide: SlideDeckData["slides"][number]; deck: SlideDeckData }> {
  return decks.flatMap((deck) => deck.slides.map((slide) => ({ slide, deck })));
}

/**
 * Writes slide HTML, presenter HTML, OG images, and optional PDFs to disk.
 */
export async function buildOutput(
  options: ResolvedSlidesPluginOptions,
  root: string,
): Promise<{ files: string[]; errors: string[] }> {
  const files: string[] = [];
  const errors: string[] = [];
  const outDir = path.resolve(root, options.outDir);
  const routeOutDir = path.join(outDir, options.routeBase);
  const decks = await buildSlideDecks(options, root);
  const napi = await loadNapiModule();

  if (options.ssg.clean) {
    await fs.rm(routeOutDir, { recursive: true, force: true });
  }

  if (options.ssg.generateOgImage) {
    const results = await generateOgImages(
      collectOgEntries(decks).map(({ slide, deck }) => ({
        props: {
          title: slide.title,
          description: slide.description ?? deck.description,
          siteName: deck.title,
        },
        outputPath: path.join(path.dirname(slide.outputPath), "og-image.png"),
      })),
      options.ogImageOptions,
      root,
    );

    for (const result of results) {
      if (result.error) {
        errors.push(`OG image failed for ${result.outputPath}: ${result.error}`);
      } else {
        files.push(result.outputPath);
      }
    }
  }

  const pdfTasks: Array<{ htmlPath: string; pdfPath: string }> = [];
  for (const deck of decks) {
    const deckFiles = await Promise.all(
      deck.slides.map(async (slide) => {
        const generatedFiles: string[] = [];
        const slideHtml = await renderRouteHtml(options, { deck, slide, presenter: false }, napi);
        generatedFiles.push(await writeGeneratedHtml(slide.outputPath, slideHtml));
        if (slide.slideNumber === 1)
          generatedFiles.push(await writeGeneratedHtml(deck.outputPath, slideHtml));

        if (options.presenter && slide.presenterOutputPath) {
          const presenterHtml = await renderRouteHtml(
            options,
            { deck, slide, presenter: true },
            napi,
          );
          generatedFiles.push(await writeGeneratedHtml(slide.presenterOutputPath, presenterHtml));
          if (slide.slideNumber === 1 && deck.presenterOutputPath) {
            generatedFiles.push(await writeGeneratedHtml(deck.presenterOutputPath, presenterHtml));
          }
        }

        return generatedFiles;
      }),
    );

    files.push(...deckFiles.flat());
    if (options.pdf.enabled && deck.printOutputPath && deck.pdfOutputPath) {
      files.push(
        await writeGeneratedHtml(deck.printOutputPath, renderDeckPrintHtml(options, deck, napi)),
      );
      pdfTasks.push({ htmlPath: deck.printOutputPath, pdfPath: deck.pdfOutputPath });
    }
  }

  if (pdfTasks.length > 0) {
    const pdfResult = await generateDeckPdfs(pdfTasks, outDir, options.pdf);
    files.push(...pdfResult.files);
    errors.push(...pdfResult.errors);
  }

  return { files, errors };
}
