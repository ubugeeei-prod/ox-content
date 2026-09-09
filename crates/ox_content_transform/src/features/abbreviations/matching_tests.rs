use crate::{AbbreviationsOptions, TransformOptions, transformer::MarkdownTransformer};
use rustc_hash::FxHashMap;

fn options(first: bool) -> TransformOptions {
    TransformOptions {
        abbreviations: Some(AbbreviationsOptions {
            enabled: Some(true),
            first_use_only: Some(first),
            terms: Some(FxHashMap::from_iter([
                ("API".into(), "short".into()),
                ("API Gateway".into(), "long".into()),
                ("TERM".into(), "term".into()),
                ("café".into(), "coffee".into()),
                ("総務".into(), "general affairs".into()),
            ])),
        }),
        ..Default::default()
    }
}

#[test]
fn longest_matching_term_keeps_unicode_boundaries() {
    let result = MarkdownTransformer::from_options(&options(false))
        .transform("API Gateway API xAPI café caféé 総務 総務省 日本語APIです。");
    assert!(result.errors.is_empty());
    assert!(result.html.contains("title=\"long\">API Gateway</abbr>"));
    assert_eq!(result.html.matches("title=\"short\"").count(), 2);
    assert_eq!(result.html.matches("title=\"coffee\"").count(), 1);
    assert_eq!(result.html.matches("title=\"general affairs\"").count(), 1);
    assert!(result.html.contains("xAPI"));
    assert!(result.html.contains("caféé"));
    assert!(result.html.contains("総務省"));
}

#[test]
fn protected_ranges_remain_valid_across_many_adjacent_matches() {
    let source = "API TERM API [API](url) TERM API <abbr title=\"x\">API</abbr> TERM API <a href=\"url\">TERM</a> API TERM";
    let result = MarkdownTransformer::from_options(&options(false)).transform(source);
    assert_eq!(result.html.matches("class=\"ox-abbr\"").count(), 9);
    assert!(result.html.contains("<abbr title=\"x\">API</abbr>"));
    assert!(result.html.contains("<a href=\"url\">TERM</a>"));
}

#[test]
fn inline_overrides_and_first_use_state_do_not_leak_between_documents() {
    let transformer = MarkdownTransformer::from_options(&options(true));
    for _ in 0..3 {
        let local = transformer.transform("*[API]: local\n*[NEW]: added\n\nAPI NEW API NEW");
        assert_eq!(local.html.matches("class=\"ox-abbr\"").count(), 2);
        assert!(local.html.contains("title=\"local\">API"));
        assert!(local.html.contains("title=\"added\">NEW"));
        let configured = transformer.transform("API NEW API\n\nTERM TERM");
        assert_eq!(configured.html.matches("class=\"ox-abbr\"").count(), 2);
        assert!(configured.html.contains("title=\"short\">API"));
        assert!(!configured.html.contains("local"));
        assert!(!configured.html.contains("added"));
    }
}

#[test]
fn indexed_candidates_match_exhaustive_longest_match_reference() {
    let entries = ["A", "API", "API Gateway", "A-B", "B", "café", "総務", "🙂", "🙂🙂"]
        .into_iter()
        .map(|term| (term.to_string(), term.to_string()))
        .collect::<Vec<_>>();
    let mut reference = entries.clone();
    reference.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
    let terms = super::terms::Terms::new(entries);
    let parts = ["A", "API Gateway", "A-B", "B", "café", "総務", "🙂🙂", "x", " ", "-", "。"];
    for left in parts {
        for middle in parts {
            for right in parts {
                let source = format!("{left}{middle}{right}");
                for from in source.char_indices().map(|(index, _)| index) {
                    let expected = source
                        .char_indices()
                        .filter(|(index, _)| *index >= from)
                        .find_map(|(start, _)| {
                            if !super::left_boundary(&source, start) {
                                return None;
                            }
                            reference.iter().find_map(|(term, _)| {
                                let end = start + term.len();
                                (source[start..].starts_with(term)
                                    && super::right_boundary(&source, end))
                                .then_some((start, end, term.as_str()))
                            })
                        });
                    let actual = super::next_term(&source, from, &terms, None)
                        .map(|found| (found.start, found.end, found.term));
                    assert_eq!(actual, expected, "{source} at {from}");
                }
            }
        }
    }
}
