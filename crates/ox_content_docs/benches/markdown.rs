//! Benchmarks for generated API Markdown rendering.

#![deny(clippy::disallowed_macros)]

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use ox_content_docs::{
    generate_markdown, ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc,
    ApiReturnDoc, ApiTypeParamDoc, MarkdownDocsOptions, MarkdownPathStrategy, MarkdownRenderStyle,
};
use std::hint::black_box;

fn push_usize(output: &mut String, value: usize) {
    let mut buffer = [0_u8; 20];
    let mut cursor = buffer.len();
    let mut rest = value;

    loop {
        cursor -= 1;
        buffer[cursor] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }

    let digits = std::str::from_utf8(&buffer[cursor..]).expect("digits are valid utf-8");
    output.push_str(digits);
}

fn option_name(index: usize) -> String {
    let mut out = String::with_capacity("option".len() + 20);
    out.push_str("option");
    push_usize(&mut out, index);
    out
}

fn default_value(index: usize) -> String {
    let mut out = String::with_capacity("default".len() + 20);
    out.push_str("default");
    push_usize(&mut out, index);
    out
}

fn entry_name(module: usize, index: usize) -> String {
    let mut out = String::with_capacity("Entry".len() + 41);
    out.push_str("Entry");
    push_usize(&mut out, module);
    out.push('_');
    push_usize(&mut out, index);
    out
}

fn module_file(module: usize) -> String {
    let mut out = String::with_capacity("/repo/packages/module/src/index.ts".len() + 20);
    out.push_str("/repo/packages/module");
    push_usize(&mut out, module);
    out.push_str("/src/index.ts");
    out
}

fn param(module: usize, entry: usize, index: usize) -> ApiParamDoc {
    let mut type_annotation = String::with_capacity("OptionConfig<, >".len() + 40);
    type_annotation.push_str("OptionConfig<");
    push_usize(&mut type_annotation, module);
    type_annotation.push_str(", ");
    push_usize(&mut type_annotation, entry);
    type_annotation.push('>');

    let mut description =
        String::with_capacity("Configuration value  for entry . See [SharedConfig].".len() + 60);
    description.push_str("Configuration value ");
    push_usize(&mut description, index);
    description.push_str(" for entry ");
    push_usize(&mut description, entry);
    description.push_str(". See [SharedConfig");
    push_usize(&mut description, module);
    description.push_str("].");

    ApiParamDoc {
        name: option_name(index),
        type_annotation,
        description,
        optional: index % 2 == 0,
        default_value: (index % 2 == 0).then(|| default_value(index)),
    }
}

fn member(module: usize, entry: usize, index: usize) -> ApiDocMember {
    let kind = if index % 5 == 0 {
        "method"
    } else if index % 7 == 0 {
        "getter"
    } else {
        "property"
    };

    let mut name = String::with_capacity("member".len() + 20);
    name.push_str("member");
    push_usize(&mut name, index);

    let mut description = String::with_capacity(
        "Member  on entry . Links to [SharedConfig] and `member` metadata.".len() + 80,
    );
    description.push_str("Member ");
    push_usize(&mut description, index);
    description.push_str(" on entry ");
    push_usize(&mut description, entry);
    description.push_str(". Links to [SharedConfig");
    push_usize(&mut description, module);
    description.push_str("] and `member");
    push_usize(&mut description, index);
    description.push_str("` metadata.");

    let signature = if kind != "property" {
        let mut signature = String::with_capacity("member(value: string): boolean".len() + 20);
        signature.push_str("member");
        push_usize(&mut signature, index);
        signature.push_str("(value: string): boolean");
        Some(signature)
    } else {
        None
    };

    ApiDocMember {
        name,
        kind: kind.to_string(),
        description,
        signature,
        type_annotation: (kind == "property").then(|| "Record<string, unknown>".to_string()),
        params: if kind != "property" { vec![param(module, entry, index)] } else { Vec::new() },
        returns: (kind != "property").then(|| ApiReturnDoc {
            type_annotation: "boolean".to_string(),
            description: "Whether the member accepted the value.".to_string(),
        }),
        optional: index % 2 == 0,
        readonly: index % 3 == 0,
        r#static: index % 11 == 0,
        private: false,
        tags: vec![],
        line: index as u32 + 10,
        end_line: index as u32 + 10,
    }
}

