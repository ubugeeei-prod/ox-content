use super::super::generate_html;
use super::rendering::{config, page};

#[test]
fn includes_css_when_citation_markup_is_present() {
    let page_data = page(
        "Citations",
        None,
        r##"<p>See <span class="ox-cite"><a class="ox-cite__ref" href="#ref-rfc9110">[1]</a></span>.</p><section class="ox-bibliography"><ol><li class="ox-bibliography__item" id="ref-rfc9110">HTTP Semantics</li></ol></section>"##,
        vec![],
        None,
        "citations",
    );
    let html = generate_html(&page_data, &[], &config("Test Site", "/", None));

    assert!(html.contains("ox-content:css:plugin-citations:start"), "{html}");
    assert!(html.contains(".content .ox-bibliography__item"), "{html}");
    assert!(html.contains(".content .ox-cite__ref"), "{html}");
}
