use super::super::generate_html;
use super::rendering::{config, page};

#[test]
fn includes_css_when_gallery_markup_is_present() {
    let page_data = page(
        "Gallery",
        None,
        r#"<figure class="ox-image-gallery"><ul class="ox-image-gallery__items"></ul></figure>"#,
        vec![],
        None,
        "gallery",
    );
    let html = generate_html(&page_data, &[], &config("Test Site", "/", None));

    assert!(html.contains("ox-content:css:image-gallery:start"), "{html}");
    assert!(html.contains(".content .ox-image-gallery__items"), "{html}");
}
