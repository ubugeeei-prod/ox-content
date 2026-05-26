import { describe, expect, it } from "vitest";

import {
  COMMAND_INSERT_CALLOUT,
  COMMAND_INSERT_CODE_FENCE,
  COMMAND_INSERT_TABLE,
  COMMAND_OPEN_PREVIEW,
} from "../../constants";
import { EDITOR_GUARDED_COMMANDS, requiresMarkdownEditor } from "../../internal/guards";

describe("requiresMarkdownEditor", () => {
  it("flags every LSP-served insert command", () => {
    for (const id of [COMMAND_INSERT_TABLE, COMMAND_INSERT_CODE_FENCE, COMMAND_INSERT_CALLOUT]) {
      expect(requiresMarkdownEditor(id)).toBe(true);
    }
  });

  it("does not flag the webview-only openPreview command — the webview has its own guard", () => {
    // openPreview's no-editor branch is handled inside `preview.openPreview`,
    // not by the middleware, so the middleware should let it through.
    expect(requiresMarkdownEditor(COMMAND_OPEN_PREVIEW)).toBe(false);
  });

  it("does not flag arbitrary unrelated commands", () => {
    expect(requiresMarkdownEditor("workbench.action.closeAllEditors")).toBe(false);
  });

  it("matches the exported set exactly so the docs and middleware never drift", () => {
    expect([...EDITOR_GUARDED_COMMANDS].sort()).toEqual(
      [COMMAND_INSERT_TABLE, COMMAND_INSERT_CODE_FENCE, COMMAND_INSERT_CALLOUT].sort(),
    );
  });
});
