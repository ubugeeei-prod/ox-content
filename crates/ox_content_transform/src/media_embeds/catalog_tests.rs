//! The providers #1100 named, and the rules every one of them follows.

use super::transform_media_embeds;

// --- Catalog additions (#1100) -------------------------------------------

fn catalog_options() -> crate::MediaEmbedsOptions {
    crate::MediaEmbedsOptions {
        loom: Some(true),
        asciinema: Some(true),
        figma: Some(true),
        note: Some(true),
        google_slides: Some(true),
        playgrounds: Some(true),
        ..Default::default()
    }
}

fn render_catalog(html: &str) -> String {
    transform_media_embeds(html, Some(&catalog_options()))
}

#[test]
fn note_article_renders_a_card() {
    let out = render_catalog(r#"<note url="https://note.com/someone/n/nabc123"></note>"#);
    assert!(out.contains("ox-provider-card--note"), "{out}");
    assert!(out.contains("note"), "{out}");
}

#[test]
fn note_magazine_renders_a_card() {
    let out = render_catalog(r#"<note url="https://note.com/someone/m/mabc123"></note>"#);
    assert!(out.contains("ox-provider-card--note"), "{out}");
}

#[test]
fn a_note_profile_is_not_an_article() {
    // No `/n/` or `/m/` segment, so there is nothing to card.
    let out = render_catalog(r#"<note url="https://note.com/someone"></note>"#);
    assert!(!out.contains("ox-provider-card--note"), "{out}");
}

#[test]
fn loom_share_link_renders_a_card() {
    let out = render_catalog(r#"<loom url="https://www.loom.com/share/abc123def"></loom>"#);
    assert!(out.contains("ox-provider-card--loom"), "{out}");
    assert!(out.contains("Watch on Loom"), "{out}");
}

#[test]
fn a_loom_folder_is_not_a_recording() {
    let out = render_catalog(r#"<loom url="https://www.loom.com/share/folder/abc123"></loom>"#);
    assert!(!out.contains("ox-provider-card--loom"), "{out}");
}

#[test]
fn asciinema_cast_renders_a_card() {
    let out = render_catalog(r#"<asciinema url="https://asciinema.org/a/569727"></asciinema>"#);
    assert!(out.contains("ox-provider-card--asciinema"), "{out}");
    assert!(out.contains("Play recording"), "{out}");
}

#[test]
fn figma_file_design_and_prototype_all_render() {
    for url in [
        "https://www.figma.com/file/AbC123/My-Design",
        "https://www.figma.com/design/AbC123/My-Design",
        "https://www.figma.com/proto/AbC123/My-Design",
        "https://www.figma.com/board/AbC123/My-Board",
        "https://www.figma.com/community/file/AbC123/Design-Kit",
    ] {
        let out = render_catalog(&format!(r#"<figma url="{url}"></figma>"#));
        assert!(out.contains("ox-provider-card--figma"), "{url}: {out}");
    }
}

#[test]
fn a_figma_url_with_no_file_key_does_not_render() {
    let out = render_catalog(r#"<figma url="https://www.figma.com/file/"></figma>"#);
    assert!(!out.contains("ox-provider-card--figma"), "{out}");
}

#[test]
fn google_slides_renders_both_the_file_and_published_forms() {
    for url in [
        "https://docs.google.com/presentation/d/1AbC_123/edit",
        "https://docs.google.com/presentation/d/e/2PACX-tok3n/pub",
    ] {
        let out = render_catalog(&format!(r#"<googleslides url="{url}"></googleslides>"#));
        assert!(out.contains("ox-provider-card--googleslides"), "{url}: {out}");
    }
}

#[test]
fn a_google_doc_is_not_a_deck() {
    let out = render_catalog(
        r#"<googleslides url="https://docs.google.com/document/d/1AbC/edit"></googleslides>"#,
    );
    assert!(!out.contains("ox-provider-card--googleslides"), "{out}");
}

#[test]
fn replit_renders_under_the_shared_playgrounds_option() {
    let out = render_catalog(r#"<replit url="https://replit.com/@someone/my-repl"></replit>"#);
    assert!(out.contains("ox-provider-card--replit"), "{out}");
}

#[test]
fn a_replit_url_without_an_owner_handle_does_not_render() {
    // The owner segment must carry the `@`; a bare path is some other page.
    let out = render_catalog(r#"<replit url="https://replit.com/someone/my-repl"></replit>"#);
    assert!(!out.contains("ox-provider-card--replit"), "{out}");
}

#[test]
fn every_catalog_addition_rejects_a_look_alike_host() {
    for (tag, url) in [
        ("note", "https://note.com.evil.example/someone/n/nabc"),
        ("loom", "https://loom.com.evil.example/share/abc"),
        ("asciinema", "https://asciinema.org.evil.example/a/1"),
        ("figma", "https://figma.com.evil.example/file/AbC/x"),
        ("googleslides", "https://docs.google.com.evil.example/presentation/d/1AbC/edit"),
        ("replit", "https://replit.com.evil.example/@someone/repl"),
    ] {
        let out = render_catalog(&format!(r#"<{tag} url="{url}"></{tag}>"#));
        assert!(!out.contains("ox-provider-card--"), "{tag} accepted a look-alike host: {out}");
        // It still becomes a plain link rather than staying as markup.
        assert!(out.contains("ox-embed-fallback"), "{tag}: {out}");
    }
}

#[test]
fn a_catalog_provider_stays_off_until_its_option_is_set() {
    let out = transform_media_embeds(
        r#"<figma url="https://www.figma.com/file/AbC123/My-Design"></figma>"#,
        Some(&crate::MediaEmbedsOptions::default()),
    );
    assert!(!out.contains("ox-provider-card"), "{out}");
}
