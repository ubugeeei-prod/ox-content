//! Same corpus/API on base and head; isolated allocation instrumentation.
#![allow(unsafe_code, clippy::print_stdout, clippy::cast_precision_loss)]
use ox_content_markdown_lint::{
    MarkdownLintOptions, MarkdownLintRuleOptions, MarkdownLinter, lint_markdown_documents,
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Instant,
};

struct CountingAllocator;
static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(size, Ordering::Relaxed);
        }
        unsafe { System.realloc(pointer, layout, size) }
    }
}

fn main() {
    let threads =
        std::env::var("LINT_BENCH_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
    let corpus = std::env::var("LINT_BENCH_CORPUS").unwrap_or_else(|_| "english".into());
    let paragraph = match corpus.as_str() {
        "english" => {
            "This is clear prose with a [visible link](https://example.com) and `code`.\nA second line completes the paragraph.\n\n"
        }
        "japanese" => {
            "この文書では、文章の構造と[設定方法](https://example.com)を説明します。`code` は検査から除外します。\n改行後も同じ段落の文章として扱い、表示される文字を確認します。\n\n"
        }
        _ => panic!("Unknown benchmark corpus: {corpus}"),
    };
    let profile = std::env::var("LINT_BENCH_PROFILE").unwrap_or_else(|_| "legacy".into());
    let paragraph = if profile == "markdownlint" {
        if corpus == "japanese" {
            "この文書では、[設定方法](https://example.com)を説明します。\n`code` は検査から除外し、次の段落まで文章を確認します。\n\n"
        } else {
            "A [visible link](https://example.com) and `code` complete this paragraph.\nA second line contains clear prose.\n\n"
        }
    } else {
        paragraph
    };
    let source = format!("# Guide\n\n{}", paragraph.repeat(120));
    let source = format!("{}\n", source.trim_end_matches('\n'));
    let sources = vec![source; 128];
    let options = MarkdownLintOptions {
        rules: Some(MarkdownLintRuleOptions { spellcheck: Some(false), ..Default::default() }),
        ..Default::default()
    };
    let native = (profile == "markdownlint").then(|| {
        MarkdownLinter::new(Some(
            serde_json::from_value(serde_json::json!({"markdownlint":{}})).unwrap(),
        ))
    });
    let run = || {
        pool.install(|| {
            if let Some(linter) = &native {
                use rayon::prelude::*;
                sources
                    .par_iter()
                    .map(|source| linter.lint_without_mask(black_box(source)))
                    .collect()
            } else {
                lint_markdown_documents(black_box(&sources), Some(options.clone()))
            }
        })
    };
    drop(run()); // warm dictionaries, regex caches, parser, workers
    let mut elapsed = Vec::new();
    for _ in 0..7 {
        let start = Instant::now();
        let results = run();
        let diagnostics: usize = results.iter().map(|v| v.diagnostics.len()).sum();
        assert_eq!(diagnostics, 0);
        black_box(results);
        elapsed.push(start.elapsed().as_secs_f64());
    }
    elapsed.sort_by(f64::total_cmp);
    ALLOCATIONS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    let results = run();
    COUNTING.store(false, Ordering::Relaxed);
    black_box(results);
    let bytes: usize = sources.iter().map(String::len).sum();
    let result = serde_json::json!({ "corpus": corpus, "profile": profile, "threads": threads, "documents": sources.len(), "sourceBytes": bytes,
        "medianMs": elapsed[3] * 1000.0, "mibPerSecond": bytes as f64 / 1_048_576.0 / elapsed[3],
        "allocations": ALLOCATIONS.load(Ordering::Relaxed), "allocatedBytes": BYTES.load(Ordering::Relaxed) });
    println!("{result}");
}
