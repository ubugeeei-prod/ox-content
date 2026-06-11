use std::path::Path;

use rustc_hash::FxHashMap;
use sha2::{Digest, Sha256};

use super::SharedAsset;

#[derive(Debug, Clone, Copy)]
pub(super) enum AssetKind {
    Css,
    Js,
}

impl AssetKind {
    const fn extension(self) -> &'static str {
        match self {
            Self::Css => "css",
            Self::Js => "js",
        }
    }
}

#[derive(Default)]
pub(super) struct AssetCache {
    indexes_by_content: FxHashMap<String, usize>,
    chunks: Vec<SharedAsset>,
}

impl AssetCache {
    /// Returns an existing content-addressed chunk or creates it once.
    ///
    /// Multiple pages often share identical base CSS, plugin CSS, and client
    /// runtime JS. Keying by full content means each unique payload is hashed
    /// and written once, while every page receives the same public URL.
    pub(super) fn get_or_create(
        &mut self,
        kind: AssetKind,
        label: &str,
        content: &str,
        out_dir: &str,
        base: &str,
    ) -> &SharedAsset {
        if let Some(index) = self.indexes_by_content.get(content).copied() {
            return &self.chunks[index];
        }

        let index = self.chunks.len();
        self.chunks.push(create_shared_asset_chunk(kind, label, content, out_dir, base));
        self.indexes_by_content.insert(content.to_string(), index);
        &self.chunks[index]
    }

    pub(super) fn into_chunks(self) -> Vec<SharedAsset> {
        self.chunks
    }
}

fn create_shared_asset_chunk(
    kind: AssetKind,
    label: &str,
    content: &str,
    out_dir: &str,
    base: &str,
) -> SharedAsset {
    let hash = create_content_hash(content);
    let file_name =
        format!("ox-content-{}-{hash}.{}", sanitize_chunk_label(label), kind.extension());
    let output_path =
        Path::new(out_dir).join("assets").join(&file_name).to_string_lossy().to_string();

    SharedAsset {
        output_path,
        public_path: to_public_asset_path(base, &file_name),
        content: content.to_string(),
    }
}

fn create_content_hash(content: &str) -> String {
    // Five SHA-256 bytes are enough for stable cache-busting filenames here:
    // chunks are generated per site build, not used as a security boundary, and
    // the content map above still de-duplicates exact matches before hashing.
    let hash = Sha256::digest(content.as_bytes());
    let mut output = String::with_capacity(10);
    for byte in hash.iter().take(5) {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn sanitize_chunk_label(label: &str) -> String {
    let mut output = String::new();
    let mut pending_dash = false;

    for ch in label.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !output.is_empty() {
                output.push('-');
            }
            output.push(ch);
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }

    if output.is_empty() {
        "asset".to_string()
    } else {
        output
    }
}

fn to_public_asset_path(base: &str, file_name: &str) -> String {
    let normalized_base = if base.ends_with('/') { base.to_string() } else { format!("{base}/") };
    format!("{normalized_base}assets/{file_name}")
}
