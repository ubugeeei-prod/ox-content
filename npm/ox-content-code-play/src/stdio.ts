import type { StdioEvent, StdioStream } from "./types";

export class StdioBuffer {
  readonly events: StdioEvent[] = [];
  private readonly startedAt: number;

  constructor(startedAt = 0) {
    this.startedAt = startedAt;
  }

  push(stream: StdioStream, text: string, timestampMs = elapsed(this.startedAt)): StdioEvent {
    const event: StdioEvent = { stream, text, timestampMs };
    this.events.push(event);
    return event;
  }

  snapshot(): StdioEvent[] {
    return this.events.slice();
  }
}

export function joinStream(events: StdioEvent[], stream: StdioStream): string {
  return events
    .filter((event) => event.stream === stream)
    .map((event) => event.text)
    .join("");
}

export function selectStream(events: StdioEvent[], stream: StdioStream): StdioEvent[] {
  return events.filter((event) => event.stream === stream);
}

export function withStdioText<T extends { stdio: StdioEvent[] }>(
  result: T,
): T & { stdout: string; stderr: string } {
  return {
    ...result,
    stdout: joinStream(result.stdio, "stdout"),
    stderr: joinStream(result.stdio, "stderr"),
  };
}

export function formatConsoleArgs(args: unknown[]): string {
  return `${args.map(formatConsoleArg).join(" ")}\n`;
}

function formatConsoleArg(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "number" || typeof value === "boolean" || typeof value === "bigint") {
    return String(value);
  }
  if (value === undefined) {
    return "undefined";
  }
  if (value === null) {
    return "null";
  }
  try {
    return JSON.stringify(value);
  } catch {
    return Object.prototype.toString.call(value);
  }
}

function elapsed(startedAt: number): number {
  const now = typeof performance !== "undefined" ? performance.now() : Date.now();
  return Math.max(0, now - startedAt);
}
