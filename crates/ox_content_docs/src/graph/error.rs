use std::path::PathBuf;

use thiserror::Error;

use crate::ExtractError;

/// Export graph error.
#[derive(Debug, Error)]
pub enum GraphError {
    /// IO error while reading a module.
    #[error("failed to read {path}: {source}")]
    Read {
        /// Path that failed to read.
        path: PathBuf,
        /// Source error.
        #[source]
        source: std::io::Error,
    },
    /// Parser error.
    #[error("failed to parse {path}: {message}")]
    Parse {
        /// Path that failed to parse.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// Resolver error.
    #[error("failed to resolve {specifier} from {importer}: {message}")]
    Resolve {
        /// Importer path.
        importer: PathBuf,
        /// Module specifier.
        specifier: String,
        /// Resolver message.
        message: String,
    },
    /// Documentation extraction error.
    #[error("failed to extract docs from {path}: {source}")]
    Extract {
        /// Path that failed to extract.
        path: PathBuf,
        /// Source error.
        #[source]
        source: ExtractError,
    },
}