fn entry(module: usize, index: usize) -> ApiDocEntry {
    let kind = match index % 4 {
        0 => "function",
        1 => "interface",
        2 => "class",
        _ => "type",
    };
    let name = entry_name(module, index);
    let file = module_file(module);

    let mut description = String::with_capacity(
        "Entry  in module . This paragraph references [SharedConfig] and {@linkcode  | the current symbol} for link rewriting.".len()
            + name.len()
            + 60,
    );
    description.push_str("Entry ");
    push_usize(&mut description, index);
    description.push_str(" in module ");
    push_usize(&mut description, module);
    description.push_str(". This paragraph references [SharedConfig");
    push_usize(&mut description, module);
    description.push_str("] and {@linkcode ");
    description.push_str(&name);
    description.push_str(" | the current symbol} for link rewriting.");

    ApiDocEntry {
        name: name.clone(),
        kind: kind.to_string(),
        description,
        params: if kind == "function" {
            (0..4).map(|i| param(module, index, i)).collect()
        } else {
            Vec::new()
        },
        returns: (kind == "function").then(|| ApiReturnDoc {
            type_annotation: {
                let mut type_annotation = String::with_capacity("Promise<Result_>".len() + 40);
                type_annotation.push_str("Promise<Result");
                push_usize(&mut type_annotation, module);
                type_annotation.push('_');
                push_usize(&mut type_annotation, index);
                type_annotation.push('>');
                type_annotation
            },
            description: "A processed result object.".to_string(),
        }),
        examples: vec![{
            let mut example = String::with_capacity(name.len() + 80);
            example.push_str("```ts\nconst result = await ");
            example.push_str(&name);
            example.push_str("({ enabled: true });\nconsole.log(result);\n```");
            example
        }],
        tags: vec![ApiDocTag { tag: "since".to_string(), value: "2.0.0".to_string() }],
        private: false,
        file,
        line: index as u32 * 8 + 1,
        end_line: index as u32 * 8 + 6,
        signature: Some(match kind {
            "function" => {
                let mut signature = String::with_capacity(name.len() + 74);
                signature.push_str("export function ");
                signature.push_str(&name);
                signature.push_str("<T extends object>(options: OptionConfig): Promise<T>");
                signature
            }
            "interface" => {
                let mut signature = String::with_capacity(name.len() + 37);
                signature.push_str("export interface ");
                signature.push_str(&name);
                signature.push_str("<T extends object>");
                signature
            }
            "class" => {
                let mut signature = String::with_capacity(name.len() + 33);
                signature.push_str("export class ");
                signature.push_str(&name);
                signature.push_str("<T extends object>");
                signature
            }
            _ => {
                let mut signature = String::with_capacity(name.len() + 60);
                signature.push_str("export type ");
                signature.push_str(&name);
                signature.push_str("<T extends object> = Record<string, T>");
                signature
            }
        }),
        members: if kind != "function" {
            (0..16).map(|i| member(module, index, i)).collect()
        } else {
            Vec::new()
        },
        type_parameters: vec![ApiTypeParamDoc {
            name: "T".to_string(),
            constraint: Some("object".to_string()),
            default: Some("Record<string, unknown>".to_string()),
            description: "Payload shape.".to_string(),
        }],
    }
}

fn docs(module_count: usize, entries_per_module: usize) -> Vec<ApiDocModule> {
    (0..module_count)
        .map(|module| {
            let file = module_file(module);
            let mut description = String::with_capacity(
                "Public API for module . Use {@link SharedConfig} to share settings.".len() + 40,
            );
            description.push_str("Public API for module ");
            push_usize(&mut description, module);
            description.push_str(". Use {@link SharedConfig");
            push_usize(&mut description, module);
            description.push_str("} to share settings.");
            ApiDocModule {
                file: file.clone(),
                description,
                source_path: file,
                examples: vec![],
                tags: vec![],
                entries: (0..entries_per_module).map(|index| entry(module, index)).collect(),
            }
        })
        .collect()
}

fn bench_markdown_generate(c: &mut Criterion) {
    let docs = docs(8, 40);
    let mut group = c.benchmark_group("docs_markdown_generate");
    group.throughput(Throughput::Elements(docs.iter().map(|doc| doc.entries.len() as u64).sum()));
    group.sample_size(10);

    group.bench_function("html_flat", |b| {
        let options = MarkdownDocsOptions::default();
        b.iter(|| black_box(generate_markdown(black_box(&docs), black_box(&options))));
    });

    group.bench_function("markdown_flat", |b| {
        let options = MarkdownDocsOptions {
            render_style: MarkdownRenderStyle::Markdown,
            ..MarkdownDocsOptions::default()
        };
        b.iter(|| black_box(generate_markdown(black_box(&docs), black_box(&options))));
    });

    group.bench_function("markdown_typedoc", |b| {
        let options = MarkdownDocsOptions {
            render_style: MarkdownRenderStyle::Markdown,
            path_strategy: MarkdownPathStrategy::TypeDoc,
            ..MarkdownDocsOptions::default()
        };
        b.iter(|| black_box(generate_markdown(black_box(&docs), black_box(&options))));
    });

    group.bench_function("markdown_category", |b| {
        let options = MarkdownDocsOptions {
            group_by: "category".to_string(),
            render_style: MarkdownRenderStyle::Markdown,
            ..MarkdownDocsOptions::default()
        };
        b.iter(|| black_box(generate_markdown(black_box(&docs), black_box(&options))));
    });

    group.finish();
}

criterion_group!(benches, bench_markdown_generate);
criterion_main!(benches);
