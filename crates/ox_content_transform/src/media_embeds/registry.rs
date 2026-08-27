//! The first-party embed provider table.
//!
//! Every provider is one row: the tag an author writes, how that tag is
//! matched, the option that turns it on, and the renderer. Adding a provider
//! means adding a row — not editing a dispatch chain and a separate
//! "is anything enabled" predicate that has to be kept in step with it.

use crate::MediaEmbedsOptions;

use super::apple_music::render_apple_music;
use super::html::ComponentElement;
use super::native::{render_audio, render_video};
use super::package_cards::{render_crates_io, render_docker_hub, render_npm_package, render_pypi};
use super::playground_cards::{render_codepen, render_jsfiddle, render_observable};
use super::provider_cards::{
    render_discord, render_facebook, render_fediverse, render_google_maps, render_instagram,
    render_mastodon, render_misskey, render_mixi2, render_qiita, render_threads, render_zenn,
};
use super::render::{
    render_bluesky, render_spotify, render_stackblitz, render_tweet, render_webcontainer,
};
use super::speaker_deck::render_speaker_deck;
use super::video_cards::{render_twitch, render_vimeo};

/// How a provider's tag is spelled in the document.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Tag {
    /// Matched case-insensitively, so `<zenn>` and `<Zenn>` both resolve.
    AnyCase,
    /// Matched exactly, reserving the lowercase spelling for the HTML element
    /// of the same name — `<audio>` stays a plain audio element, `<Audio>`
    /// becomes the embed.
    Pascal,
}

/// One first-party embed provider.
pub(super) struct Provider {
    pub(super) name: &'static str,
    pub(super) tag: Tag,
    pub(super) enabled: fn(&MediaEmbedsOptions) -> bool,
    pub(super) render: fn(&ComponentElement<'_>) -> Option<String>,
}

fn on(value: Option<bool>) -> bool {
    value.unwrap_or(false)
}

/// Providers in the order they rewrite a document.
///
/// The order is part of the contract: a document is rewritten once per row, so
/// a renderer only ever sees markup the rows above it have already settled.
pub(super) const PROVIDERS: &[Provider] = &[
    Provider {
        name: "spotify",
        tag: Tag::AnyCase,
        enabled: |o| on(o.spotify),
        render: render_spotify,
    },
    Provider {
        name: "applemusic",
        tag: Tag::AnyCase,
        enabled: |o| on(o.apple_music),
        render: render_apple_music,
    },
    Provider {
        name: "speakerdeck",
        tag: Tag::AnyCase,
        enabled: |o| on(o.speaker_deck),
        render: render_speaker_deck,
    },
    Provider { name: "Audio", tag: Tag::Pascal, enabled: |o| on(o.audio), render: render_audio },
    Provider { name: "Video", tag: Tag::Pascal, enabled: |o| on(o.video), render: render_video },
    Provider {
        name: "stackblitz",
        tag: Tag::AnyCase,
        enabled: |o| on(o.stack_blitz),
        render: render_stackblitz,
    },
    Provider { name: "tweet", tag: Tag::AnyCase, enabled: |o| on(o.twitter), render: render_tweet },
    Provider { name: "xpost", tag: Tag::AnyCase, enabled: |o| on(o.twitter), render: render_tweet },
    Provider {
        name: "bluesky",
        tag: Tag::AnyCase,
        enabled: |o| on(o.bluesky),
        render: render_bluesky,
    },
    Provider {
        name: "googlemaps",
        tag: Tag::AnyCase,
        enabled: |o| on(o.google_maps),
        render: render_google_maps,
    },
    Provider { name: "qiita", tag: Tag::AnyCase, enabled: |o| on(o.qiita), render: render_qiita },
    Provider { name: "zenn", tag: Tag::AnyCase, enabled: |o| on(o.zenn), render: render_zenn },
    Provider {
        name: "npmpackage",
        tag: Tag::AnyCase,
        enabled: |o| on(o.package_registry),
        render: render_npm_package,
    },
    Provider {
        name: "cratesio",
        tag: Tag::AnyCase,
        enabled: |o| on(o.package_registry),
        render: render_crates_io,
    },
    Provider {
        name: "pypi",
        tag: Tag::AnyCase,
        enabled: |o| on(o.package_registry),
        render: render_pypi,
    },
    Provider {
        name: "dockerhub",
        tag: Tag::AnyCase,
        enabled: |o| on(o.package_registry),
        render: render_docker_hub,
    },
    Provider {
        name: "codepen",
        tag: Tag::AnyCase,
        enabled: |o| on(o.playgrounds),
        render: render_codepen,
    },
    Provider {
        name: "jsfiddle",
        tag: Tag::AnyCase,
        enabled: |o| on(o.playgrounds),
        render: render_jsfiddle,
    },
    Provider {
        name: "observable",
        tag: Tag::AnyCase,
        enabled: |o| on(o.playgrounds),
        render: render_observable,
    },
    Provider { name: "vimeo", tag: Tag::AnyCase, enabled: |o| on(o.vimeo), render: render_vimeo },
    Provider {
        name: "twitch",
        tag: Tag::AnyCase,
        enabled: |o| on(o.twitch),
        render: render_twitch,
    },
    Provider {
        name: "discord",
        tag: Tag::AnyCase,
        enabled: |o| on(o.discord),
        render: render_discord,
    },
    Provider {
        name: "fediverse",
        tag: Tag::AnyCase,
        enabled: |o| on(o.fediverse),
        render: render_fediverse,
    },
    Provider {
        name: "mastodon",
        tag: Tag::AnyCase,
        enabled: |o| on(o.fediverse),
        render: render_mastodon,
    },
    Provider {
        name: "misskey",
        tag: Tag::AnyCase,
        enabled: |o| on(o.fediverse),
        render: render_misskey,
    },
    Provider {
        name: "mixi2",
        tag: Tag::AnyCase,
        enabled: |o| on(o.fediverse),
        render: render_mixi2,
    },
    Provider {
        name: "facebook",
        tag: Tag::AnyCase,
        enabled: |o| on(o.facebook),
        render: render_facebook,
    },
    Provider {
        name: "threads",
        tag: Tag::AnyCase,
        enabled: |o| on(o.threads),
        render: render_threads,
    },
    Provider {
        name: "instagram",
        tag: Tag::AnyCase,
        enabled: |o| on(o.instagram),
        render: render_instagram,
    },
    Provider {
        name: "webcontainer",
        tag: Tag::AnyCase,
        enabled: |o| on(o.web_container),
        render: render_webcontainer,
    },
];
