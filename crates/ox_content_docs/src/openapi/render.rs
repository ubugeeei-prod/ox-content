use std::collections::BTreeMap;

use crate::nav::DocsNavItem;
use crate::string_builder::{StringBuilder, join3};

use super::routes::{operation_file, route_path, spec_file};
use super::{MediaTypeDoc, OperationDoc, SchemaDoc, SpecDoc};

pub struct RenderedOpenApiSpec {
    pub pages: BTreeMap<String, String>,
    pub nav_item: DocsNavItem,
}

pub(super) fn render_spec_doc(spec: &SpecDoc, base_path: Option<&str>) -> RenderedOpenApiSpec {
    let mut pages = BTreeMap::new();
    let index_file = spec_file(spec, "index.md");
    pages.insert(index_file.clone(), render_index(spec));

    for operation in &spec.operations {
        let file_name = operation_file(operation, spec);
        pages.insert(file_name, render_operation(spec, operation));
    }

    if !spec.schemas.is_empty() {
        pages.insert(spec_file(spec, "schemas.md"), render_schemas(spec));
    }

    RenderedOpenApiSpec { nav_item: render_nav(spec, base_path, &index_file), pages }
}

fn render_index(spec: &SpecDoc) -> String {
    let mut out = page_frontmatter(&spec.title, &spec.description);
    out.push_str("# ");
    out.push_str(&spec.title);
    out.push_str("\n\n");
    if !spec.description.is_empty() {
        out.push_str(&spec.description);
        out.push_str("\n\n");
    }
    if !spec.version.is_empty() {
        out.push_str("- Version: `");
        out.push_str(&spec.version);
        out.push_str("`\n");
    }
    if !spec.servers.is_empty() {
        out.push_str("- Servers: ");
        out.push_str(
            &spec.servers.iter().map(|server| inline_code(server)).collect::<Vec<_>>().join(", "),
        );
        out.push('\n');
    }
    if !spec.version.is_empty() || !spec.servers.is_empty() {
        out.push('\n');
    }
    out.push_str("| Endpoint | Summary | Tags |\n| --- | --- | --- |\n");
    for operation in &spec.operations {
        let file_name = operation_file(operation, spec);
        let link = file_name.rsplit('/').next().unwrap_or(&file_name);
        out.push_str("| [");
        out.push_str(&escape_table_cell(&operation.title));
        out.push_str("](./");
        out.push_str(link);
        out.push_str(") | ");
        out.push_str(&escape_table_cell(&operation.summary));
        out.push_str(" | ");
        out.push_str(&escape_table_cell(&operation.tags.join(", ")));
        out.push_str(" |\n");
    }
    if !spec.schemas.is_empty() {
        out.push_str("\n[Schema reference](./schemas.md)\n");
    }
    out
}

fn render_operation(spec: &SpecDoc, operation: &OperationDoc) -> String {
    let description = first_non_empty(&operation.summary, &operation.description);
    let mut out = page_frontmatter(&operation.title, description);
    out.push_str("# ");
    out.push_str(&operation.title);
    out.push_str("\n\n");
    if !operation.summary.is_empty() {
        out.push_str(&operation.summary);
        out.push_str("\n\n");
    }
    if !operation.description.is_empty() {
        out.push_str(&operation.description);
        out.push_str("\n\n");
    }
    out.push_str("## Endpoint\n\n");
    out.push_str("- Method: `");
    out.push_str(&operation.method);
    out.push_str("`\n- Path: `");
    out.push_str(&operation.path);
    out.push_str("`\n");
    if let Some(operation_id) = &operation.operation_id {
        out.push_str("- Operation ID: `");
        out.push_str(operation_id);
        out.push_str("`\n");
    }
    if !operation.tags.is_empty() {
        out.push_str("- Tags: ");
        out.push_str(
            &operation.tags.iter().map(|tag| inline_code(tag)).collect::<Vec<_>>().join(", "),
        );
        out.push('\n');
    }
    if operation.deprecated {
        out.push_str("- Deprecated: `true`\n");
    }
    render_security(&mut out, operation);
    render_parameters(&mut out, operation);
    render_request_body(&mut out, operation);
    render_responses(&mut out, operation);

    if !spec.schemas.is_empty() {
        out.push_str("\n[Back to ");
        out.push_str(&spec.title);
        out.push_str("](./index.md)\n");
    }
    out
}

fn render_security(out: &mut String, operation: &OperationDoc) {
    out.push_str("\n## Security\n\n");
    if operation.security.is_empty() {
        out.push_str("No security requirements.\n");
        return;
    }
    out.push_str("| Scheme | Type | Scopes | Description |\n| --- | --- | --- | --- |\n");
    for requirement in &operation.security {
        out.push_str(&table_row(&[
            &requirement.name,
            &requirement.kind,
            &requirement.scopes.join(", "),
            &requirement.description,
        ]));
    }
}

fn render_parameters(out: &mut String, operation: &OperationDoc) {
    out.push_str("\n## Parameters\n\n");
    if operation.parameters.is_empty() {
        out.push_str("No parameters.\n");
        return;
    }
    out.push_str(
        "| Name | In | Required | Schema | Description |\n| --- | --- | --- | --- | --- |\n",
    );
    for parameter in &operation.parameters {
        out.push_str(&table_row(&[
            &parameter.name,
            &parameter.location,
            if parameter.required { "yes" } else { "no" },
            &parameter.schema,
            &parameter.description,
        ]));
        if let Some(example) = &parameter.example {
            out.push_str("\nExample for `");
            out.push_str(&parameter.name);
            out.push_str("`:\n\n```json\n");
            out.push_str(example);
            out.push_str("\n```\n");
        }
    }
}

