import { describe, expect, it } from "vite-plus/test";
import { CODE_PLAY_STYLES } from "./styles";

describe("CODE_PLAY_STYLES", () => {
  it("pairs official theme text and background so dark docs stay readable", () => {
    expect(CODE_PLAY_STYLES).toContain("--octc-color-text");
    expect(CODE_PLAY_STYLES).toContain("--octc-color-bg");
    expect(CODE_PLAY_STYLES).toContain("--octc-color-bg-alt");
    expect(CODE_PLAY_STYLES).not.toContain("--octc-surface");
    expect(CODE_PLAY_STYLES).toContain("color: var(--octc-color-text, CanvasText)");
    expect(CODE_PLAY_STYLES).toContain("background: var(--octc-color-bg, Canvas)");
    expect(CODE_PLAY_STYLES).toContain("--octc-color-bg-alt");
  });

  it("does not let docs .content pre styles paint the empty output panel", () => {
    expect(CODE_PLAY_STYLES).toContain(".ox-code-play__panel pre");
    expect(CODE_PLAY_STYLES).toContain("background: transparent !important");
  });

  it("styles status and keyboard focus states", () => {
    expect(CODE_PLAY_STYLES).toContain(".ox-code-play__status");
    expect(CODE_PLAY_STYLES).toContain('[data-ox-run-state="offline"]');
    expect(CODE_PLAY_STYLES).toContain(":focus-visible");
    expect(CODE_PLAY_STYLES).toContain("border-radius: 6px");
  });

  it("keeps chrome density compact for docs examples", () => {
    expect(CODE_PLAY_STYLES).toContain("min-height: 1.9rem");
    expect(CODE_PLAY_STYLES).toContain("grid-template-columns: 5.4rem 1fr");
    expect(CODE_PLAY_STYLES).toContain(".ox-code-play__runtime-chip {\n  display: inline-flex");
  });
});
