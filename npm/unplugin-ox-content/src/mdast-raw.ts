import type { MdastNode, MdastRoot } from "./types";
import {
  parseTransferEnvelope,
  requireTransferSection,
  TRANSFER_PAYLOAD_KIND_MDAST,
} from "./transfer";

const LEGACY_MAGIC = 0x3152444d;
const LEGACY_VERSION = 1;
const LEGACY_HEADER_LEN = 28;
const NODE_RECORD_LEN = 60;
const NONE_U32 = 0xffffffff;

const MDAST_PAYLOAD_VERSION = 1;
const MDAST_SECTION_NODES = 1;
const MDAST_SECTION_CHILD_INDICES = 2;
const MDAST_SECTION_ALIGNS = 3;
const MDAST_SECTION_STRINGS = 4;
const MDAST_SECTION_SOURCE_ORIGIN = 7;

interface SourceOriginPoint {
  byteOffset: number;
  offset: number;
  line: number;
  column: number;
}

const KIND_ROOT = 0;
const KIND_PARAGRAPH = 1;
const KIND_HEADING = 2;
const KIND_THEMATIC_BREAK = 3;
const KIND_BLOCKQUOTE = 4;
const KIND_LIST = 5;
const KIND_LIST_ITEM = 6;
const KIND_CODE = 7;
const KIND_HTML = 8;
const KIND_TABLE = 9;
const KIND_TABLE_ROW = 10;
const KIND_TABLE_CELL = 11;
const KIND_TEXT = 12;
const KIND_EMPHASIS = 13;
const KIND_STRONG = 14;
const KIND_INLINE_CODE = 15;
const KIND_BREAK = 16;
const KIND_LINK = 17;
const KIND_IMAGE = 18;
const KIND_DELETE = 19;
const KIND_FOOTNOTE_REFERENCE = 20;
const KIND_DEFINITION = 21;
const KIND_FOOTNOTE_DEFINITION = 22;

const FLAG_ORDERED = 1 << 0;
const FLAG_SPREAD = 1 << 1;
const FLAG_CHECKED_PRESENT = 1 << 2;
const FLAG_CHECKED_VALUE = 1 << 3;

interface SourceBoundaryPoint {
  byteOffset: number;
  offset: number;
  line: number;
  column: number;
}

interface MdastBufferLayout {
  nodeCount: number;
  childCount: number;
  alignCount: number;
  rootIndex: number;
  nodesOffset: number;
  childIndicesOffset: number;
  alignsOffset: number;
  stringsOffset: number;
  stringsEnd: number;
}

