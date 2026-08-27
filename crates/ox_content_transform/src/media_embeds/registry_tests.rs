//! Invariants of the provider table itself.
//!
//! A table trades a dispatch chain for rows, and the hazard it introduces is a
//! mis-wired row — a provider pointing at the wrong option, so it turns on with
//! an unrelated feature or never turns on at all. The compiler cannot catch
//! that, so it is pinned here.

use crate::MediaEmbedsOptions;

use super::registry::PROVIDERS;
use super::transform_media_embeds;

/// Turns one option on, leaving the rest at their defaults.
type SetOption = fn(&mut MediaEmbedsOptions);

/// Every option a caller can set, paired with the name it is known by.
const SETTERS: &[(&str, SetOption)] = &[
    ("spotify", |o| o.spotify = Some(true)),
    ("appleMusic", |o| o.apple_music = Some(true)),
    ("speakerDeck", |o| o.speaker_deck = Some(true)),
    ("audio", |o| o.audio = Some(true)),
    ("video", |o| o.video = Some(true)),
    ("stackBlitz", |o| o.stack_blitz = Some(true)),
    ("twitter", |o| o.twitter = Some(true)),
    ("bluesky", |o| o.bluesky = Some(true)),
    ("googleMaps", |o| o.google_maps = Some(true)),
    ("qiita", |o| o.qiita = Some(true)),
    ("zenn", |o| o.zenn = Some(true)),
    ("packageRegistry", |o| o.package_registry = Some(true)),
    ("playgrounds", |o| o.playgrounds = Some(true)),
    ("vimeo", |o| o.vimeo = Some(true)),
    ("twitch", |o| o.twitch = Some(true)),
    ("discord", |o| o.discord = Some(true)),
    ("fediverse", |o| o.fediverse = Some(true)),
    ("facebook", |o| o.facebook = Some(true)),
    ("threads", |o| o.threads = Some(true)),
    ("instagram", |o| o.instagram = Some(true)),
    ("webContainer", |o| o.web_container = Some(true)),
    ("loom", |o| o.loom = Some(true)),
    ("asciinema", |o| o.asciinema = Some(true)),
    ("figma", |o| o.figma = Some(true)),
    ("note", |o| o.note = Some(true)),
    ("googleSlides", |o| o.google_slides = Some(true)),
];

fn enabled_by(set: SetOption) -> Vec<&'static str> {
    let mut options = MediaEmbedsOptions::default();
    set(&mut options);
    PROVIDERS.iter().filter(|p| (p.enabled)(&options)).map(|p| p.name).collect()
}

#[test]
fn each_option_gates_exactly_its_own_providers() {
    let mut report = String::new();
    for (name, set) in SETTERS {
        report.push_str(name);
        report.push_str(" -> ");
        report.push_str(&enabled_by(*set).join(", "));
        report.push('\n');
    }
    insta::assert_snapshot!(report);
}

#[test]
fn every_provider_is_reachable_from_some_option() {
    for provider in PROVIDERS {
        let reachable = SETTERS.iter().any(|(_, set)| enabled_by(*set).contains(&provider.name));
        assert!(reachable, "{} has no option that turns it on", provider.name);
    }
}

#[test]
fn provider_tags_are_unique() {
    let mut seen: Vec<&str> = Vec::new();
    for provider in PROVIDERS {
        assert!(!seen.contains(&provider.name), "duplicate provider tag {}", provider.name);
        seen.push(provider.name);
    }
}

#[test]
fn nothing_is_enabled_by_default() {
    let options = MediaEmbedsOptions::default();
    for provider in PROVIDERS {
        assert!(!(provider.enabled)(&options), "{} is on by default", provider.name);
    }

    // And the whole transform is a no-op, tags left exactly as authored.
    let source = r#"<Spotify url="https://open.spotify.com/track/abc123"></Spotify>"#;
    assert_eq!(transform_media_embeds(source, Some(&options)), source);
}
