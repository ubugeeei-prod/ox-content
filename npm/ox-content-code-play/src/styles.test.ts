import { describe, expect, it } from "vite-plus/test";
import { CODE_PLAY_STYLES } from "./styles";

describe("CODE_PLAY_STYLES", () => {
  it("pairs official theme text and background so dark docs stay readable", () => {
    expect(CODE_PLAY_STYLES).toContain("--octc-color-text");
    expect(CODE_PLAY_STYLES).toContain("--octc-color-bg");
    expect(CODE_PLAY_STYLES).toContain("--octc-color-bg-alt");
    expect(CODE_PLAY_STYLES).not.toContain("--octc-surface");
    expect(CODE_PLAY_STYLES).toContain("color: var(--octc-color-text, CanvasText)");
    expect(CODE_PLAY_STYLES).toContain(
      "background: var(--octc-color-bg-alt, var(--octc-color-bg, Canvas))",
    );
  });

  it("does not let docs .content pre styles paint the empty output panel", () => {
    expect(CODE_PLAY_STYLES).toContain(".ox-code-play__panel pre");
    expect(CODE_PLAY_STYLES).toContain("background: transparent !important");
  });
});
