import { describe, expect, it } from "vite-plus/test";
import { resolveMathOptions } from "./index";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

describe("math options", () => {
  it("resolves math omitted to false and true or object to true", () => {
    expect(resolveMathOptions(undefined).enabled).toBe(false);
    expect(resolveMathOptions(true).enabled).toBe(true);
    expect(resolveMathOptions({}).enabled).toBe(true);
  });

  it("transforms when resolved options omit math", async () => {
    const options = { math: { enabled: false } } as ResolvedOptions;
    delete (options as { math?: unknown }).math;
    const result = await transformMarkdown("$E=mc^2$", "docs/math.md", options);
    expect(result.html).toContain("$E=mc^2$");
    expect(result.html).not.toContain("ox-math");
  });

  it("forwards math: false and math: true to the NAPI transform", async () => {
    const disabled = await transformMarkdown("$E=mc^2$", "docs/math.md", {
      math: { enabled: false },
    } as ResolvedOptions);
    expect(disabled.html).toContain("$E=mc^2$");
    expect(disabled.html).not.toContain("ox-math");

    const enabled = await transformMarkdown("$E=mc^2$", "docs/math.md", {
      math: { enabled: true },
    } as ResolvedOptions);
    expect(enabled.html).toContain('class="ox-math ox-math-inline"');
    expect(enabled.html).toContain("<mtext>E=mc^2</mtext>");
  });
});
