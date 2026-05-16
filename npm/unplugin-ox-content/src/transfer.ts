export const TRANSFER_MAGIC = 0x5254584f;
export const TRANSFER_VERSION = 1;
export const TRANSFER_HEADER_LEN = 24;
export const TRANSFER_SECTION_RECORD_LEN = 12;

export const TRANSFER_PAYLOAD_KIND_MDAST = 1;
export const TRANSFER_PAYLOAD_KIND_MARKDOWN_IT_TOKENS = 2;
export const TRANSFER_PAYLOAD_KIND_PREPARED_SOURCE = 3;

export interface TransferSection {
  id: number;
  offset: number;
  len: number;
}

export interface TransferEnvelope {
  kind: number;
  payloadVersion: number;
  rootHandle: number;
  sections: Map<number, TransferSection>;
}

const utf8Decoder = new TextDecoder("utf-8");

export function parseTransferEnvelope(buffer: Uint8Array): TransferEnvelope | null {
  if (buffer.byteLength < TRANSFER_HEADER_LEN) {
    return null;
  }

  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const magic = view.getUint32(0, true);
  if (magic !== TRANSFER_MAGIC) {
    return null;
  }

  const version = view.getUint16(4, true);
  if (version !== TRANSFER_VERSION) {
    throw new Error("[ox-content] Unsupported transfer buffer version.");
  }

  const kind = view.getUint16(6, true);
  const payloadVersion = view.getUint32(8, true);
  const sectionCount = view.getUint32(12, true);
  const rootHandle = view.getUint32(16, true);
  const sectionTableEnd = TRANSFER_HEADER_LEN + sectionCount * TRANSFER_SECTION_RECORD_LEN;

  if (sectionTableEnd > buffer.byteLength) {
    throw new Error("[ox-content] Transfer buffer section table is truncated.");
  }

  const sections = new Map<number, TransferSection>();
  for (let index = 0; index < sectionCount; index += 1) {
    const base = TRANSFER_HEADER_LEN + index * TRANSFER_SECTION_RECORD_LEN;
    const id = view.getUint32(base, true);
    const offset = view.getUint32(base + 4, true);
    const len = view.getUint32(base + 8, true);

    if (offset + len > buffer.byteLength) {
      throw new Error("[ox-content] Transfer buffer references an invalid section.");
    }

    sections.set(id, { id, offset, len });
  }

  return {
    kind,
    payloadVersion,
    rootHandle,
    sections,
  };
}

export function requireTransferSection(
  envelope: TransferEnvelope,
  id: number,
  message: string,
): TransferSection {
  const section = envelope.sections.get(id);
  if (!section) {
    throw new Error(message);
  }
  return section;
}

export function sliceTransferSection(buffer: Uint8Array, section: TransferSection): Uint8Array {
  return buffer.subarray(section.offset, section.offset + section.len);
}

export function decodeTransferSectionUtf8(
  buffer: Uint8Array,
  envelope: TransferEnvelope,
  id: number,
  message: string,
): string {
  return utf8Decoder.decode(
    sliceTransferSection(buffer, requireTransferSection(envelope, id, message)),
  );
}
