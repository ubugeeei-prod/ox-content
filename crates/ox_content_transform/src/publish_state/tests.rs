use rustc_hash::FxHashMap;
use serde_json::{Value, json};

use super::{PublishDecision, PublishStateOptions, classify_publish_state};

fn frontmatter(pairs: &[(&str, Value)]) -> FxHashMap<String, Value> {
    pairs.iter().map(|(key, value)| ((*key).to_string(), value.clone())).collect()
}

fn enabled_at(now: &str) -> PublishStateOptions {
    PublishStateOptions { enabled: true, now: Some(now.to_string()), include_drafts: false }
}

fn classify(pairs: &[(&str, Value)], options: &PublishStateOptions) -> PublishDecision {
    classify_publish_state(&frontmatter(pairs), options)
}

#[test]
fn disabled_publishes_every_page() {
    let off = PublishStateOptions::default();
    assert_eq!(
        classify(&[("draft", json!(true)), ("unlisted", json!(true))], &off),
        PublishDecision { output: true, listed: true }
    );
    assert_eq!(
        classify(&[("scheduled", json!("2099-01-01T00:00:00Z"))], &off),
        PublishDecision { output: true, listed: true }
    );
}

#[test]
fn draft_true_is_unpublished_in_production() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    assert_eq!(
        classify(&[("draft", json!(true))], &options),
        PublishDecision { output: false, listed: false }
    );
    assert_eq!(
        classify(&[("draft", json!(false))], &options),
        PublishDecision { output: true, listed: true }
    );
}

#[test]
fn unlisted_is_built_but_not_listed() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    assert_eq!(
        classify(&[("unlisted", json!(true))], &options),
        PublishDecision { output: true, listed: false }
    );
}

#[test]
fn draft_wins_over_unlisted() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    assert_eq!(
        classify(&[("draft", json!(true)), ("unlisted", json!(true))], &options),
        PublishDecision { output: false, listed: false }
    );
}

#[test]
fn scheduled_and_date_wait_until_now() {
    let options = enabled_at("2026-08-24T12:00:00Z");
    assert_eq!(
        classify(&[("scheduled", json!("2026-08-24T12:00:00Z"))], &options),
        PublishDecision { output: true, listed: true }
    );
    assert_eq!(
        classify(&[("scheduled", json!("2026-08-24T12:00:01Z"))], &options),
        PublishDecision { output: false, listed: false }
    );
    assert_eq!(
        classify(&[("date", json!("2026-08-25"))], &options),
        PublishDecision { output: false, listed: false }
    );
    assert_eq!(
        classify(&[("date", json!("2026-08-24"))], &options),
        PublishDecision { output: true, listed: true }
    );
}

#[test]
fn scheduled_wins_over_date() {
    let options = enabled_at("2026-08-24T12:00:00Z");
    assert_eq!(
        classify(
            &[("scheduled", json!("2026-08-25T00:00:00Z")), ("date", json!("2020-01-01"))],
            &options
        ),
        PublishDecision { output: false, listed: false }
    );
}

#[test]
fn expiry_unpublishes_after_the_instant() {
    let options = enabled_at("2026-08-24T12:00:00Z");
    assert_eq!(
        classify(&[("expiry", json!("2026-08-24T11:59:59Z"))], &options),
        PublishDecision { output: false, listed: false }
    );
    assert_eq!(
        classify(&[("expiry", json!("2026-08-24T12:00:00Z"))], &options),
        PublishDecision { output: true, listed: true }
    );
}

#[test]
fn unlisted_plus_future_schedule_is_unpublished() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    assert_eq!(
        classify(
            &[("unlisted", json!(true)), ("scheduled", json!("2026-09-01T00:00:00Z"))],
            &options
        ),
        PublishDecision { output: false, listed: false }
    );
}

