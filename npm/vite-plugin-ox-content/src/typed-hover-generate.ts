import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
export interface TypedHoverRange {
  start: number;
  end: number;
  type: string;
}

export interface TypedHoverAttachment {
  code: string;
  hovers: TypedHoverRange[];
}

type TsgoApi = {
  API: new (options?: { cwd?: string; tsserverPath?: string }) => {
    updateSnapshot: (params: { openFiles?: string[] }) => {
      getDefaultProjectForFile: (file: string) => TsgoProject | undefined;
    };
    close: () => void;
  };
};

type TsgoProject = {
  checker: {
    getTypeAtPosition: (file: string, position: number) => TsgoType | undefined;
    getBaseTypeOfLiteralType: (type: TsgoType) => TsgoType | undefined;
    typeToString: (type: TsgoType) => string;
  };
};

type TsgoType = {
  isErrorType?: () => boolean;
};

async function loadTsgoApi(): Promise<TsgoApi | undefined> {
  try {
    const specifier = "@typescript/native-preview/unstable/sync";
    return (await import(specifier)) as TsgoApi;
  } catch {
    return undefined;
  }
}

export async function generateTypedHoverAttachments(
  fences: readonly { language: string; code: string }[],
  tsgoCommand?: string,
): Promise<TypedHoverAttachment[]> {
  if (fences.length === 0) {
    return [];
  }

  const apiMod = await loadTsgoApi();
  if (!apiMod) {
    return fences.map((fence) => ({ code: fence.code, hovers: [] }));
  }

  const temp = await mkdtemp(join(tmpdir(), "ox-content-typed-hover-"));
  const api = new apiMod.API({
    cwd: temp,
    ...(tsgoCommand ? { tsserverPath: tsgoCommand } : {}),
  });
  try {
    const files = await Promise.all(
      fences.map(async (fence, index) => {
        const extension = fence.language.toLowerCase() === "tsx" ? "tsx" : "ts";
        const file = join(temp, `snippet-${index}.${extension}`);
        await writeFile(file, fence.code);
        return { fence, file };
      }),
    );
    const snapshot = api.updateSnapshot({ openFiles: files.map((item) => item.file) });
    return files.map(({ fence, file }) => {
      const project = snapshot.getDefaultProjectForFile(file);
      if (!project) {
        return { code: fence.code, hovers: [] };
      }
      const hovers: TypedHoverRange[] = [];
      for (const ident of collectIdentifierRanges(fence.code)) {
        const type = project.checker.getTypeAtPosition(file, ident.start);
        if (!type || type.isErrorType?.()) {
          continue;
        }
        const widened = project.checker.getBaseTypeOfLiteralType(type) ?? type;
        const text = project.checker.typeToString(widened);
        if (text) {
          hovers.push({ start: ident.start, end: ident.end, type: text });
        }
      }
      return { code: fence.code, hovers };
    });
  } finally {
    api.close();
    await rm(temp, { recursive: true, force: true });
  }
}

const IDENTIFIER_KEYWORDS = new Set([
  "abstract",
  "any",
  "as",
  "asserts",
  "async",
  "await",
  "bigint",
  "boolean",
  "break",
  "case",
  "catch",
  "class",
  "const",
  "continue",
  "debugger",
  "declare",
  "default",
  "delete",
  "do",
  "else",
  "enum",
  "export",
  "extends",
  "false",
  "finally",
  "for",
  "from",
  "function",
  "if",
  "implements",
  "import",
  "in",
  "infer",
  "instanceof",
  "interface",
  "is",
  "keyof",
  "let",
  "never",
  "new",
  "null",
  "number",
  "object",
  "of",
  "package",
  "private",
  "protected",
  "public",
  "readonly",
  "return",
  "satisfies",
  "static",
  "string",
  "super",
  "switch",
  "symbol",
  "this",
  "throw",
  "true",
  "try",
  "type",
  "typeof",
  "undefined",
  "unique",
  "unknown",
  "using",
  "var",
  "void",
  "while",
  "with",
  "yield",
]);

export function collectIdentifierRanges(code: string): Array<{ start: number; end: number }> {
  const ranges: Array<{ start: number; end: number }> = [];
  let index = 0;
  while (index < code.length) {
    const char = code[index]!;
    if (char === "/" && code[index + 1] === "/") {
      index = code.indexOf("\n", index);
      if (index === -1) {
        break;
      }
      continue;
    }
    if (char === "/" && code[index + 1] === "*") {
      const close = code.indexOf("*/", index + 2);
      index = close === -1 ? code.length : close + 2;
      continue;
    }
    if (char === '"' || char === "'" || char === "`") {
      index = skipQuoted(code, index, char);
      continue;
    }
    if (/[A-Za-z_$]/.test(char)) {
      const start = index;
      index += 1;
      while (index < code.length && /[\w$]/.test(code[index]!)) {
        index += 1;
      }
      const name = code.slice(start, index);
      if (!IDENTIFIER_KEYWORDS.has(name)) {
        ranges.push({ start, end: index });
      }
      continue;
    }
    index += 1;
  }
  return ranges;
}

function skipQuoted(code: string, start: number, quote: string): number {
  let index = start + 1;
  while (index < code.length) {
    if (code[index] === "\\") {
      index += 2;
      continue;
    }
    if (code[index] === quote) {
      return index + 1;
    }
    index += 1;
  }
  return code.length;
}
