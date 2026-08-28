use napi_derive::napi;

use crate::JsReaderChrome;

pub(super) fn convert_reader_chrome(
    chrome: Option<JsReaderChrome>,
) -> ox_content_ssg::ReaderChrome {
    match chrome {
        None => ox_content_ssg::ReaderChrome::disabled(),
        Some(chrome) => ox_content_ssg::ReaderChrome {
            copy: chrome.copy.unwrap_or(true),
            external_links: chrome.external_links.unwrap_or(true),
            back_to_top: chrome.back_to_top.unwrap_or(true),
        },
    }
}

/// Rewrites rendered article HTML with reader-chrome controls.
#[napi(js_name = "applySsgReaderChromeHtml")]
pub fn apply_ssg_reader_chrome_html(html: String, chrome: Option<JsReaderChrome>) -> String {
    ox_content_ssg::apply_reader_chrome(&html, convert_reader_chrome(chrome))
}

/// CSS used by the built-in reader chrome and custom hosts.
#[napi(js_name = "getSsgReaderChromeCss")]
pub fn get_ssg_reader_chrome_css() -> String {
    ox_content_ssg::READER_CHROME_CSS.to_string()
}

/// Auto-initializing browser script used by the built-in reader chrome.
#[napi(js_name = "getSsgReaderChromeScript")]
pub fn get_ssg_reader_chrome_script() -> String {
    ox_content_ssg::READER_CHROME_JS.to_string()
}
