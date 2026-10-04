use protocol_support::{Server, temp_uri};
use serde_json::json;
mod protocol_support;

#[test]
fn native_prose_is_opt_in_and_quickfixes_have_exact_utf16_ranges() {
    let uri = temp_uri("native-prose.md");
    let text = "# Guide\n\n😀 Javascript\n";
    let mut disabled = Server::start();
    disabled.initialize_and_open(&uri, text);
    let values = disabled.await_diagnostics_version(&uri, 1);
    assert!(
        values["diagnostics"].as_array().unwrap().iter().all(|v| v["source"] != "ox-content-lint")
    );
    disabled.shutdown();

    let mut server = Server::start();
    server.initialize_and_open_with(&uri, text, json!({"markdownLint": {"textRules": {"terminology": [{"term":"Javascript","replacement":"JavaScript"}]}}}));
    let values = server.await_diagnostics_version(&uri, 1);
    let diagnostic = values["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["code"] == "terminology")
        .unwrap();
    assert_eq!(diagnostic["range"]["start"], json!({"line":2,"character":3}));
    let id = server.request("textDocument/codeAction", json!({"textDocument":{"uri":uri},"range":diagnostic["range"],"context":{"diagnostics":[diagnostic]}}));
    let actions = server.await_response(id);
    assert!(
        actions
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["edit"]["changes"][&uri][0]["newText"] == "JavaScript")
    );
    server.did_change_incremental(&uri, 2, (2, 3), (2, 13), "JavaScript");
    let values = server.await_diagnostics_version(&uri, 2);
    assert!(values["diagnostics"].as_array().unwrap().iter().all(|v| v["code"] != "terminology"));
    server.shutdown();
}
