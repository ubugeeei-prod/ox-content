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

export function encodeJpeg(image: RgbaImage, quality = 80): Buffer {
  const yQuant = scaleQuant(LUM_QUANT, quality);
  const cQuant = scaleQuant(CHR_QUANT, quality);
  const width = image.width;
  const height = image.height;
  const duY = new Int32Array(64);
  const duCb = new Int32Array(64);
  const duCr = new Int32Array(64);
  const bits = new BitWriter();
  let dcY = 0;
  let dcCb = 0;
  let dcCr = 0;
  for (let y = 0; y < height; y += 8) {
    for (let x = 0; x < width; x += 8) {
      sampleBlock(image, x, y, duY, duCb, duCr);
      dcY = encodeBlock(bits, duY, yQuant, dcY, YDC, YAC);
      dcCb = encodeBlock(bits, duCb, cQuant, dcCb, CDC, CAC);
      dcCr = encodeBlock(bits, duCr, cQuant, dcCr, CDC, CAC);
    }
  }
  bits.flush();
  return Buffer.concat([
    jpegHeader(width, height, yQuant, cQuant),
    bits.toBuffer(),
    Buffer.from([0xff, 0xd9]),
  ]);
}

function sampleBlock(
  image: RgbaImage,
  left: number,
  top: number,
  yOut: Int32Array,
  cbOut: Int32Array,
  crOut: Int32Array,
): void {
  for (let j = 0; j < 8; j++) {
    const y = Math.min(image.height - 1, top + j);
    for (let i = 0; i < 8; i++) {
      const x = Math.min(image.width - 1, left + i);
      const p = (y * image.width + x) * 4;
      const r = image.data[p] ?? 0;
      const g = image.data[p + 1] ?? 0;
      const b = image.data[p + 2] ?? 0;
      const idx = j * 8 + i;
      yOut[idx] = ((66 * r + 129 * g + 25 * b + 128) >> 8) - 128;
      cbOut[idx] = (-38 * r - 74 * g + 112 * b + 128) >> 8;
      crOut[idx] = (112 * r - 94 * g - 18 * b + 128) >> 8;
    }
  }
}

function encodeBlock(
  bits: BitWriter,
  block: Int32Array,
  quant: number[],
  lastDc: number,
  dcTable: HuffmanTable,
  acTable: HuffmanTable,
): number {
  const dct = forwardDct(block);
  const zz = new Int32Array(64);
  for (let i = 0; i < 64; i++) {
    zz[i] = Math.round(dct[ZIGZAG[i]!]! / quant[i]!);
  }
  const dc = zz[0] ?? 0;
  writeCoeff(bits, dc - lastDc, dcTable);
  let zeroRun = 0;
  for (let i = 1; i < 64; i++) {
    const value = zz[i] ?? 0;
    if (value === 0) {
      zeroRun++;
      continue;
    }
    while (zeroRun > 15) {
      writeCode(bits, acTable, 0xf0);
      zeroRun -= 16;
    }
    writeCoeff(bits, value, acTable, zeroRun);
    zeroRun = 0;
  }
  if (zeroRun > 0) {
    writeCode(bits, acTable, 0);
  }
  return dc;
}

function writeCoeff(bits: BitWriter, value: number, table: HuffmanTable, run = 0): void {
  const category = bitCategory(value);
  writeCode(bits, table, (run << 4) | category);
  if (category > 0) {
    bits.writeBits(value < 0 ? value + ((1 << category) - 1) : value, category);
  }
}

function writeCode(bits: BitWriter, table: HuffmanTable, symbol: number): void {
  const entry = table.get(symbol);
  if (!entry) {
    throw new Error("missing Huffman code");
  }
  bits.writeBits(entry.code, entry.len);
}

function bitCategory(value: number): number {
  const abs = Math.abs(value);
  if (!Number.isFinite(abs) || abs === 0) {
    return 0;
  }
  return Math.min(11, Math.ceil(Math.log2(abs + 1)));
}

function forwardDct(block: Int32Array): Float64Array {
  const out = new Float64Array(64);
  for (let v = 0; v < 8; v++) {
    for (let u = 0; u < 8; u++) {
      let sum = 0;
      for (let y = 0; y < 8; y++) {
        for (let x = 0; x < 8; x++) {
          sum +=
            (block[y * 8 + x] ?? 0) *
            Math.cos(((2 * x + 1) * u * Math.PI) / 16) *
            Math.cos(((2 * y + 1) * v * Math.PI) / 16);
        }
      }
      const cu = u === 0 ? Math.SQRT1_2 : 1;
      const cv = v === 0 ? Math.SQRT1_2 : 1;
      out[v * 8 + u] = 0.25 * cu * cv * sum;
    }
  }
  return out;
}

function scaleQuant(base: number[], quality: number): number[] {
  const q = Math.max(1, Math.min(100, quality));
  const scale = q < 50 ? Math.floor(5000 / q) : Math.floor(200 - q * 2);
  return base.map((value) => Math.max(1, Math.min(255, Math.floor((value * scale + 50) / 100))));
}

