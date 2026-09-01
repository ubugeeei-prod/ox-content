use serde::Serialize;
use std::fmt::Write as _;

pub(crate) const VITE_CONFIG: &str = r#"import { defineConfig } from "vite-plus";
import { defineConfig as definePackConfig } from "vite-plus/pack";

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: definePackConfig({
    entry: ["src/index.ts"],
    format: ["esm", "cjs"],
    dts: true,
    clean: true,
    hash: false,
    deps: {
      neverBundle: ["@ox-content/vite-plugin"],
    },
  }),
});
"#;

pub(crate) fn tsconfig_json() -> TsConfig {
    TsConfig {
        compiler_options: CompilerOptions {
            target: "ES2022",
            module: "ESNext",
            module_resolution: "bundler",
            strict: true,
            es_module_interop: true,
            skip_lib_check: true,
            declaration: true,
            declaration_map: true,
            out_dir: "dist",
            root_dir: "src",
            types: vec!["node"],
        },
        include: vec!["src/**/*.ts"],
        exclude: vec!["node_modules", "dist"],
    }
}

pub(crate) fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string cannot fail")
}

pub(crate) fn string_record(entries: &[(&str, String)], indent: &str) -> String {
    let mut output = String::new();
    for (key, value) in entries {
        writeln!(output, "{indent}{}: {},", json_string(key), json_string(value))
            .expect("writing to a String cannot fail");
    }
    output
}

pub(crate) fn borrowed_string_record(entries: &[(&str, &str)], indent: &str) -> String {
    let mut output = String::new();
    for (key, value) in entries {
        writeln!(output, "{indent}{}: {},", json_string(key), json_string(value))
            .expect("writing to a String cannot fail");
    }
    output
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TsConfig {
    compiler_options: CompilerOptions,
    include: Vec<&'static str>,
    exclude: Vec<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompilerOptions {
    target: &'static str,
    module: &'static str,
    module_resolution: &'static str,
    strict: bool,
    es_module_interop: bool,
    skip_lib_check: bool,
    declaration: bool,
    declaration_map: bool,
    out_dir: &'static str,
    root_dir: &'static str,
    types: Vec<&'static str>,
}
