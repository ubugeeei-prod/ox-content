import { PassThrough } from "node:stream";
import { describe, expect, it } from "vite-plus/test";
import { applyChanges, readMessages, writeMessage } from "../bin/oxct-lsp-protocol.mjs";
import { frontmatterCompletion, frontmatterHover } from "../bin/oxct-frontmatter-completion.mjs";

const shape = {
  type: "object",
  required: ["title"],
  properties: {
    title: { type: "string", description: "Display title" },
    draft: { type: "boolean", default: false },
    level: { anyOf: [{ const: "beginner" }, { const: "advanced" }] },
    author: { $ref: "#/$defs/author" },
    contributors: { type: "array", items: { $ref: "#/$defs/author" } },
  },
  $defs: {
    author: {
      type: "object",
      properties: { name: { type: "string" }, role: { enum: ["writer", "reviewer"] } },
    },
  },
};
const complete = (text, line, character) => frontmatterCompletion(shape, text, { line, character });

describe("frontmatter editor protocol", () => {
  it("handles UTF-8 framing split at every byte and multiple messages", () => {
    const stream = new PassThrough(),
      output = new PassThrough(),
      messages = [],
      chunks = [];
    readMessages(
      stream,
      (message) => messages.push(message),
      (error) => {
        throw error;
      },
    );
    output.on("data", (chunk) => chunks.push(chunk));
    writeMessage(output, { id: 1, value: "日本語🦀" });
    writeMessage(output, { id: 2, value: "other" });
    for (const byte of Buffer.concat(chunks)) stream.write(Buffer.from([byte]));
    expect(messages).toEqual([
      { id: 1, value: "日本語🦀" },
      { id: 2, value: "other" },
    ]);
  });

  it.each([
    "Content-Length: 20000000\r\n\r\n",
    "Content-Length: nope\r\n\r\n",
    "Bad: header\r\n\r\n",
  ])("rejects invalid framing %s", (header) => {
    const stream = new PassThrough();
    let error;
    readMessages(
      stream,
      () => {},
      (value) => {
        error = value;
      },
    );
    stream.write(header);
    expect(error).toBeInstanceOf(Error);
  });

  it("applies sequential incremental edits with UTF-16 positions", () => {
    expect(
      applyChanges("日本語🦀\nname: old\n", [
        { range: { start: { line: 0, character: 3 }, end: { line: 0, character: 5 } }, text: "猫" },
        {
          range: { start: { line: 1, character: 6 }, end: { line: 1, character: 9 } },
          text: "new",
        },
      ]),
    ).toBe("日本語猫\nname: new\n");
  });

  it("completes required and optional fields without repeating existing keys", () => {
    const items = complete("---\ndraft: false\nti\n---\n", 2, 2);
    expect(items.map((item) => item.label)).toEqual(["title"]);
    expect(items[0]).toMatchObject({ sortText: "0title", textEdit: { newText: "title: " } });
    const all = complete("---\ndraft: false\n\n---\n", 2, 0);
    expect(all.map((item) => item.label)).not.toContain("draft");
  });

  it("completes booleans, union literals, nested references and sequence items", () => {
    expect(complete("---\nlevel: \n---\n", 1, 7).map((item) => item.label)).toEqual([
      "beginner",
      "advanced",
    ]);
    expect(complete("---\ndraft: \n---\n", 1, 7).map((item) => item.label)).toEqual([
      "false",
      "true",
    ]);
    expect(complete("---\nauthor:\n  role: \n---\n", 2, 8).map((item) => item.label)).toEqual([
      "writer",
      "reviewer",
    ]);
    expect(
      complete("---\ncontributors:\n  - name: Jane\n    ro\n---\n", 3, 6).map((item) => item.label),
    ).toEqual(["role"]);
  });

  it("provides schema descriptions on hover and leaves Markdown body to the native server", () => {
    expect(
      frontmatterHover(shape, "---\ntitle: Hello\n---\n", { line: 1, character: 3 }).contents.value,
    ).toContain("Display title");
    expect(complete("---\ntitle: Hello\n---\n# Body", 3, 2)).toBeUndefined();
  });
});