export function deserializeMdastFromRaw(buffer: Uint8Array, source: string): MdastRoot {
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const decoder = new TextDecoder("utf-8");

  const {
    nodeCount,
    childCount,
    alignCount,
    rootIndex,
    nodesOffset,
    childIndicesOffset,
    alignsOffset,
    stringsOffset,
    stringsEnd,
  } = resolveMdastLayout(buffer, view);

  if (stringsEnd > buffer.byteLength) {
    throw new Error("[ox-content] mdast raw transfer buffer is truncated.");
  }

  const sourceIndex = buildSourceIndex(source);
  const sourceOrigin = readSourceOrigin(buffer, view);

  const readString = (offset: number, len: number): string | undefined => {
    if (offset === NONE_U32) {
      return undefined;
    }
    const start = stringsOffset + offset;
    const end = start + len;
    if (end > stringsEnd) {
      throw new Error("[ox-content] mdast raw transfer buffer references an invalid string.");
    }
    return decoder.decode(buffer.subarray(start, end));
  };

  const readPosition = (startOffset: number, endOffset: number) => {
    const start = pointForByteOffset(startOffset, sourceIndex);
    const end = pointForByteOffset(endOffset, sourceIndex);

    if (!sourceOrigin) {
      return { start, end };
    }

    return {
      start: applySourceOrigin(start, sourceOrigin),
      end: applySourceOrigin(end, sourceOrigin),
    };
  };

  const readChildIndex = (index: number): number => {
    if (index >= childCount) {
      throw new Error("[ox-content] mdast raw transfer buffer references an invalid child index.");
    }
    return view.getUint32(childIndicesOffset + index * 4, true);
  };

  const readChildren = (start: number, len: number): MdastNode[] => {
    if (start + len > childCount) {
      throw new Error("[ox-content] mdast raw transfer buffer has an invalid child range.");
    }
    const children: MdastNode[] = [];
    for (let index = 0; index < len; index += 1) {
      children.push(readNode(readChildIndex(start + index)));
    }
    return children;
  };

  const readAlign = (start: number, len: number): Array<"left" | "center" | "right" | null> => {
    if (start + len > alignCount) {
      throw new Error(
        "[ox-content] mdast raw transfer buffer has an invalid table alignment range.",
      );
    }
    const align: Array<"left" | "center" | "right" | null> = [];
    for (let index = 0; index < len; index += 1) {
      const value = buffer[alignsOffset + start + index];
      align.push(value === 1 ? "left" : value === 2 ? "center" : value === 3 ? "right" : null);
    }
    return align;
  };

  const readNode = (index: number): MdastNode => {
    if (index >= nodeCount) {
      throw new Error("[ox-content] mdast raw transfer buffer references an invalid node.");
    }

    const base = nodesOffset + index * NODE_RECORD_LEN;
    const kind = view.getUint8(base);
    const flags = view.getUint8(base + 1);
    const spanStart = view.getUint32(base + 4, true);
    const spanEnd = view.getUint32(base + 8, true);
    const childStart = view.getUint32(base + 12, true);
    const childLen = view.getUint32(base + 16, true);
    const num0 = view.getUint32(base + 20, true);
    const num1 = view.getUint32(base + 24, true);
    const str0 = readString(view.getUint32(base + 28, true), view.getUint32(base + 32, true));
    const str1 = readString(view.getUint32(base + 36, true), view.getUint32(base + 40, true));
    const str2 = readString(view.getUint32(base + 44, true), view.getUint32(base + 48, true));
    const str3 = readString(view.getUint32(base + 52, true), view.getUint32(base + 56, true));
    const children = childLen > 0 ? readChildren(childStart, childLen) : undefined;
    const position = readPosition(spanStart, spanEnd);

    switch (kind) {
      case KIND_ROOT:
        return { type: "root", children: children ?? [], position } as MdastRoot;
      case KIND_PARAGRAPH:
        return { type: "paragraph", children: children ?? [], position };
      case KIND_HEADING:
        return { type: "heading", depth: num0, children: children ?? [], position };
      case KIND_THEMATIC_BREAK:
        return { type: "thematicBreak", position };
      case KIND_BLOCKQUOTE:
        return { type: "blockquote", children: children ?? [], position };
      case KIND_LIST: {
        const node: MdastNode = {
          type: "list",
          ordered: (flags & FLAG_ORDERED) !== 0,
          spread: (flags & FLAG_SPREAD) !== 0,
          children: children ?? [],
          position,
        };
        if (num0 !== NONE_U32) {
          node.start = num0;
        }
        return node;
      }
      case KIND_LIST_ITEM: {
        const node: MdastNode = {
          type: "listItem",
          spread: (flags & FLAG_SPREAD) !== 0,
          children: children ?? [],
          position,
        };
        if ((flags & FLAG_CHECKED_PRESENT) !== 0) {
          node.checked = (flags & FLAG_CHECKED_VALUE) !== 0;
        }
        return node;
      }
      case KIND_CODE: {
        const node: MdastNode = { type: "code", value: str2 ?? "", position };
        if (str0 !== undefined) {
          node.lang = str0;
        }
        if (str1 !== undefined) {
          node.meta = str1;
        }
        return node;
      }
      case KIND_HTML:
        return { type: "html", value: str0 ?? "", position };
      case KIND_TABLE:
        return {
          type: "table",
          align: readAlign(num0, num1),
          children: children ?? [],
          position,
        };
      case KIND_TABLE_ROW:
        return { type: "tableRow", children: children ?? [], position };
      case KIND_TABLE_CELL:
        return { type: "tableCell", children: children ?? [], position };
      case KIND_TEXT:
        return { type: "text", value: str0 ?? "", position };
      case KIND_EMPHASIS:
        return { type: "emphasis", children: children ?? [], position };
      case KIND_STRONG:
        return { type: "strong", children: children ?? [], position };
      case KIND_INLINE_CODE:
        return { type: "inlineCode", value: str0 ?? "", position };
      case KIND_BREAK:
        return { type: "break", position };
      case KIND_LINK: {
        const node: MdastNode = {
          type: "link",
          url: str0 ?? "",
          children: children ?? [],
          position,
        };
        if (str1 !== undefined) {
          node.title = str1;
        }
        return node;
      }
      case KIND_IMAGE: {
        const node: MdastNode = {
          type: "image",
          url: str0 ?? "",
          position,
        };
        if (str1 !== undefined) {
          node.alt = str1;
        }
        if (str2 !== undefined) {
          node.title = str2;
        }
        return node;
      }
      case KIND_DELETE:
        return { type: "delete", children: children ?? [], position };
      case KIND_FOOTNOTE_REFERENCE: {
        const node: MdastNode = {
          type: "footnoteReference",
          identifier: str0 ?? "",
          position,
        };
        if (str1 !== undefined) {
          node.label = str1;
        }
        return node;
      }
      case KIND_DEFINITION: {
        const node: MdastNode = {
          type: "definition",
          identifier: str0 ?? "",
          url: str2 ?? "",
          position,
        };
        if (str1 !== undefined) {
          node.label = str1;
        }
        if (str3 !== undefined) {
          node.title = str3;
        }
        return node;
      }
      case KIND_FOOTNOTE_DEFINITION: {
        const node: MdastNode = {
          type: "footnoteDefinition",
          identifier: str0 ?? "",
          children: children ?? [],
          position,
        };
        if (str1 !== undefined) {
          node.label = str1;
        }
        return node;
      }
      default:
        throw new Error(`[ox-content] Unsupported mdast raw node kind: ${kind}`);
    }
  };

  const root = readNode(rootIndex);
  if (root.type !== "root" || !Array.isArray(root.children)) {
    throw new Error("[ox-content] Native parser returned an invalid mdast root.");
  }

  return root;
}

