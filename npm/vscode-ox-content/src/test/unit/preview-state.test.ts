/**
 * Pure-node unit tests for the preview policy extracted from
 * `preview.ts`. The webview lifecycle itself is covered by the Electron
 * integration suite; these pin the title/command decisions that don't
 * need a host.
 */

import { describe, expect, it } from "vitest";

import { SERVER_COMMAND_PREVIEW_HTML, SERVER_COMMAND_PREVIEW_SUBSCRIBE } from "../../constants";
import {
  previewPanelTitle,
  previewSeedCommand,
  pushedPreviewTitle,
} from "../../internal/preview-state";

describe("previewSeedCommand", () => {
  it("subscribes for pushed updates when auto-refresh is on", () => {
    expect(previewSeedCommand(true)).toBe(SERVER_COMMAND_PREVIEW_SUBSCRIBE);
  });

  it("does a one-shot HTML fetch when auto-refresh is off", () => {
    expect(previewSeedCommand(false)).toBe(SERVER_COMMAND_PREVIEW_HTML);
  });
});

describe("previewPanelTitle", () => {
  it("uses the payload title when present", () => {
    expect(previewPanelTitle("Getting Started", "/docs/guide.md")).toBe("Getting Started");
  });

  it("falls back to the basename plus suffix when the payload title is empty", () => {
    expect(previewPanelTitle("", "/docs/guide.md")).toBe("guide.md Preview");
  });

  it("uses Untitled for a buffer with no filename", () => {
    expect(previewPanelTitle("", "")).toBe("Untitled Preview");
  });
});

describe("pushedPreviewTitle", () => {
  it("adopts the pushed title when the server provides one", () => {
    expect(pushedPreviewTitle("New Title", "Old Title")).toBe("New Title");
  });

  it("keeps the current title when the pushed title is empty", () => {
    expect(pushedPreviewTitle("", "Old Title")).toBe("Old Title");
  });
});