function jpegHeader(width: number, height: number, yQuant: number[], cQuant: number[]): Buffer {
  const chunks = [
    Buffer.from([0xff, 0xd8]),
    jfifApp0(),
    dqt(0, yQuant),
    dqt(1, cQuant),
    sof(width, height),
    dht(0, 0, STD_DC_LUM_NCODES, STD_DC_LUM_VALUES),
    dht(0, 1, STD_DC_CHR_NCODES, STD_DC_CHR_VALUES),
    dht(1, 0, STD_AC_LUM_NCODES, STD_AC_LUM_VALUES),
    dht(1, 1, STD_AC_CHR_NCODES, STD_AC_CHR_VALUES),
    sos(),
  ];
  return Buffer.concat(chunks);
}

function jfifApp0(): Buffer {
  return Buffer.from([
    0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01,
    0x00, 0x00,
  ]);
}

function dqt(id: number, table: number[]): Buffer {
  const out = Buffer.alloc(5 + 64);
  out[0] = 0xff;
  out[1] = 0xdb;
  out.writeUInt16BE(67, 2);
  out[4] = id;
  for (let i = 0; i < 64; i++) {
    out[5 + i] = table[i] ?? 1;
  }
  return out;
}

function sof(width: number, height: number): Buffer {
  const out = Buffer.from([
    0xff, 0xc0, 0x00, 0x11, 0x08, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x11, 0x00, 0x02, 0x11, 0x01,
    0x03, 0x11, 0x01,
  ]);
  out.writeUInt16BE(height, 5);
  out.writeUInt16BE(width, 7);
  return out;
}

function dht(cls: number, id: number, ncodes: number[], values: number[]): Buffer {
  const out = Buffer.alloc(5 + 16 + values.length);
  out[0] = 0xff;
  out[1] = 0xc4;
  out.writeUInt16BE(3 + 16 + values.length, 2);
  out[4] = (cls << 4) | id;
  Buffer.from(ncodes).copy(out, 5);
  Buffer.from(values).copy(out, 21);
  return out;
}

function sos(): Buffer {
  return Buffer.from([
    0xff, 0xda, 0x00, 0x0c, 0x03, 0x01, 0x00, 0x02, 0x11, 0x03, 0x11, 0x00, 0x3f, 0x00,
  ]);
}

class BitWriter {
  private bytes: number[] = [];
  private bits = 0;
  private length = 0;

  writeBits(value: number, count: number): void {
    for (let i = count - 1; i >= 0; i--) {
      this.bits = (this.bits << 1) | ((value >> i) & 1);
      this.length++;
      if (this.length === 8) {
        this.pushByte();
      }
    }
  }

  flush(): void {
    if (this.length > 0) {
      this.bits <<= 8 - this.length;
      this.pushByte();
    }
  }

  toBuffer(): Buffer {
    return Buffer.from(this.bytes);
  }

  private pushByte(): void {
    this.bytes.push(this.bits & 0xff);
    if ((this.bits & 0xff) === 0xff) {
      this.bytes.push(0);
    }
    this.bits = 0;
    this.length = 0;
  }
}

type HuffmanTable = Map<number, { code: number; len: number }>;

function buildHuffman(ncodes: number[], values: number[]): HuffmanTable {
  const table: HuffmanTable = new Map();
  let code = 0;
  let index = 0;
  for (let len = 1; len <= 16; len++) {
    const count = ncodes[len - 1] ?? 0;
    for (let i = 0; i < count; i++) {
      table.set(values[index++] ?? 0, { code, len });
      code++;
    }
    code <<= 1;
  }
  return table;
}

const ZIGZAG = [
  0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
  13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52,
  45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

const LUM_QUANT = [
  16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56,
  14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113,
  92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
];

const CHR_QUANT = [
  17, 18, 24, 47, 99, 99, 99, 99, 18, 21, 26, 66, 99, 99, 99, 99, 24, 26, 56, 99, 99, 99, 99, 99,
  47, 66, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
  99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
];

const STD_DC_LUM_NCODES = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
const STD_DC_LUM_VALUES = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const STD_DC_CHR_NCODES = [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
const STD_DC_CHR_VALUES = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const STD_AC_LUM_NCODES = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 125];
const STD_AC_LUM_VALUES = [
  0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
  0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0,
  0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28,
  0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
  0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
  0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
  0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
  0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5,
  0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2,
  0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
  0xf9, 0xfa,
];
const STD_AC_CHR_NCODES = [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 119];
const STD_AC_CHR_VALUES = [
  0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71,
  0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xa1, 0xb1, 0xc1, 0x09, 0x23, 0x33, 0x52, 0xf0,
  0x15, 0x62, 0x72, 0xd1, 0x0a, 0x16, 0x24, 0x34, 0xe1, 0x25, 0xf1, 0x17, 0x18, 0x19, 0x1a, 0x26,
  0x27, 0x28, 0x29, 0x2a, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
  0x49, 0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
  0x69, 0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
  0x88, 0x89, 0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5,
  0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3,
  0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda,
  0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
  0xf9, 0xfa,
];

const YDC = buildHuffman(STD_DC_LUM_NCODES, STD_DC_LUM_VALUES);
const CDC = buildHuffman(STD_DC_CHR_NCODES, STD_DC_CHR_VALUES);
const YAC = buildHuffman(STD_AC_LUM_NCODES, STD_AC_LUM_VALUES);
const CAC = buildHuffman(STD_AC_CHR_NCODES, STD_AC_CHR_VALUES);
