const MAX_MESSAGE = 16 * 1024 * 1024;

/** Byte-oriented LSP framing: chunks may split headers, JSON, and UTF-8 characters. */
export function readMessages(stream, onMessage, onError) {
  let buffer = Buffer.alloc(0);
  let length;
  let failed = false;
  stream.on("data", (chunk) => {
    if (failed) return;
    try {
      buffer = Buffer.concat([buffer, chunk]);
      for (;;) {
        if (length === undefined) {
          const end = buffer.indexOf("\r\n\r\n");
          if (end < 0) {
            if (buffer.length > 8192) throw new Error("LSP header exceeds 8192 bytes");
            break;
          }
          const match = /^Content-Length:\s*(\d+)\s*$/im.exec(buffer.subarray(0, end).toString());
          length = match ? Number(match[1]) : NaN;
          if (!Number.isSafeInteger(length) || length < 1 || length > MAX_MESSAGE)
            throw new Error("Invalid LSP Content-Length");
          buffer = buffer.subarray(end + 4);
        }
        if (buffer.length < length) break;
        const message = JSON.parse(buffer.subarray(0, length).toString("utf8"));
        buffer = buffer.subarray(length);
        length = undefined;
        onMessage(message);
      }
    } catch (error) {
      failed = true;
      onError(error);
    }
  });
  stream.on("end", () => {
    if (!failed && (buffer.length || length !== undefined))
      onError(new Error("Truncated LSP message"));
  });
}

export function writeMessage(stream, message) {
  const body = Buffer.from(JSON.stringify(message));
  stream.write(Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]));
}

function offsetAt(text, position) {
  let offset = 0;
  for (let line = 0; line < position.line; line++) {
    const next = text.indexOf("\n", offset);
    if (next < 0) return text.length;
    offset = next + 1;
  }
  const end = text.indexOf("\n", offset);
  return Math.min(offset + position.character, end < 0 ? text.length : end);
}

export function applyChanges(text, changes) {
  for (const change of changes) {
    text = change.range
      ? text.slice(0, offsetAt(text, change.range.start)) +
        change.text +
        text.slice(offsetAt(text, change.range.end))
      : change.text;
  }
  return text;
}
