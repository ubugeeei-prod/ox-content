import { parseDocument, stringify } from "yaml";

function shapeAt(schema, path, root = schema, seen = new Set()) {
  if (!schema || typeof schema !== "object") return {};
  if (schema.$ref?.startsWith("#/") && !seen.has(schema.$ref)) {
    seen.add(schema.$ref);
    const target = schema.$ref
      .slice(2)
      .split("/")
      .reduce((node, key) => node?.[key.replaceAll("~1", "/").replaceAll("~0", "~")], root);
    schema = { ...schema, ...shapeAt(target, [], root, seen) };
  }
  const branches = schema.allOf ?? schema.anyOf ?? schema.oneOf ?? [];
  if (branches.length) {
    const shapes = branches.map((branch) => shapeAt(branch, [], root, new Set(seen)));
    schema = {
      ...schema,
      properties: Object.assign({}, schema.properties, ...shapes.map((shape) => shape.properties)),
      required: [
        ...new Set([
          ...(schema.required ?? []),
          ...shapes.flatMap((shape) => shape.required ?? []),
        ]),
      ],
      enum: [
        ...new Set([
          ...(schema.enum ?? []),
          ...shapes.flatMap((shape) => shape.enum ?? ("const" in shape ? [shape.const] : [])),
        ]),
      ],
    };
  }
  if (!path.length) return schema;
  const [key, ...rest] = path;
  return shapeAt(
    typeof key === "number" ? schema.items : schema.properties?.[key],
    rest,
    root,
    seen,
  );
}

/** Locate an indented YAML field, including mappings nested in sequence items. */
export function frontmatterContext(source, position) {
  const lines = source.split("\n").map((line) => line.replace(/\r$/, ""));
  if (!/^\uFEFF?---\s*$/.test(lines[0] ?? "") || position.line < 1) return undefined;
  const end = lines.findIndex((line, i) => i > 0 && /^(---|\.\.\.)\s*$/.test(line));
  if (end >= 0 && position.line >= end) return undefined;
  const line = lines[position.line] ?? "";
  const prefix = line.slice(0, position.character);
  const indent = /^\s*/.exec(prefix)[0].length;
  const stack = [];
  for (let i = 1; i < position.line; i++) {
    const text = lines[i];
    if (!text.trim() || text.trimStart().startsWith("#")) continue;
    const spaces = /^\s*/.exec(text)[0].length;
    while (stack.length && stack.at(-1).indent >= spaces) stack.pop();
    const key = /^\s*([\w.-]+):(?:\s*(?:#.*)?)?$/.exec(text);
    if (key) stack.push({ indent: spaces, key: key[1] });
    else if (/^\s*-\s*/.test(text)) {
      const array = stack.at(-1);
      if (array) stack.push({ indent: spaces, key: 0 });
      const itemKey = /^\s*-\s+([\w.-]+):\s*$/.exec(text);
      if (itemKey) stack.push({ indent: spaces + 2, key: itemKey[1] });
    }
  }
  while (stack.length && stack.at(-1).indent >= indent) stack.pop();
  const path = stack.map((entry) => entry.key);
  const item = /^\s*-\s*/.exec(prefix);
  if (item) path.push(0);
  const contentStart = item ? item[0].length : indent;
  const field = prefix.slice(contentStart);
  const colon = field.indexOf(":");
  const key = (colon < 0 ? field : field.slice(0, colon)).trim().replace(/^['"]|['"]$/g, "");
  const valueStart = contentStart + colon + 1 + /^\s*/.exec(field.slice(colon + 1))[0].length;
  return {
    path,
    key,
    value: colon >= 0,
    start: colon >= 0 ? valueStart : contentStart,
    line: position.line,
    character: position.character,
    yaml: lines.slice(1, end < 0 ? undefined : end).join("\n"),
  };
}

function yamlValue(value) {
  return stringify(value, { lineWidth: 0 }).trimEnd();
}

function detail(schema) {
  const type = Array.isArray(schema.type) ? schema.type.join(" | ") : schema.type;
  return [
    schema.title,
    schema.description,
    type && `Type: ${type}`,
    schema.enum?.length &&
      `Values: ${schema.enum.map((value) => JSON.stringify(value)).join(", ")}`,
  ]
    .filter(Boolean)
    .join("\n\n");
}

export function frontmatterCompletion(schema, source, position) {
  const context = frontmatterContext(source, position);
  if (!context) return undefined;
  const shape = shapeAt(schema, context.path);
  const range = { start: { line: context.line, character: context.start }, end: position };
  if (context.value) {
    const property = shapeAt(schema, [...context.path, context.key]);
    const candidates = property.enum?.length
      ? property.enum
      : "const" in property
        ? [property.const]
        : [
            ...("default" in property ? [property.default] : []),
            ...(property.examples ?? []),
            ...(property.type === "boolean" ? [true, false] : []),
          ];
    const values = [...new Map(candidates.map((value) => [JSON.stringify(value), value])).values()];
    return values.map((value) => ({
      label: yamlValue(value),
      kind: 12,
      documentation: detail(property),
      textEdit: { range, newText: yamlValue(value) },
    }));
  }
  let existing = {};
  try {
    existing = context.path.reduce(
      (value, key) => value?.[key],
      parseDocument(context.yaml).toJS({ maxAliasCount: 100 }),
    );
  } catch {
    /* Incomplete YAML is normal while typing. */
  }
  return Object.entries(shape.properties ?? {})
    .filter(
      ([key]) =>
        key.startsWith(context.key) && (key === context.key || !Object.hasOwn(existing ?? {}, key)),
    )
    .map(([key, value]) => {
      const property = shapeAt(value, [], schema);
      const required = shape.required?.includes(key);
      const initial = "default" in property ? yamlValue(property.default) : "";
      return {
        label: key,
        kind: 10,
        detail: required ? "Required frontmatter field" : "Frontmatter field",
        documentation: detail(property),
        sortText: `${required ? "0" : "1"}${key}`,
        textEdit: { range, newText: `${key}: ${initial}` },
      };
    });
}

export function frontmatterHover(schema, source, position) {
  const line = source.split("\n")[position.line] ?? "";
  const context = frontmatterContext(source, {
    ...position,
    character: Math.max(position.character, line.indexOf(":") + 1),
  });
  if (!context) return undefined;
  const property = shapeAt(schema, [...context.path, context.key]);
  const description = detail(property);
  return description
    ? { contents: { kind: "plaintext", value: `${context.key}\n\n${description}` } }
    : null;
}