fn render_request_body(out: &mut String, operation: &OperationDoc) {
    out.push_str("\n## Request Body\n\n");
    let Some(body) = &operation.request_body else {
        out.push_str("No request body.\n");
        return;
    };
    if !body.description.is_empty() {
        out.push_str(&body.description);
        out.push_str("\n\n");
    }
    out.push_str("- Required: `");
    out.push_str(if body.required { "true" } else { "false" });
    out.push_str("`\n\n");
    render_media_types(out, &body.content);
}

fn render_responses(out: &mut String, operation: &OperationDoc) {
    out.push_str("\n## Responses\n\n");
    if operation.responses.is_empty() {
        out.push_str("No responses declared.\n");
        return;
    }
    for response in &operation.responses {
        out.push_str("### `");
        out.push_str(&response.status);
        out.push_str("`\n\n");
        if !response.description.is_empty() {
            out.push_str(&response.description);
            out.push_str("\n\n");
        }
        render_media_types(out, &response.content);
    }
}

fn render_media_types(out: &mut String, content: &[MediaTypeDoc]) {
    if content.is_empty() {
        out.push_str("No content types declared.\n");
        return;
    }
    out.push_str("| Media type | Schema |\n| --- | --- |\n");
    for media in content {
        out.push_str(&table_row(&[&media.media_type, &media.schema]));
    }
    for media in content {
        for example in &media.examples {
            out.push_str("\nExample `");
            out.push_str(&example.name);
            out.push_str("` for `");
            out.push_str(&media.media_type);
            out.push_str("`:\n\n```json\n");
            out.push_str(&example.value);
            out.push_str("\n```\n");
        }
    }
}

fn render_schemas(spec: &SpecDoc) -> String {
    let mut out = page_frontmatter("Schema reference", "OpenAPI component schemas.");
    out.push_str("# Schema Reference\n\n");
    for schema in &spec.schemas {
        render_schema(&mut out, schema);
    }
    out
}

fn render_schema(out: &mut String, schema: &SchemaDoc) {
    out.push_str("## ");
    out.push_str(&schema.name);
    out.push_str("\n\n");
    if !schema.description.is_empty() {
        out.push_str(&schema.description);
        out.push_str("\n\n");
    }
    out.push_str("- Schema: `");
    out.push_str(&schema.schema);
    out.push_str("`\n");
    if !schema.required.is_empty() {
        out.push_str("- Required: ");
        out.push_str(
            &schema.required.iter().map(|name| inline_code(name)).collect::<Vec<_>>().join(", "),
        );
        out.push('\n');
    }
    if schema.properties.is_empty() {
        out.push('\n');
        return;
    }
    out.push_str("\n| Property | Required | Schema | Description |\n| --- | --- | --- | --- |\n");
    for property in &schema.properties {
        out.push_str(&table_row(&[
            &property.name,
            if property.required { "yes" } else { "no" },
            &property.schema,
            &property.description,
        ]));
    }
    out.push('\n');
}

fn render_nav(spec: &SpecDoc, base_path: Option<&str>, index_file: &str) -> DocsNavItem {
    let mut tag_groups = BTreeMap::<String, Vec<DocsNavItem>>::new();
    for operation in &spec.operations {
        let tag = operation.tags.first().cloned().unwrap_or_else(|| "Endpoints".to_string());
        let file_name = operation_file(operation, spec);
        tag_groups.entry(tag).or_default().push(DocsNavItem {
            title: operation.title.clone(),
            path: route_path(base_path, &file_name),
            children: None,
        });
    }
    let mut children = tag_groups
        .into_iter()
        .map(|(title, items)| DocsNavItem {
            title,
            path: route_path(base_path, index_file),
            children: Some(items),
        })
        .collect::<Vec<_>>();
    if !spec.schemas.is_empty() {
        children.push(DocsNavItem {
            title: "Schemas".to_string(),
            path: route_path(base_path, &spec_file(spec, "schemas.md")),
            children: None,
        });
    }
    DocsNavItem {
        title: spec.title.clone(),
        path: route_path(base_path, index_file),
        children: Some(children),
    }
}

fn page_frontmatter(title: &str, description: &str) -> String {
    let mut out = StringBuilder::with_capacity(title.len() + description.len() + 80);
    out.push_str("---\ntitle: ");
    out.push_str(&yaml_string(title));
    if !description.is_empty() {
        out.push_str("\ndescription: ");
        out.push_str(&yaml_string(description));
    }
    out.push_str("\napiReference: true\n---\n\n");
    out.into_string()
}

fn inline_code(value: &str) -> String {
    join3("`", value, "`")
}

fn yaml_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

fn escape_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

fn table_row(cells: &[&str]) -> String {
    let escaped = cells.iter().map(|cell| escape_table_cell(cell)).collect::<Vec<_>>();
    join3("| ", &escaped.join(" | "), " |\n")
}

fn first_non_empty<'a>(first: &'a str, second: &'a str) -> &'a str {
    if first.is_empty() { second } else { first }
}
