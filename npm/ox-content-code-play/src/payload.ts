import type { PlayPayload } from "./types";

export function encodePayload(payload: PlayPayload): string {
  return bytesToBase64(new TextEncoder().encode(JSON.stringify(payload)));
}

export function decodePayload(value: string): PlayPayload {
  const json = new TextDecoder().decode(base64ToBytes(value));
  const parsed = JSON.parse(json) as PlayPayload;
  if (!parsed || typeof parsed !== "object" || typeof parsed.language !== "string") {
    throw new Error("Invalid Code Play payload.");
  }
  return parsed;
}

function bytesToBase64(bytes: Uint8Array): string {
  if (typeof Buffer !== "undefined") {
    return Buffer.from(bytes).toString("base64");
  }
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

function base64ToBytes(value: string): Uint8Array {
  if (typeof Buffer !== "undefined") {
    return new Uint8Array(Buffer.from(value, "base64"));
  }
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}