function readSourceOrigin(buffer: Uint8Array, view: DataView): SourceOriginPoint | undefined {
  const envelope = parseTransferEnvelope(buffer);
  const section = envelope?.sections.get(MDAST_SECTION_SOURCE_ORIGIN);
  if (!section) {
    return undefined;
  }

  if (section.len !== 16) {
    throw new Error("[ox-content] mdast transfer source origin section is misaligned.");
  }

  return {
    byteOffset: view.getUint32(section.offset, true),
    offset: view.getUint32(section.offset + 4, true),
    line: view.getUint32(section.offset + 8, true),
    column: view.getUint32(section.offset + 12, true),
  };
}

function resolveMdastLayout(buffer: Uint8Array, view: DataView): MdastBufferLayout {
  const envelope = parseTransferEnvelope(buffer);
  if (envelope) {
    if (envelope.kind !== TRANSFER_PAYLOAD_KIND_MDAST) {
      throw new Error("[ox-content] Transfer buffer does not contain an mdast payload.");
    }
    if (envelope.payloadVersion !== MDAST_PAYLOAD_VERSION) {
      throw new Error("[ox-content] Unsupported mdast transfer payload version.");
    }

    const nodesSection = requireTransferSection(
      envelope,
      MDAST_SECTION_NODES,
      "[ox-content] mdast transfer is missing the nodes section.",
    );
    const childIndicesSection = requireTransferSection(
      envelope,
      MDAST_SECTION_CHILD_INDICES,
      "[ox-content] mdast transfer is missing the child indices section.",
    );
    const alignsSection = requireTransferSection(
      envelope,
      MDAST_SECTION_ALIGNS,
      "[ox-content] mdast transfer is missing the table alignments section.",
    );
    const stringsSection = requireTransferSection(
      envelope,
      MDAST_SECTION_STRINGS,
      "[ox-content] mdast transfer is missing the string table section.",
    );

    if (nodesSection.len % NODE_RECORD_LEN !== 0) {
      throw new Error("[ox-content] mdast transfer nodes section is misaligned.");
    }
    if (childIndicesSection.len % 4 !== 0) {
      throw new Error("[ox-content] mdast transfer child indices section is misaligned.");
    }

    return {
      nodeCount: nodesSection.len / NODE_RECORD_LEN,
      childCount: childIndicesSection.len / 4,
      alignCount: alignsSection.len,
      rootIndex: envelope.rootHandle,
      nodesOffset: nodesSection.offset,
      childIndicesOffset: childIndicesSection.offset,
      alignsOffset: alignsSection.offset,
      stringsOffset: stringsSection.offset,
      stringsEnd: stringsSection.offset + stringsSection.len,
    };
  }

  const magic = view.getUint32(0, true);
  const version = view.getUint32(4, true);
  if (magic !== LEGACY_MAGIC || version !== LEGACY_VERSION) {
    throw new Error("[ox-content] Invalid mdast raw transfer buffer.");
  }

  const nodeCount = view.getUint32(8, true);
  const childCount = view.getUint32(12, true);
  const alignCount = view.getUint32(16, true);
  const stringBytesLen = view.getUint32(20, true);
  const rootIndex = view.getUint32(24, true);

  const nodesOffset = LEGACY_HEADER_LEN;
  const childIndicesOffset = nodesOffset + nodeCount * NODE_RECORD_LEN;
  const alignsOffset = childIndicesOffset + childCount * 4;
  const stringsOffset = alignsOffset + alignCount;
  const stringsEnd = stringsOffset + stringBytesLen;

  return {
    nodeCount,
    childCount,
    alignCount,
    rootIndex,
    nodesOffset,
    childIndicesOffset,
    alignsOffset,
    stringsOffset,
    stringsEnd,
  };
}

