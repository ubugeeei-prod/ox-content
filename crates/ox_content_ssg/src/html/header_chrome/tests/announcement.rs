use super::super::super::ThemeConfig;
use super::super::{PageChromeFlags, ThemeAnnouncement};
use super::{announce_html, body_class, render};

#[test]
fn announcement_escaped() {
    let html = render(
        Some(ThemeConfig {
            announcement: Some(ThemeAnnouncement {
                text: "<script>alert(1)</script>".to_string(),
                link: Some("https://example.com/news".to_string()),
                dismiss_key: Some("welcome".to_string()),
            }),
            ..ThemeConfig::default()
        }),
        false,
        PageChromeFlags::default(),
    );

    assert!(html.contains(r#"<div class="ox-announce""#), "{html}");
    assert!(body_class(&html).contains("ox-has-announce"), "{html}");
    assert!(html.contains(r#"data-ox-announce="welcome""#), "{html}");
    assert!(html.contains(".ox-announce[hidden]"), "{html}");
    assert!(html.contains("parentElement"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    assert!(!html.contains("<script>alert(1)</script>"), "{html}");
    assert!(html.contains(r#"href="https://example.com/news""#), "{html}");
}

#[test]
fn announcement_javascript_link_rejected() {
    let html = render(
        Some(ThemeConfig {
            announcement: Some(ThemeAnnouncement {
                text: "Ship day".to_string(),
                link: Some("javascript:alert(1)".to_string()),
                dismiss_key: None,
            }),
            ..ThemeConfig::default()
        }),
        false,
        PageChromeFlags::default(),
    );

    assert!(html.contains("Ship day"), "{html}");
    assert!(!html.contains("javascript:"), "{html}");
    assert!(!announce_html(&html).contains("<a "), "{}", announce_html(&html));
}

#[test]
fn announcement_http_and_protocol_relative_rejected() {
    for link in ["http://evil.example/x", "//evil.example/x", "data:text/html,hi"] {
        let html = render(
            Some(ThemeConfig {
                announcement: Some(ThemeAnnouncement {
                    text: "Notice".to_string(),
                    link: Some(link.to_string()),
                    dismiss_key: None,
                }),
                ..ThemeConfig::default()
            }),
            false,
            PageChromeFlags::default(),
        );
        let bar = announce_html(&html);
        assert!(bar.contains("Notice"), "{bar}");
        assert!(!bar.contains("<a "), "{link} {bar}");
        assert!(!html.contains(link), "{html}");
    }
}
