use std::path::PathBuf;

use clap::{Parser as ClapParser, Subcommand};

#[derive(ClapParser, Debug)]
#[command(name = "ox-content-profile", about = "Profiling driver for Ox Content")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,

    /// Iterations to warm up before recording (defaults to 5).
    #[arg(long, global = true, default_value_t = 5)]
    pub warmup: usize,

    /// Iterations to record after warmup (defaults to 50).
    #[arg(long, global = true, default_value_t = 50)]
    pub iters: usize,

    /// Emit machine-readable JSON instead of the table view.
    #[arg(long, global = true)]
    pub json: bool,

    /// Use GFM-enabled parser options for the run.
    #[arg(long, global = true)]
    pub gfm: bool,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Profile the parser only (allocator + AST construction).
    Parse {
        /// Markdown file to read. If omitted, the embedded corpus is used.
        file: Option<PathBuf>,
    },
    /// Profile the renderer only. The input is parsed once outside the
    /// measurement loop so the timing reflects the HTML pass in isolation.
    Render { file: Option<PathBuf> },
    /// Profile the full pipeline: allocator + parse + render.
    Pipeline { file: Option<PathBuf> },
    /// Profile JS/TS docs extraction over a directory tree: OXC parse + JSDoc
    /// parse + AST visit + normalize, for every matched source file.
    DocsExtract { dir: PathBuf },
    /// Profile the docs Markdown render path. Extraction runs once outside the
    /// measurement loop so the timing reflects rendering in isolation.
    DocsRender { dir: PathBuf },
    /// Profile the full docs pipeline: extraction + normalize + Markdown render.
    DocsPipeline { dir: PathBuf },
    /// Profile the entry-point docs path (extractDocsFromEntryPoints): builds
    /// the export graph and extracts normalized docs for the given entry files.
    /// This is the path published packages (e.g. gunshi) use, where the export
    /// graph and doc extraction each parse every reachable module.
    DocsEntrypoints { entries: Vec<PathBuf> },
}
