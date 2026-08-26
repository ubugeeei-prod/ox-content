use super::{ReportMode, resolve, transform};
use crate::transformer::MarkdownTransformer;
use crate::{
    AttrsOptions, ContainerOptions, ContainerTypeOptions, ImageGalleryOptions, TransformOptions,
};

fn gallery_options(missing_alt: Option<&str>, empty: Option<&str>) -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        image_galleries: Some(ImageGalleryOptions {
            enabled: Some(true),
            lazy: None,
            missing_alt: missing_alt.map(ToString::to_string),
            empty: empty.map(ToString::to_string),
        }),
        ..Default::default()
    }
}

fn html(source: &str, options: TransformOptions) -> crate::TransformResult {
    MarkdownTransformer::from_options(&options).transform(source)
}

#[test]
fn resolve_is_none_when_omitted_or_false() {
    assert!(resolve(None, false, None).is_none());
    assert!(
        resolve(
            Some(&ImageGalleryOptions { enabled: Some(false), ..Default::default() }),
            false,
            None,
        )
        .is_none()
    );
}

#[test]
fn resolve_defaults_to_strict_accessibility() {
    let resolved =
        resolve(Some(&ImageGalleryOptions { enabled: None, ..Default::default() }), false, None)
            .expect("enabled");
    assert!(resolved.image.lazy);
    assert_eq!(resolved.missing_alt, ReportMode::Error);
    assert_eq!(resolved.empty, ReportMode::Error);
}

#[test]
fn inherits_lazy_default_from_images_option() {
    let images = super::super::images::ResolvedImageOptions { lazy: false, attrs: false };
    let resolved = resolve(
        Some(&ImageGalleryOptions { enabled: None, ..Default::default() }),
        false,
        Some(&images),
    )
    .expect("enabled");
    assert!(!resolved.image.lazy);
}

#[test]
fn disabled_by_default() {
    let source = "::: gallery\n![Alt](/x.png \"Caption\")\n:::\n";
    let result = html(source, TransformOptions::default());
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(!result.html.contains("ox-image-gallery"), "{html}", html = result.html);
    assert!(result.html.contains("gallery"), "{html}", html = result.html);
}

