import { EventEmitter } from "node:events";
import { describe, expect, it } from "vite-plus/test";
import { applyChanges, readMessages, writeMessage } from "../bin/oxct-lsp-protocol.mjs";

const MAX_MESSAGE = 16 * 1024 * 1024;
const frame = (body, header = (length) => `Content-Length: ${length}\r\n\r\n`) =>
  Buffer.concat([Buffer.from(header(Buffer.byteLength(body))), Buffer.from(body)]);
/** Feed chunks synchronously and collect what the reader delivered. */
function read(chunks, { end = false } = {}) {
  const stream = new EventEmitter();
  const messages = [];
  const errors = [];
  readMessages(
    stream,
    (message) => messages.push(message),
    (error) => errors.push(error),
  );
  for (const chunk of chunks) stream.emit("data", Buffer.from(chunk));
  if (end) stream.emit("end");
  return { messages, errors: errors.map((error) => error.message), raw: errors };
}
const edit = (startLine, startCharacter, endLine, endCharacter, text) => ({
  range: {
    start: { line: startLine, character: startCharacter },
    end: { line: endLine, character: endCharacter },
  },
  text,
});

describe("LSP message reader", () => {
  it("delivers every message of a coalesced chunk in order", () => {
    const chunk = Buffer.concat([frame('{"id":1}'), frame('{"id":2}'), frame('{"id":3}')]);
    expect(read([chunk])).toMatchObject({
      messages: [{ id: 1 }, { id: 2 }, { id: 3 }],
      errors: [],
    });
  });

  it("resumes a message whose header and body arrive in separate chunks", () => {
    const first = frame('{"id":1,"text":"日本語"}');
    const second = frame('{"id":2}');
    const bytes = Buffer.concat([first, second]);
    for (const split of [1, 15, first.length - 4, first.length, first.length + 9]) {
      expect(read([bytes.subarray(0, split), bytes.subarray(split)])).toMatchObject({
        messages: [{ id: 1, text: "日本語" }, { id: 2 }],
        errors: [],
      });
    }
  });

  it.each([
    ["a lowercase name", (length) => `content-length: ${length}\r\n\r\n`],
    ["no space after the colon", (length) => `Content-Length:${length}\r\n\r\n`],
    [
      "a preceding Content-Type header",
      (length) =>
        `Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: ${length}\r\n\r\n`,
    ],
    [
      "a following Content-Type header",
      (length) => `Content-Length: ${length}\r\nContent-Type: application/vscode-jsonrpc\r\n\r\n`,
    ],
  ])("accepts a Content-Length header with %s", (_, header) => {
    expect(read([frame('{"ok":true}', header)])).toMatchObject({
      messages: [{ ok: true }],
      errors: [],
    });
  });

  it.each([["0"], ["-5"], ["1.5"], [String(MAX_MESSAGE + 1)], [""], ["0x10"]])(
    "rejects the Content-Length %j",
    (length) => {
      expect(read([`Content-Length: ${length}\r\n\r\n{}`])).toMatchObject({
        messages: [],
        errors: ["Invalid LSP Content-Length"],
      });
    },
  );

  it("accepts the largest allowed Content-Length and keeps waiting for its body", () => {
    expect(read([`Content-Length: ${MAX_MESSAGE}\r\n\r\n{"partial":`])).toMatchObject({
      messages: [],
      errors: [],
    });
  });

  it("bounds an unterminated header at 8192 bytes", () => {
    expect(read([`X: ${"a".repeat(8189)}`])).toMatchObject({ messages: [], errors: [] });
    expect(read([`X: ${"a".repeat(8190)}`])).toMatchObject({
      messages: [],
      errors: ["LSP header exceeds 8192 bytes"],
    });
    expect(read([`X: ${"a".repeat(5000)}`, "b".repeat(5000)]).errors).toEqual([
      "LSP header exceeds 8192 bytes",
    ]);
  });

  it("measures the body in bytes, not UTF-16 code units", () => {
    const body = JSON.stringify({ text: "日本語🦀" });
    expect(read([frame(body)]).messages).toEqual([{ text: "日本語🦀" }]);
    const short = read([`Content-Length: ${body.length}\r\n\r\n${body}`]);
    expect(short.messages).toEqual([]);
    expect(short.raw[0]).toBeInstanceOf(SyntaxError);
  });

  it("reports invalid JSON once and ignores everything after the failure", () => {
    const result = read([frame("{nope"), frame('{"id":2}'), "garbage"], { end: true });
    expect(result.messages).toEqual([]);
    expect(result.raw).toHaveLength(1);
    expect(result.raw[0]).toBeInstanceOf(SyntaxError);
    expect(read(["Bad: header\r\n\r\n", frame('{"id":2}')], { end: true })).toMatchObject({
      messages: [],
      errors: ["Invalid LSP Content-Length"],
    });
  });

  it("keeps the messages delivered before a failure", () => {
    expect(read([frame('{"id":1}'), frame("{nope"), frame('{"id":3}')]).messages).toEqual([
      { id: 1 },
    ]);
  });

  it.each([
    ["a partial header", ["Content-Len"]],
    ["a header without a body", ["Content-Length: 8\r\n\r\n"]],
    ["a partial body", [frame('{"id":1}').subarray(0, 24)]],
    ["bare line feed separators", ['Content-Length: 8\n\n{"id":1}']],
  ])("reports a stream that ends with %s as truncated", (_, chunks) => {
    expect(read(chunks, { end: true })).toMatchObject({
      messages: [],
      errors: ["Truncated LSP message"],
    });
  });

  it("ends cleanly on a message boundary", () => {
    expect(read([frame('{"id":1}')], { end: true })).toMatchObject({
      messages: [{ id: 1 }],
      errors: [],
    });
    expect(read([], { end: true })).toMatchObject({ messages: [], errors: [] });
  });
});

