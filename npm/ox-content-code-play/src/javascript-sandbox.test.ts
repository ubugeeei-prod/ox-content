import { describe, expect, it } from "vite-plus/test";
import { StdioBuffer } from "./stdio";
import {
  applySandboxStreams,
  buildJavaScriptSandboxDocument,
  embedJson,
  JS_SANDBOX_FLAGS,
} from "./javascript-sandbox";

describe("JavaScript browser sandbox", () => {
  it("uses allow-scripts only and never allow-same-origin", () => {
    expect(JS_SANDBOX_FLAGS).toBe("allow-scripts");
    expect(JS_SANDBOX_FLAGS).not.toMatch(/same-origin|allow-forms|allow-popups/);
  });

  it("embeds sample source as JSON so </script> cannot break srcdoc", () => {
    const code = `</script><script>parent.steal()</script>`;
    const html = buildJavaScriptSandboxDocument(code, "msg-1");
    expect(html).toContain("\\u003c/script>");
    expect(html).not.toContain("</script><script>parent.steal()");
    expect(html).toContain("new Function");
    expect(html).toContain("parent.postMessage");
    expect(html).toContain(embedJson("msg-1"));
  });

  it("escapes raw < in JSON payloads", () => {
    expect(embedJson({ html: "<img>" })).toBe('{"html":"\\u003cimg>"}');
  });

  it("copies sandbox stdout and stderr onto the host stdio buffer", () => {
    const stdio = new StdioBuffer(0);
    applySandboxStreams(stdio, {
      stdout: ["hello\n"],
      stderr: ["warn\n"],
    });
    expect(stdio.snapshot().map((event) => [event.stream, event.text])).toEqual([
      ["stdout", "hello\n"],
      ["stderr", "warn\n"],
    ]);
  });
});