#[test]
fn enabled_gallery_renders_static_semantic_html() {
    let result = html(
        "::: gallery title=\"Launch shots\"\n- ![Stage](/stage.png \"Keynote\")\n- ![Booth](https://example.com/booth.jpg)\n:::\n",
        gallery_options(None, None),
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(
        result.html.contains(r#"<figure class="ox-image-gallery">"#),
        "{html}",
        html = result.html
    );
    assert!(
        result
            .html
            .contains(r#"<figcaption class="ox-image-gallery__caption">Launch shots</figcaption>"#),
        "{html}",
        html = result.html
    );
    assert!(
        result.html.contains(r#"<ul class="ox-image-gallery__items">"#),
        "{html}",
        html = result.html
    );
    assert!(
        result.html.contains(r#"<li class="ox-image-gallery__item">"#),
        "{html}",
        html = result.html
    );
    assert!(
        result.html.contains(r#"<img src="/stage.png" alt="Stage" loading="lazy">"#),
        "{html}",
        html = result.html
    );
    assert!(
        result
            .html
            .contains(r#"<figcaption class="ox-image-gallery__item-caption">Keynote</figcaption>"#),
        "{html}",
        html = result.html
    );
    assert!(
        result
            .html
            .contains(r#"<img src="https://example.com/booth.jpg" alt="Booth" loading="lazy">"#),
        "{html}",
        html = result.html
    );
    assert!(!result.html.contains("```"), "{html}", html = result.html);
}

#[test]
fn attrs_dimensions_and_eager_images_compose() {
    let mut options = gallery_options(None, None);
    options.attributes = Some(AttrsOptions { enabled: Some(true) });
    options.image_galleries.as_mut().unwrap().lazy = Some(false);
    let result = html(
        "::: gallery\n![Wide](./wide.png){.hero width=640 height=360 data-kind=wide}\n:::\n",
        options,
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.html.contains(r#"<img src="./wide.png" alt="Wide" class="hero" data-kind="wide" width="640" height="360">"#), "{html}", html = result.html);
    assert!(!result.html.contains("loading="), "{html}", html = result.html);
    assert!(!result.html.contains("{.hero"), "{html}", html = result.html);
}

#[test]
fn missing_alt_error_preserves_gallery_source() {
    let result = html("::: gallery\n![](/x.png)\n:::\n", gallery_options(None, None));
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(result.errors[0].contains("missing alt text"), "{:?}", result.errors);
    assert!(!result.html.contains("ox-image-gallery"), "{html}", html = result.html);
    assert!(result.html.contains("gallery"), "{html}", html = result.html);
}

#[test]
fn missing_alt_warn_renders_and_reports_diagnostic() {
    let result = html("::: gallery\n![](/x.png)\n:::\n", gallery_options(Some("warn"), None));
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(
        result.html.contains(r#"<img src="/x.png" alt="" loading="lazy">"#),
        "{html}",
        html = result.html
    );
}

#[test]
fn empty_gallery_is_diagnostic_and_not_rewritten() {
    let result = html("::: gallery\n\n:::\n", gallery_options(None, None));
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(result.errors[0].contains("empty"), "{:?}", result.errors);
    assert!(!result.html.contains("ox-image-gallery"), "{html}", html = result.html);
}

#[test]
fn malformed_item_is_actionable() {
    let result = html("::: gallery\nnot an image\n:::\n", gallery_options(None, None));
    assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    assert!(result.errors[0].contains("Markdown image"), "{:?}", result.errors);
    assert!(!result.html.contains("ox-image-gallery"), "{html}", html = result.html);
}

#[test]
fn skips_fenced_and_indented_gallery_markers() {
    let fenced =
        html("````md\n::: gallery\n![Alt](/x.png)\n:::\n````\n", gallery_options(None, None));
    assert!(!fenced.html.contains("ox-image-gallery"), "{html}", html = fenced.html);

    let indented =
        html("    ::: gallery\n    ![Alt](/x.png)\n    :::\n", gallery_options(None, None));
    assert!(!indented.html.contains("ox-image-gallery"), "{html}", html = indented.html);
}

#[test]
fn unsafe_sources_are_sanitized_like_images() {
    let result =
        html("::: gallery\n![Alt](javascript:alert(1))\n:::\n", gallery_options(None, None));
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(
        result.html.contains(r#"<img alt="Alt" loading="lazy">"#),
        "{html}",
        html = result.html
    );
    assert!(!result.html.contains("javascript:"), "{html}", html = result.html);
}

#[test]
fn gallery_precedes_custom_container_with_same_name() {
    let mut types = rustc_hash::FxHashMap::default();
    types.insert(
        "gallery".to_string(),
        ContainerTypeOptions { title: Some("Container".to_string()), tag: None },
    );
    let result = html(
        "::: gallery\n![Alt](/x.png)\n:::\n",
        TransformOptions {
            gfm: Some(true),
            containers: Some(ContainerOptions { enabled: Some(true), types: Some(types) }),
            image_galleries: Some(ImageGalleryOptions {
                enabled: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    );
    assert!(result.html.contains("ox-image-gallery"), "{html}", html = result.html);
    assert!(!result.html.contains("ox-container--gallery"), "{html}", html = result.html);
}

#[test]
fn preprocess_snapshot_html() {
    let resolved = resolve(
        Some(&ImageGalleryOptions { enabled: Some(true), ..Default::default() }),
        false,
        None,
    )
    .expect("enabled");
    let mut errors = Vec::new();
    let source = transform(
        "::: gallery \"Screenshots\"\n![One](/one.png \"First\")\n![Two](/two.png)\n:::\n",
        &resolved,
        &mut errors,
    );
    assert!(errors.is_empty(), "{errors:?}");
    insta::assert_snapshot!(source);
}
