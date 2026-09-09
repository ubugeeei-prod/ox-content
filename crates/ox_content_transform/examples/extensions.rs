//! Reproducible native extension timings: cargo run --release -p ox_content_transform --example extensions
#![allow(clippy::print_stdout, clippy::print_stderr)]
#[path = "extensions/cases.rs"]
mod extensions;
use ox_content_transform::{
    features::{TransformFeatureOptions, preprocess_markdown},
    transformer::MarkdownTransformer,
};
use serde_json::json;
use std::{hint::black_box, time::Instant};

fn samples(mut run: impl FnMut()) -> Vec<f64> {
    if std::env::var_os("OX_EXTENSION_VERIFY").is_some() {
        return vec![];
    }
    for _ in 0..10 {
        run();
    }
    (0..9)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..10 {
                run();
            }
            start.elapsed().as_secs_f64() * 100.0
        })
        .collect()
}

fn main() {
    let filter = std::env::args().nth(1).unwrap_or_default();
    let absent = extensions::PROSE.repeat(32);
    let mut failed = false;
    for case in extensions::cases() {
        if !case.name.contains(&filter) {
            continue;
        }
        let transformer = MarkdownTransformer::from_options(&case.options);
        let features = TransformFeatureOptions::from_options(&case.options);
        for (mode, source) in [("present", case.source.as_str()), ("absent", absent.as_str())] {
            let result = transformer.transform(source);
            assert!(result.errors.is_empty(), "{}: {:?}", case.name, result.errors);
            if mode == "present" && !result.html.contains(case.marker) {
                eprintln!(
                    "{} missing {}: {}",
                    case.name,
                    case.marker,
                    result.html.chars().take(300).collect::<String>()
                );
                failed = true;
                continue;
            }
            let preprocess_ms = samples(|| {
                black_box(preprocess_markdown(black_box(source), &features));
            });
            let transform_ms = samples(|| {
                black_box(transformer.transform(black_box(source)));
            });
            let fresh_ms = samples(|| {
                black_box(
                    MarkdownTransformer::from_options(black_box(&case.options))
                        .transform(black_box(source)),
                );
            });
            println!(
                "{}",
                json!({"name":case.name,"mode":mode,"bytes":source.len(),"iterations":10,"preprocess_ms":preprocess_ms,"transform_ms":transform_ms,"fresh_ms":fresh_ms,"html":result.html,"frontmatter":result.frontmatter,"toc":toc_json(&result.toc),"imports":result.imports.len(),"exports":result.exports,"components":result.components})
            );
        }
    }
    assert!(!failed, "Invalid extension fixtures");
}

fn toc_json(entries: &[ox_content_transform::TocEntry]) -> serde_json::Value {
    json!(entries.iter().map(|entry| json!({"depth":entry.depth,"text":entry.text,"slug":entry.slug,"children":toc_json(&entry.children)})).collect::<Vec<_>>())
}