function buildSourceIndex(source: string): SourceBoundaryPoint[] {
  const points: SourceBoundaryPoint[] = [
    {
      byteOffset: 0,
      offset: 0,
      line: 1,
      column: 1,
    },
  ];

  let byteOffset = 0;
  let offset = 0;
  let line = 1;
  let column = 1;

  for (const character of source) {
    byteOffset += utf8ByteLength(character);
    offset += character.length;

    if (character === "\n") {
      line += 1;
      column = 1;
    } else {
      column += 1;
    }

    points.push({
      byteOffset,
      offset,
      line,
      column,
    });
  }

  return points;
}

function pointForByteOffset(byteOffset: number, points: SourceBoundaryPoint[]) {
  const maxByteOffset = points[points.length - 1]?.byteOffset ?? 0;
  const clampedOffset = Math.max(0, Math.min(byteOffset, maxByteOffset));
  let low = 0;
  let high = points.length - 1;

  while (low <= high) {
    const middle = (low + high) >> 1;
    const point = points[middle];
    const nextPoint = points[middle + 1];

    if (clampedOffset < point.byteOffset) {
      high = middle - 1;
    } else if (nextPoint && clampedOffset >= nextPoint.byteOffset) {
      low = middle + 1;
    } else {
      return {
        line: point.line,
        column: point.column,
        offset: point.offset,
      };
    }
  }

  const point = points[points.length - 1] ?? {
    byteOffset: 0,
    offset: 0,
    line: 1,
    column: 1,
  };
  return {
    line: point.line,
    column: point.column,
    offset: point.offset,
  };
}

function applySourceOrigin(
  point: Pick<SourceBoundaryPoint, "line" | "column" | "offset">,
  origin: SourceOriginPoint,
) {
  return {
    line: point.line + origin.line - 1,
    column: point.line === 1 ? point.column + origin.column - 1 : point.column,
    offset: point.offset + origin.offset,
  };
}

function utf8ByteLength(character: string): number {
  const codePoint = character.codePointAt(0);
  if (codePoint === undefined) {
    return 0;
  }
  if (codePoint <= 0x7f) {
    return 1;
  }
  if (codePoint <= 0x7ff) {
    return 2;
  }
  if (codePoint <= 0xffff) {
    return 3;
  }
  return 4;
}
