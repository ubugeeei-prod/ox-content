// Test fixtures deliberately use owned formatting outside parser hot paths.
#![allow(clippy::disallowed_macros)]

use super::{escaped_pipe_scan_start, is_escaped_table_pipe, unescape_table_pipes};
use ox_content_allocator::Allocator;

#[test]
fn pipe_candidates_match_scalar_scan_across_length_and_escape_boundaries() {
    for prefix_len in 0..=128 {
        for backslashes in 0..=5 {
            for suffix in ["", "|", " \\| end", " 日本語 \\| 🙂"] {
                let source =
                    format!("{}{}|{suffix}", "x".repeat(prefix_len), "\\".repeat(backslashes));
                let bytes = source.as_bytes();
                let expected = bytes.iter().enumerate().find_map(|(index, &byte)| {
                    (byte == b'|' && is_escaped_table_pipe(bytes, index)).then_some(index)
                });
                let scan_start = escaped_pipe_scan_start(bytes);
                assert_eq!(scan_start.is_some(), expected.is_some(), "{source:?}");
                if let (Some(start), Some(first)) = (scan_start, expected) {
                    assert!(start <= first, "must not skip the first escaped pipe: {source:?}");
                }
            }
        }
        assert_eq!(escaped_pipe_scan_start("x".repeat(prefix_len).as_bytes()), None);
    }
}

#[test]
fn skipped_unescaped_pipes_keep_content_and_source_offsets() {
    for prefix_len in [0, 8, 31, 63, 64, 65, 128] {
        let prefix = "x".repeat(prefix_len);
        let source = format!("{prefix}|日本語\\|end");
        let allocator = Allocator::new();
        let result = unescape_table_pipes(&allocator, &source);
        assert_eq!(result.content, format!("{prefix}|日本語|end"));
        let offsets = result.source_offsets.unwrap();
        assert_eq!(offsets.len(), result.content.len() + 1);
        let removed_at = prefix_len + "|日本語".len();
        for (index, offset) in offsets.iter().enumerate() {
            assert_eq!(*offset as usize, index + usize::from(index > removed_at));
        }
    }
}
