import { describe, expect, it } from "vite-plus/test";
import { transformMediaEmbeds, type MediaEmbedOptions } from "./media";

/**
 * The transform pre-scans for a provider tag before doing any work. That list
 * used to be hand-kept in this file and had drifted: `codesandbox` was absent,
 * so a page whose only embed was a `<CodeSandbox>` skipped the transform and
 * shipped the raw tag. It now comes from the Rust registry.
 */
describe("the media marker pre-scan", () => {
  it("recognises a document whose only embed is CodeSandbox", async () => {
    const html = '<p><codesandbox url="https://codesandbox.io/p/sandbox/abc123"></codesandbox></p>';
    const out = await transformMediaEmbeds(html, { playgrounds: true } as MediaEmbedOptions);
    expect(out).toContain("ox-provider-card--codesandbox");
  });

  it("recognises each provider added to the catalog", async () => {
    const cases: [string, string, MediaEmbedOptions][] = [
      ["loom", "https://www.loom.com/share/abcdef1234567890", { loom: true }],
      ["asciinema", "https://asciinema.org/a/569727", { asciinema: true }],
      ["figma", "https://www.figma.com/design/AbC123xyz/Design", { figma: true }],
      ["note", "https://note.com/someone/n/nabcdef123456", { note: true }],
      [
        "googleslides",
        "https://docs.google.com/presentation/d/1AbC_defGHI/edit",
        { googleSlides: true },
      ],
      ["replit", "https://replit.com/@someone/my-repl", { playgrounds: true }],
    ];
    for (const [tag, url, options] of cases) {
      const out = await transformMediaEmbeds(`<${tag} url="${url}"></${tag}>`, options);
      expect(out, `${tag} did not render`).toContain(`ox-provider-card--${tag}`);
    }
  });

  it("leaves a plain <audio> element alone", async () => {
    // The Pascal-only tags must not match their lowercase spelling.
    const html = '<audio src="/a.mp3" controls></audio>';
    const out = await transformMediaEmbeds(html, { audio: true } as MediaEmbedOptions);
    expect(out).toBe(html);
  });

  it("does nothing when the document has no provider tag", async () => {
    const html = "<p>Nothing to see.</p>";
    expect(await transformMediaEmbeds(html, { figma: true } as MediaEmbedOptions)).toBe(html);
  });
});