#[test]
fn include_drafts_keeps_wip_pages_in_dev() {
    let options = PublishStateOptions {
        enabled: true,
        now: Some("2026-08-24T00:00:00Z".to_string()),
        include_drafts: true,
    };
    assert_eq!(
        classify(&[("draft", json!(true))], &options),
        PublishDecision { output: true, listed: true }
    );
    assert_eq!(
        classify(&[("scheduled", json!("2099-01-01T00:00:00Z"))], &options),
        PublishDecision { output: true, listed: true }
    );
    assert_eq!(
        classify(&[("unlisted", json!(true))], &options),
        PublishDecision { output: true, listed: false }
    );
}

#[test]
fn invalid_scheduled_and_expiry_are_unpublished() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    assert_eq!(
        classify(&[("scheduled", json!("not-a-date"))], &options),
        PublishDecision { output: false, listed: false }
    );
    assert_eq!(
        classify(&[("expiry", json!("January 2024"))], &options),
        PublishDecision { output: false, listed: false }
    );
}

#[test]
fn invalid_date_is_ignored() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    assert_eq!(
        classify(&[("date", json!("Q1 2024"))], &options),
        PublishDecision { output: true, listed: true }
    );
}

#[test]
fn offsets_are_honored_and_naive_values_are_utc() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    assert_eq!(
        classify(&[("scheduled", json!("2026-08-24T09:00:00+09:00"))], &options),
        PublishDecision { output: true, listed: true }
    );
    assert_eq!(
        classify(&[("scheduled", json!("2026-08-24T09:00:01+09:00"))], &options),
        PublishDecision { output: false, listed: false }
    );
    assert_eq!(
        classify(&[("scheduled", json!("2026-08-24T00:00:01"))], &options),
        PublishDecision { output: false, listed: false }
    );
}

#[test]
fn hostile_frontmatter_does_not_panic() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    let cases = [
        vec![("draft", json!("true"))],
        vec![("draft", json!(1))],
        vec![("draft", json!(["yes"]))],
        vec![("unlisted", json!({"$ne": null}))],
        vec![("scheduled", json!("<script>alert(1)</script>"))],
        vec![("date", json!({"toString": "2099-01-01"}))],
        vec![("expiry", json!([2026, 1, 1]))],
        vec![("scheduled", json!(true))],
        vec![("title", json!("</loc><script>alert(1)</script>"))],
    ];
    for pairs in cases {
        let decision = classify(&pairs, &options);
        assert!(
            decision.output || !decision.listed,
            "hostile input must stay defined: {pairs:?} -> {decision:?}"
        );
    }
}

#[test]
fn only_json_true_counts_as_draft_or_unlisted() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    assert_eq!(
        classify(&[("draft", json!("yes"))], &options),
        PublishDecision { output: true, listed: true }
    );
    assert_eq!(
        classify(&[("unlisted", json!("true"))], &options),
        PublishDecision { output: true, listed: true }
    );
}

#[test]
fn injected_clock_is_used_instead_of_system_time() {
    let early = enabled_at("2020-01-01T00:00:00Z");
    let late = enabled_at("2030-01-01T00:00:00Z");
    let pairs = [("scheduled", json!("2026-06-01T00:00:00Z"))];
    assert_eq!(classify(&pairs, &early), PublishDecision { output: false, listed: false });
    assert_eq!(classify(&pairs, &late), PublishDecision { output: true, listed: true });
}

#[test]
fn invalid_now_falls_back_to_system_clock() {
    let options = PublishStateOptions {
        enabled: true,
        now: Some("<script>".to_string()),
        include_drafts: false,
    };
    assert_eq!(
        classify(&[("draft", json!(true))], &options),
        PublishDecision { output: false, listed: false }
    );
}

#[test]
fn null_date_fields_are_omitted() {
    let options = enabled_at("2026-08-24T00:00:00Z");
    assert_eq!(
        classify(
            &[("scheduled", Value::Null), ("date", Value::Null), ("expiry", Value::Null)],
            &options
        ),
        PublishDecision { output: true, listed: true }
    );
}
