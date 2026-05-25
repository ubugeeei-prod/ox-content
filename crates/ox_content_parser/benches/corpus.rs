//! Real-world Markdown corpus benchmarks.
//!
//! Walks `<repo-root>/benchmarks/corpus/<project>/` for `.md` files and runs
//! parse + parse-then-render against each project as one Criterion benchmark
//! group. The corpus is populated by `node scripts/fetch-bench-corpus.mjs`
//! and is `.gitignore`'d, so this benchmark gracefully degrades when the
//! corpus is missing — it prints a hint and runs no measurements rather
//! than failing the build.

use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

fn corpus_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is `crates/ox_content_parser` — climb to the
    // workspace root and join `benchmarks/corpus`.
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(2)
        .map(|root| root.join("benchmarks").join("corpus"))
        .expect("workspace root should be two levels above ox_content_parser")
}

#[derive(Debug)]
struct CorpusProject {
    name: String,
    concatenated: String,
}

fn load_corpus(root: &Path) -> Vec<CorpusProject> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };

    let mut projects = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }

        let mut concatenated = String::new();
        collect_markdown(&path, &mut concatenated);
        if !concatenated.is_empty() {
            projects.push(CorpusProject { name: name.to_string(), concatenated });
        }
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    projects
}

fn collect_markdown(dir: &Path, out: &mut String) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip nested `.git` directories from sparse-checkout setup.
            if path.file_name().and_then(|name| name.to_str()) == Some(".git") {
                continue;
            }
            collect_markdown(&path, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        if let Ok(contents) = fs::read_to_string(&path) {
            out.push_str(&contents);
            out.push('\n');
        }
    }
}

fn bench_corpus_parse(c: &mut Criterion) {
    let root = corpus_root();
    let projects = load_corpus(&root);
    if projects.is_empty() {
        // Surface a hint when the corpus is missing so users know how to
        // populate it. The workspace lints `print_stderr` as warn, so
        // this is explicitly allowed for the bench-only path.
        #[allow(clippy::print_stderr)]
        {
            eprintln!(
                "corpus benchmark: no markdown found under {}. \
                 Run `node scripts/fetch-bench-corpus.mjs` to populate it.",
                root.display()
            );
        }
        return;
    }

    let mut group = c.benchmark_group("corpus_parse");
    for project in &projects {
        group.throughput(Throughput::Bytes(project.concatenated.len() as u64));
        group.bench_function(&project.name, |b| {
            b.iter(|| {
                let allocator = Allocator::new();
                let _ = Parser::with_options(
                    &allocator,
                    black_box(project.concatenated.as_str()),
                    ParserOptions::gfm(),
                )
                .parse();
            });
        });
    }
    group.finish();
}

fn bench_corpus_parse_render(c: &mut Criterion) {
    let root = corpus_root();
    let projects = load_corpus(&root);
    if projects.is_empty() {
        return;
    }

    let mut group = c.benchmark_group("corpus_parse_render");
    for project in &projects {
        group.throughput(Throughput::Bytes(project.concatenated.len() as u64));
        group.bench_function(&project.name, |b| {
            b.iter(|| {
                let allocator = Allocator::new();
                let doc = Parser::with_options(
                    &allocator,
                    black_box(project.concatenated.as_str()),
                    ParserOptions::gfm(),
                )
                .parse()
                .unwrap();
                let mut renderer = HtmlRenderer::new();
                let _ = renderer.render(&doc);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_corpus_parse, bench_corpus_parse_render);
criterion_main!(benches);
