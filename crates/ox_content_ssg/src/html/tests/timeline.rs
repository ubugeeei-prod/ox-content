use super::super::generate_html;
use super::rendering::{config, page};

#[test]
fn includes_css_when_timeline_markup_is_present() {
    let page_data = page(
        "Timeline",
        None,
        r#"<section class="ox-timeline"><ol class="ox-timeline__items"></ol></section>"#,
        vec![],
        None,
        "timeline",
    );
    let html = generate_html(&page_data, &[], &config("Test Site", "/", None));

    assert!(html.contains("ox-content:css:timeline:start"), "{html}");
    assert!(html.contains(".content .ox-timeline__item"), "{html}");
    assert!(html.contains("@media (max-width: 640px)"), "{html}");
}
