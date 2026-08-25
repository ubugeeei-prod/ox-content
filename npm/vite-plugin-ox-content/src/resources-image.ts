/**
 * Build-time PNG/JPEG pixel helpers for page resources.
 *
 * PNG is decoded and re-encoded for resize/crop. JPEG is encode-only so a
 * `format=jpeg` transform can change the container after the pixel pass.
 */

import { deflateSync, inflateSync } from "node:zlib";

export interface RgbaImage {
  width: number;
  height: number;
  data: Uint8Array;
}

const PNG_SIGNATURE = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

export function isPng(buffer: Buffer): boolean {
  return buffer.length >= 8 && PNG_SIGNATURE.equals(buffer.subarray(0, 8));
}

export function pngSize(buffer: Buffer): { width: number; height: number } {
  return {
    width: buffer.readUInt32BE(16),
    height: buffer.readUInt32BE(20),
  };
}

export function decodePng(buffer: Buffer): RgbaImage {
  if (!isPng(buffer)) {
    throw new Error("not a PNG");
  }
  let width = 0;
  let height = 0;
  let bitDepth = 0;
  let colorType = 0;
  const idat: Buffer[] = [];
  let offset = 8;
  while (offset + 12 <= buffer.length) {
    const length = buffer.readUInt32BE(offset);
    const type = buffer.toString("ascii", offset + 4, offset + 8);
    const start = offset + 8;
    const end = start + length;
    if (end + 4 > buffer.length) {
      break;
    }
    const chunk = buffer.subarray(start, end);
    if (type === "IHDR") {
      width = chunk.readUInt32BE(0);
      height = chunk.readUInt32BE(4);
      bitDepth = chunk[8] ?? 0;
      colorType = chunk[9] ?? 0;
    } else if (type === "IDAT") {
      idat.push(Buffer.from(chunk));
    } else if (type === "IEND") {
      break;
    }
    offset = end + 4;
  }
  if (bitDepth !== 8 || (colorType !== 2 && colorType !== 6)) {
    throw new Error("unsupported PNG");
  }
  const channels = colorType === 6 ? 4 : 3;
  const raw = inflateSync(Buffer.concat(idat));
  const stride = width * channels;
  const data = new Uint8Array(width * height * 4);
  let src = 0;
  const prior = new Uint8Array(stride);
  const recon = new Uint8Array(stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[src++] ?? 0;
    for (let x = 0; x < stride; x++) {
      const sample = raw[src++] ?? 0;
      const a = x >= channels ? recon[x - channels]! : 0;
      const b = prior[x] ?? 0;
      const c = x >= channels ? prior[x - channels]! : 0;
      recon[x] = (sample + paethPredict(filter, a, b, c)) & 0xff;
    }
    for (let x = 0; x < width; x++) {
      const i = x * channels;
      const o = (y * width + x) * 4;
      data[o] = recon[i] ?? 0;
      data[o + 1] = recon[i + 1] ?? 0;
      data[o + 2] = recon[i + 2] ?? 0;
      data[o + 3] = channels === 4 ? (recon[i + 3] ?? 255) : 255;
    }
    prior.set(recon);
  }
  return { width, height, data };
}

function paethPredict(filter: number, a: number, b: number, c: number): number {
  switch (filter) {
    case 0:
      return 0;
    case 1:
      return a;
    case 2:
      return b;
    case 3:
      return (a + b) >> 1;
    case 4: {
      const p = a + b - c;
      const pa = Math.abs(p - a);
      const pb = Math.abs(p - b);
      const pc = Math.abs(p - c);
      if (pa <= pb && pa <= pc) return a;
      if (pb <= pc) return b;
      return c;
    }
    default:
      throw new Error("unsupported PNG filter");
  }
}

export function encodePng(image: RgbaImage): Buffer {
  const { width, height, data } = image;
  const raw = Buffer.alloc((width * 4 + 1) * height);
  let offset = 0;
  for (let y = 0; y < height; y++) {
    raw[offset++] = 0;
    raw.set(data.subarray(y * width * 4, (y + 1) * width * 4), offset);
    offset += width * 4;
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  return Buffer.concat([
    PNG_SIGNATURE,
    pngChunk("IHDR", ihdr),
    pngChunk("IDAT", deflateSync(raw)),
    pngChunk("IEND", Buffer.alloc(0)),
  ]);
}

function pngChunk(type: string, data: Buffer): Buffer {
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const chunk = Buffer.alloc(12 + data.length);
  chunk.writeUInt32BE(data.length, 0);
  body.copy(chunk, 4);
  chunk.writeUInt32BE(crc32(body), 8 + data.length);
  return chunk;
}

function crc32(data: Buffer): number {
  let crc = 0xffffffff;
  for (const byte of data) {
    crc ^= byte;
    for (let i = 0; i < 8; i++) {
      crc = crc & 1 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1;
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

export function createRgba(
  width: number,
  height: number,
  pixel: (x: number, y: number) => [number, number, number, number?],
): RgbaImage {
  const data = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const [r, g, b, a = 255] = pixel(x, y);
      const i = (y * width + x) * 4;
      data[i] = r;
      data[i + 1] = g;
      data[i + 2] = b;
      data[i + 3] = a;
    }
  }
  return { width, height, data };
}

export function resizeNearest(image: RgbaImage, width: number, height: number): RgbaImage {
  const data = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) {
    const sy = Math.min(image.height - 1, Math.floor((y * image.height) / height));
    for (let x = 0; x < width; x++) {
      const sx = Math.min(image.width - 1, Math.floor((x * image.width) / width));
      data.set(
        image.data.subarray((sy * image.width + sx) * 4, (sy * image.width + sx) * 4 + 4),
        (y * width + x) * 4,
      );
    }
  }
  return { width, height, data };
}

export function cropImage(
  image: RgbaImage,
  x: number,
  y: number,
  width: number,
  height: number,
): RgbaImage {
  const left = Math.max(0, Math.min(image.width, Math.floor(x)));
  const top = Math.max(0, Math.min(image.height, Math.floor(y)));
  const cropW = Math.max(1, Math.min(image.width - left, Math.floor(width)));
  const cropH = Math.max(1, Math.min(image.height - top, Math.floor(height)));
  const data = new Uint8Array(cropW * cropH * 4);
  for (let row = 0; row < cropH; row++) {
    const src = ((top + row) * image.width + left) * 4;
    data.set(image.data.subarray(src, src + cropW * 4), row * cropW * 4);
  }
  return { width: cropW, height: cropH, data };
}

export function coverCrop(image: RgbaImage, width: number, height: number): RgbaImage {
  const scale = Math.max(width / image.width, height / image.height);
  const scaled = resizeNearest(
    image,
    Math.max(width, Math.round(image.width * scale)),
    Math.max(height, Math.round(image.height * scale)),
  );
  const x = Math.max(0, Math.floor((scaled.width - width) / 2));
  const y = Math.max(0, Math.floor((scaled.height - height) / 2));
  return cropImage(scaled, x, y, width, height);
}

export { encodeJpeg } from "./resources-jpeg";