describe("LSP message writer", () => {
  const written = (...messages) => {
    const chunks = [];
    const stream = { write: (chunk) => chunks.push(chunk) };
    for (const message of messages) writeMessage(stream, message);
    return chunks;
  };

  it("writes one chunk per message with a byte-accurate Content-Length", () => {
    const chunks = written({ v: "日本語🦀" }, {});
    expect(chunks.map((chunk) => chunk.toString())).toEqual([
      'Content-Length: 21\r\n\r\n{"v":"日本語🦀"}',
      "Content-Length: 2\r\n\r\n{}",
    ]);
    expect(chunks.every((chunk) => Buffer.isBuffer(chunk))).toBe(true);
  });

  it("round-trips nested messages through the reader", () => {
    const messages = [
      { jsonrpc: "2.0", id: 1, method: "textDocument/hover", params: { position: { line: 0 } } },
      { jsonrpc: "2.0", id: 1, result: { contents: { kind: "plaintext", value: "a\r\n\r\nb" } } },
      { jsonrpc: "2.0", method: "exit" },
    ];
    expect(read(written(...messages)).messages).toEqual(messages);
  });
});

describe("LSP incremental edits", () => {
  it.each([
    ["no changes", "abc", [], "abc"],
    ["a full replacement", "abc", [{ text: "xyz" }], "xyz"],
    [
      "a full replacement followed by a range edit",
      "abc",
      [{ text: "one\ntwo" }, edit(1, 0, 1, 3, "2")],
      "one\n2",
    ],
    ["an insertion at the start", "abc", [edit(0, 0, 0, 0, "X")], "Xabc"],
    ["an insertion at the end", "a\nb", [edit(1, 1, 1, 1, "!")], "a\nb!"],
    ["an insertion into an empty document", "", [edit(0, 0, 0, 0, "new")], "new"],
    ["an insertion on the line after a trailing newline", "a\n", [edit(1, 0, 1, 0, "b")], "a\nb"],
    ["a line past the end as the end of the document", "a\nb", [edit(9, 0, 9, 0, "!")], "a\nb!"],
    [
      "a character past the end as the end of the line",
      "ab\ncd",
      [edit(0, 99, 0, 99, "!")],
      "ab!\ncd",
    ],
    ["a deletion across lines", "one\ntwo\nthree", [edit(0, 2, 2, 2, "")], "onree"],
    ["a line join", "a\nb", [edit(0, 1, 1, 0, "")], "ab"],
    ["a multi-line replacement", "a\nb\nc", [edit(0, 1, 2, 0, "\nX\nY\n")], "a\nX\nY\nc"],
    [
      "edits inside CRLF lines",
      "a\r\nb\r\n",
      [edit(0, 1, 0, 1, "X"), edit(1, 0, 1, 1, "Y")],
      "aX\r\nY\r\n",
    ],
    [
      "later edits against the already edited text",
      "abc",
      [edit(0, 1, 0, 1, "12"), edit(0, 3, 0, 3, "X")],
      "a12Xbc",
    ],
    ["surrogate pairs as two UTF-16 units", "🦀🦀", [edit(0, 2, 0, 4, "x")], "🦀x"],
  ])("applies %s", (_, text, changes, expected) => {
    expect(applyChanges(text, changes)).toBe(expected);
  });

  it("does not mutate the change list or reuse stale offsets", () => {
    const changes = [edit(0, 0, 0, 1, "---"), edit(0, 3, 0, 3, "\ntitle: x")];
    const snapshot = structuredClone(changes);
    expect(applyChanges("-", changes)).toBe("---\ntitle: x");
    expect(applyChanges("-", changes)).toBe("---\ntitle: x");
    expect(changes).toEqual(snapshot);
  });
});
