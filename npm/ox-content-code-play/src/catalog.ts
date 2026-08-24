import type { ConfigField, LanguageDefinition } from "./types";

const rustSchema: ConfigField[] = [
  {
    key: "channel",
    label: "Channel",
    type: "select",
    options: [
      { value: "stable", label: "stable" },
      { value: "beta", label: "beta" },
      { value: "nightly", label: "nightly" },
    ],
    default: "stable",
  },
  {
    key: "edition",
    label: "Edition",
    type: "select",
    options: [
      { value: "2018", label: "2018" },
      { value: "2021", label: "2021" },
      { value: "2024", label: "2024" },
    ],
    default: "2024",
  },
  {
    key: "mode",
    label: "Mode",
    type: "select",
    options: [
      { value: "debug", label: "debug" },
      { value: "release", label: "release" },
    ],
    default: "debug",
  },
  {
    key: "crateType",
    label: "Crate type",
    type: "select",
    options: [
      { value: "auto", label: "auto" },
      { value: "bin", label: "bin" },
      { value: "lib", label: "lib" },
    ],
    default: "auto",
  },
];

const tsSchema: ConfigField[] = [
  { key: "strict", label: "Strict", type: "boolean", default: true },
  {
    key: "target",
    label: "Target",
    type: "select",
    options: [
      { value: "ES2020", label: "ES2020" },
      { value: "ES2022", label: "ES2022" },
      { value: "ESNext", label: "ESNext" },
    ],
    default: "ES2022",
  },
  {
    key: "jsx",
    label: "JSX",
    type: "select",
    options: [
      { value: "react-jsx", label: "react-jsx" },
      { value: "preserve", label: "preserve" },
    ],
    default: "react-jsx",
  },
];

function remote(
  id: string,
  name: string,
  aliases: string[],
  pistonLanguage: string,
  extra: ConfigField[] = [],
): LanguageDefinition {
  return {
    id,
    name,
    aliases,
    capabilities: { execute: true, typecheck: false },
    defaultConfig: { version: "*" },
    configSchema: [
      { key: "version", label: "Runtime version", type: "string", default: "*" },
      ...extra,
    ],
    backend: "remote",
    remote: { pistonLanguage },
  };
}

function framework(
  id: "vue" | "react" | "svelte" | "solid",
  name: string,
  aliases: string[],
): LanguageDefinition {
  return {
    id,
    name,
    aliases,
    capabilities: { execute: true, typecheck: false },
    defaultConfig: {},
    configSchema: [],
    backend: "framework",
    framework: id,
  };
}

export const LANGUAGE_CATALOG: LanguageDefinition[] = [
  {
    id: "typescript",
    name: "TypeScript",
    aliases: ["ts", "tsx", "mts", "cts"],
    capabilities: { execute: true, typecheck: true },
    defaultConfig: { strict: true, target: "ES2022", jsx: "react-jsx" },
    configSchema: tsSchema,
    backend: "typescript",
  },
  {
    id: "javascript",
    name: "JavaScript",
    aliases: ["js", "jsx", "mjs", "cjs"],
    capabilities: { execute: true, typecheck: false },
    defaultConfig: {},
    configSchema: [],
    backend: "javascript",
  },
  {
    id: "rust",
    name: "Rust",
    aliases: ["rs"],
    capabilities: { execute: true, typecheck: true },
    defaultConfig: { channel: "stable", edition: "2024", mode: "debug", crateType: "auto" },
    configSchema: rustSchema,
    backend: "rust-playground",
  },
  {
    id: "go",
    name: "Go",
    aliases: ["golang"],
    capabilities: { execute: true, typecheck: true },
    defaultConfig: { withVet: true },
    configSchema: [{ key: "withVet", label: "Run vet", type: "boolean", default: true }],
    backend: "go-playground",
  },
  framework("vue", "Vue", ["vue"]),
  framework("react", "React", ["react"]),
  framework("svelte", "Svelte", ["svelte"]),
  framework("solid", "Solid", ["solid"]),
  remote("python", "Python", ["py"], "python"),
  remote("php", "PHP", [], "php"),
  remote("ruby", "Ruby", ["rb"], "ruby"),
  remote("sh", "sh", ["bash", "shell", "zsh"], "bash", [
    {
      key: "shell",
      label: "Shell",
      type: "select",
      options: [
        { value: "bash", label: "bash" },
        { value: "sh", label: "sh" },
      ],
      default: "bash",
    },
  ]),
  remote("java", "Java", [], "java"),
  remote("swift", "Swift", [], "swift"),
  remote("kotlin", "Kotlin", ["kt"], "kotlin"),
  remote("c", "C", [], "c"),
  remote("cpp", "C++", ["c++", "cc", "cxx"], "c++", [
    {
      key: "std",
      label: "Standard",
      type: "select",
      options: [
        { value: "c++17", label: "C++17" },
        { value: "c++20", label: "C++20" },
        { value: "c++23", label: "C++23" },
      ],
      default: "c++20",
    },
  ]),
  remote("zig", "Zig", [], "zig"),
  remote("haskell", "Haskell", ["hs"], "haskell"),
  remote("ocaml", "OCaml", ["ml"], "ocaml"),
  remote("csharp", "C#", ["cs", "c#"], "csharp"),
  remote("elixir", "Elixir", ["ex"], "elixir"),
  remote("fsharp", "F#", ["fs", "f#"], "fsharp"),
  remote("clojure", "Clojure", ["clj", "cloujure"], "clojure"),
  remote("scheme", "Scheme", ["scm"], "scheme"),
  remote("moonbit", "MoonBit", ["mbt"], "moonbit"),
  remote("lean", "Lean", ["lean4"], "lean"),
  remote("rocq", "Rocq", ["coq"], "coq"),
];

const byId = new Map(LANGUAGE_CATALOG.map((language) => [language.id, language]));
const byAlias = new Map<string, LanguageDefinition>();
for (const language of LANGUAGE_CATALOG) {
  byAlias.set(language.id, language);
  for (const alias of language.aliases) {
    byAlias.set(alias.toLowerCase(), language);
  }
}

export function resolveLanguage(input: string): LanguageDefinition | undefined {
  return byAlias.get(input.trim().toLowerCase()) ?? byId.get(input.trim().toLowerCase());
}

export function listLanguages(): LanguageDefinition[] {
  return LANGUAGE_CATALOG.slice();
}
