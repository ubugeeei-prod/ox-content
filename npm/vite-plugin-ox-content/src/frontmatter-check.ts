import { isNode, LineCounter, parseDocument } from "yaml";
import {
  checkFrontmatterValue,
  selectFrontmatterSchema,
  type FrontmatterSchemas,
  type FrontmatterIssue,
} from "./frontmatter-schemas";

export interface FrontmatterDiagnostic {
  file: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  message: string;
  path: PropertyKey[];
}

/** Validate source YAML through the same asynchronous Standard Schema in every host. */
export async function checkFrontmatter(
  source: string,
  file: string,
  schemas: FrontmatterSchemas | undefined,
  root: string,
) {
  const selected = selectFrontmatterSchema(schemas, file, root);
  if (!selected) return { diagnostics: [] as FrontmatterDiagnostic[], value: undefined };
  const opening = source.match(/^\uFEFF?---[ \t]*\r?\n/);
  const body = opening ? source.slice(opening[0].length) : "";
  const closing = opening ? /^(?:---|\.\.\.)[ \t]*(?:\r?\n|$)/m.exec(body) : undefined;
  const counter = new LineCounter();
  const doc = parseDocument(closing ? body.slice(0, closing.index) : "", {
    lineCounter: counter,
    prettyErrors: false,
  });
  const diagnostic = (issue: FrontmatterIssue, offset = 0): FrontmatterDiagnostic => {
    const node = issue.path.length
      ? doc.getIn(
          issue.path.filter((key) => typeof key !== "symbol"),
          true,
        )
      : undefined;
    if (isNode(node) && node.range) offset = node.range[0];
    const point = counter.linePos(offset);
    const line = opening ? point.line + 1 : 1,
      column = opening ? point.col : 1;
    return { file, line, column, endLine: line, endColumn: column + 1, ...issue };
  };
  if (opening && !closing)
    return {
      diagnostics: [diagnostic({ message: "Unterminated YAML frontmatter", path: [] })],
      value: undefined,
    };
  if (doc.errors.length)
    return {
      diagnostics: doc.errors.map((error) =>
        diagnostic({ message: error.message, path: [] }, error.pos[0]),
      ),
      value: undefined,
    };
  let value: unknown;
  try {
    value = doc.toJS({ maxAliasCount: 100 }) ?? {};
  } catch (error) {
    return { diagnostics: [diagnostic({ message: String(error), path: [] })], value: undefined };
  }
  if (!value || typeof value !== "object" || Array.isArray(value))
    return {
      diagnostics: [diagnostic({ message: "YAML frontmatter must be an object", path: [] })],
      value: undefined,
    };
  const result = await checkFrontmatterValue(selected[1], value);
  return { diagnostics: result.issues.map((issue) => diagnostic(issue)), value: result.value };
}

export function formatFrontmatterDiagnostics(diagnostics: FrontmatterDiagnostic[]): string {
  return diagnostics
    .map(
      (issue) =>
        `${issue.file}:${issue.line}:${issue.column} ${issue.path.length ? `${issue.path.map(String).join(".")}: ` : ""}${issue.message}`,
    )
    .join("\n");
}
