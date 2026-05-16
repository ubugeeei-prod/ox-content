import type { ThemeRegistration } from "@ox-content/vite-plugin";

export const oxContentHighlightTheme: ThemeRegistration = {
  name: "ox-content-voidzero",
  type: "dark",
  colors: {
    "editor.background": "#0a1020",
    "editor.foreground": "#dbe7ff",
    "editor.lineHighlightBackground": "#111b34",
    "editor.selectionBackground": "#18284b",
  },
  settings: [
    {
      settings: {
        foreground: "#dbe7ff",
        background: "#0a1020",
      },
    },
    {
      scope: ["comment", "comment.block", "comment.line", "punctuation.definition.comment"],
      settings: {
        foreground: "#667a9d",
        fontStyle: "italic",
      },
    },
    {
      scope: [
        "keyword",
        "storage",
        "storage.type",
        "keyword.control",
        "keyword.operator.new",
        "support.type.primitive",
      ],
      settings: {
        foreground: "#8ea4d6",
      },
    },
    {
      scope: [
        "entity.name.function",
        "support.function",
        "variable.function",
        "meta.function-call",
        "entity.name.method",
      ],
      settings: {
        foreground: "#95b6dd",
      },
    },
    {
      scope: [
        "entity.name.type",
        "entity.name.class",
        "support.type",
        "support.class",
        "support.class.component",
        "entity.other.inherited-class",
        "entity.name.tag",
      ],
      settings: {
        foreground: "#afc0e2",
      },
    },
    {
      scope: [
        "variable",
        "variable.parameter",
        "meta.parameter",
        "meta.object-literal.key",
        "variable.other.property",
        "entity.other.attribute-name",
      ],
      settings: {
        foreground: "#dbe7ff",
      },
    },
    {
      scope: ["string", "string.quoted", "punctuation.definition.string", "constant.other.symbol"],
      settings: {
        foreground: "#88bfdc",
      },
    },
    {
      scope: [
        "constant.numeric",
        "constant.language",
        "constant.character",
        "constant.character.escape",
        "constant.escape",
      ],
      settings: {
        foreground: "#a2cae0",
      },
    },
    {
      scope: [
        "keyword.operator",
        "punctuation",
        "meta.brace",
        "punctuation.separator",
        "punctuation.terminator",
      ],
      settings: {
        foreground: "#7387ad",
      },
    },
    {
      scope: ["markup.heading", "markup.bold"],
      settings: {
        foreground: "#b9c9ff",
        fontStyle: "bold",
      },
    },
    {
      scope: ["markup.italic"],
      settings: {
        foreground: "#b9c9ff",
        fontStyle: "italic",
      },
    },
  ],
};
