import { describe, expect, it, vi } from "vite-plus/test";
import { bootCodePlay } from "./boot";

describe("bootCodePlay", () => {
  it("does nothing without a document", () => {
    const hydrate = vi.fn();
    bootCodePlay(hydrate, undefined);
    expect(hydrate).not.toHaveBeenCalled();
  });

  it("hydrates immediately when the document is already ready", () => {
    const hydrate = vi.fn();
    bootCodePlay(hydrate, { readyState: "complete", addEventListener: vi.fn() });
    expect(hydrate).toHaveBeenCalledTimes(1);
  });

  it("waits for DOMContentLoaded while the document is loading", () => {
    const hydrate = vi.fn();
    let listener: (() => void) | undefined;
    const addEventListener = vi.fn((type: string, next: () => void) => {
      expect(type).toBe("DOMContentLoaded");
      listener = next;
    });
    bootCodePlay(hydrate, { readyState: "loading", addEventListener });
    expect(hydrate).not.toHaveBeenCalled();
    expect(addEventListener).toHaveBeenCalledWith("DOMContentLoaded", expect.any(Function), {
      once: true,
    });
    listener?.();
    expect(hydrate).toHaveBeenCalledTimes(1);
  });
});
