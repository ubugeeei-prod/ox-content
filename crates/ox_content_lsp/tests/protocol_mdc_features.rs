use serde_json::{Value, json};

use protocol_support::Server;

#[allow(dead_code)]
mod protocol_support;

#[test]
fn hover_describes_mdc_component_and_prop_from_registry() {
    let (root, registry_path) = write_mdc_registry("hover");
    let root_uri = format!("file://{}", root.display());
    let uri = format!("file://{}", root.join("content/guide.mdc").display());

    let mut server = Server::start();
    initialize_with_mdc_registry(&mut server, &root_uri, &uri, "ox-content.components.json");
    let _ = server.await_notification("textDocument/publishDiagnostics");

    let component = hover_at(&mut server, &uri, 0, 2);
    let component_value = component["contents"]["value"].as_str().expect("component hover value");
    assert!(
        component_value.contains("Inline alert callout.") && component_value.contains("`tone`"),
        "component hover should include description and props from {registry_path:?}, got {component_value}"
    );

    let prop = hover_at(&mut server, &uri, 0, 8);
    let prop_value = prop["contents"]["value"].as_str().expect("prop hover value");
    assert!(
        prop_value.contains("MDC prop for `<Alert>`")
            && prop_value.contains("Type: `info | warn | error`")
            && prop_value.contains("Visual tone."),
        "prop hover should include owning component, type, and docs, got {prop_value}"
    );

    server.shutdown();
}

#[test]
fn definition_jumps_mdc_component_and_prop_to_registry_names() {
    let (root, registry_path) = write_mdc_registry("definition");
    let root_uri = format!("file://{}", root.display());
    let uri = format!("file://{}", root.join("content/guide.mdc").display());

    let mut server = Server::start();
    initialize_with_mdc_registry(&mut server, &root_uri, &uri, "ox-content.components.json");
    let _ = server.await_notification("textDocument/publishDiagnostics");
    let registry_uri = format!("file://{}", registry_path.display());

    let component = definition_at(&mut server, &uri, 0, 2);
    assert_eq!(component["uri"].as_str(), Some(registry_uri.as_str()));
    assert_eq!(component["range"]["start"]["line"].as_u64(), Some(3));
    assert_eq!(component["range"]["start"]["character"].as_u64(), Some(15));

    let prop = definition_at(&mut server, &uri, 0, 8);
    assert_eq!(prop["uri"].as_str(), Some(registry_uri.as_str()));
    assert_eq!(prop["range"]["start"]["line"].as_u64(), Some(6));
    assert_eq!(prop["range"]["start"]["character"].as_u64(), Some(19));

    server.shutdown();
}

fn write_mdc_registry(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root =
        std::env::temp_dir().join(format!("ox-content-lsp-mdc-{name}-{}", std::process::id()));
    let content_dir = root.join("content");
    std::fs::create_dir_all(&content_dir).expect("create content dir");
    let registry_path = root.join("ox-content.components.json");
    std::fs::write(
        &registry_path,
        r#"{
  "components": [
    {
      "name": "Alert",
      "description": "Inline alert callout.",
      "attributes": [
        { "name": "tone", "type": "info | warn | error", "description": "Visual tone." },
        { "name": "icon", "description": "Icon name" }
      ]
    }
  ]
}
"#,
    )
    .expect("write registry");
    (root, registry_path)
}

fn initialize_with_mdc_registry(server: &mut Server, root_uri: &str, uri: &str, registry: &str) {
    let id = server.request(
        "initialize",
        json!({
            "capabilities": {},
            "processId": null,
            "rootUri": root_uri,
            "initializationOptions": { "mdcComponents": registry }
        }),
    );
    let _ = server.await_response(id);
    server.notify("initialized", json!({}));
    server.notify(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "markdown",
                "version": 1,
                "text": "<Alert tone=\"info\">Heads up</Alert>\n",
            }
        }),
    );
}

fn hover_at(server: &mut Server, uri: &str, line: u32, character: u32) -> Value {
    let id = server.request(
        "textDocument/hover",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }),
    );
    server.await_response(id)
}

fn definition_at(server: &mut Server, uri: &str, line: u32, character: u32) -> Value {
    let id = server.request(
        "textDocument/definition",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }),
    );
    server.await_response(id)
}
