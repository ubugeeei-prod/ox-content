use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use crate::string_builder::join3;

use super::{
    ExampleDoc, MediaTypeDoc, OpenApiDocsError, OpenApiDocsResult, OpenApiSpecInput, OperationDoc,
    ParameterDoc, RequestBodyDoc, ResponseDoc, SPEC_SLUG_PLACEHOLDER, SchemaDoc, SchemaPropertyDoc,
    SecurityRequirementDoc, SpecDoc, inspect,
};
use inspect::{
    example_value, fallback_title, invalid, object_string, pointer_string, resolve_node,
    schema_label, server_urls, slugify, sorted_child_objects, sorted_child_values, string_array,
    string_field, unique_slug,
};

const METHODS: &[&str] = &["get", "put", "post", "delete", "options", "head", "patch", "trace"];

#[derive(Debug, Clone, Default)]
struct SecurityScheme {
    kind: String,
    description: String,
}

pub(super) fn build_spec_doc(
    root: &Value,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<SpecDoc> {
    let object = root.as_object().ok_or_else(|| invalid(source_path, "expected an object"))?;
    let version = string_field(object, "openapi")
        .ok_or_else(|| invalid(source_path, "missing `openapi` version"))?;
    if !version.starts_with("3.0.") && !version.starts_with("3.1.") {
        return Err(OpenApiDocsError::UnsupportedVersion {
            path: source_path.to_string(),
            version: version.to_string(),
        });
    }

    let title = input
        .name
        .clone()
        .or_else(|| pointer_string(root, &["info", "title"]))
        .unwrap_or_else(|| fallback_title(source_path));
    let security_schemes = collect_security_schemes(root, source_path, input)?;
    let root_security = security_requirements(root.get("security"), &security_schemes);

    Ok(SpecDoc {
        title: title.clone(),
        description: pointer_string(root, &["info", "description"]).unwrap_or_default(),
        version: pointer_string(root, &["info", "version"]).unwrap_or_default(),
        slug: inspect::slugify(&title),
        servers: server_urls(root),
        operations: collect_operations(
            root,
            source_path,
            input,
            &security_schemes,
            &root_security,
        )?,
        schemas: collect_schemas(root, source_path, input)?,
    })
}

fn collect_operations(
    root: &Value,
    source_path: &str,
    input: &OpenApiSpecInput,
    schemes: &BTreeMap<String, SecurityScheme>,
    root_security: &[SecurityRequirementDoc],
) -> OpenApiDocsResult<Vec<OperationDoc>> {
    let mut operations = Vec::new();
    let mut slugs = BTreeSet::new();

    for (path_name, path_item) in sorted_child_objects(root.pointer("/paths")) {
        let common_parameters = collect_parameters(
            root,
            path_item.get("parameters").and_then(Value::as_array),
            source_path,
            input,
        )?;
        for method in METHODS {
            let Some(operation) = path_item.get(*method).and_then(Value::as_object) else {
                continue;
            };
            let mut parameters = common_parameters.clone();
            parameters.extend(collect_parameters(
                root,
                operation.get("parameters").and_then(Value::as_array),
                source_path,
                input,
            )?);
            let slug_seed = string_field(operation, "operationId")
                .map_or_else(|| join3(method, " ", path_name), ToOwned::to_owned);
            let operation_slug = unique_slug(&slugify(&slug_seed), &mut slugs);
            let method_name = method.to_ascii_uppercase();
            operations.push(OperationDoc {
                title: join3(&method_name, " ", path_name),
                file_name: join3(
                    "openapi/",
                    SPEC_SLUG_PLACEHOLDER,
                    &join3("/", &operation_slug, ".md"),
                ),
                method: method_name,
                path: path_name.to_string(),
                summary: string_field(operation, "summary").unwrap_or_default().to_string(),
                description: string_field(operation, "description").unwrap_or_default().to_string(),
                operation_id: string_field(operation, "operationId").map(ToOwned::to_owned),
                tags: string_array(operation.get("tags")),
                deprecated: operation.get("deprecated").and_then(Value::as_bool).unwrap_or(false),
                parameters,
                request_body: collect_request_body(
                    root,
                    operation.get("requestBody"),
                    source_path,
                    input,
                )?,
                responses: collect_responses(root, operation.get("responses"), source_path, input)?,
                security: operation.get("security").map_or_else(
                    || root_security.to_vec(),
                    |value| security_requirements(Some(value), schemes),
                ),
            });
        }
    }

    Ok(operations)
}

fn collect_parameters(
    root: &Value,
    values: Option<&Vec<Value>>,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Vec<ParameterDoc>> {
    let mut parameters = Vec::new();
    for parameter in values.into_iter().flatten() {
        let resolved = resolve_node(root, parameter, source_path, input)?;
        let Some(object) = resolved.as_object() else {
            continue;
        };
        let schema = object
            .get("schema")
            .map(|schema| schema_label(root, schema, source_path, input))
            .transpose()?
            .unwrap_or_else(|| "-".to_string());
        parameters.push(ParameterDoc {
            name: string_field(object, "name").unwrap_or_default().to_string(),
            location: string_field(object, "in").unwrap_or_default().to_string(),
            required: object.get("required").and_then(Value::as_bool).unwrap_or(false),
            schema,
            description: string_field(object, "description").unwrap_or_default().to_string(),
            example: object.get("example").map(example_value),
        });
    }
    Ok(parameters)
}

fn collect_request_body(
    root: &Value,
    value: Option<&Value>,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Option<RequestBodyDoc>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let resolved = resolve_node(root, value, source_path, input)?;
    let Some(object) = resolved.as_object() else {
        return Ok(None);
    };
    Ok(Some(RequestBodyDoc {
        required: object.get("required").and_then(Value::as_bool).unwrap_or(false),
        description: string_field(object, "description").unwrap_or_default().to_string(),
        content: collect_content(root, object.get("content"), source_path, input)?,
    }))
}

fn collect_responses(
    root: &Value,
    value: Option<&Value>,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Vec<ResponseDoc>> {
    let mut responses = Vec::new();
    for (status, response) in sorted_child_values(value) {
        let resolved = resolve_node(root, response, source_path, input)?;
        let Some(object) = resolved.as_object() else {
            continue;
        };
        responses.push(ResponseDoc {
            status: status.to_string(),
            description: string_field(object, "description").unwrap_or_default().to_string(),
            content: collect_content(root, object.get("content"), source_path, input)?,
        });
    }
    Ok(responses)
}

fn collect_content(
    root: &Value,
    value: Option<&Value>,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Vec<MediaTypeDoc>> {
    let mut content = Vec::new();
    for (media_type, media) in sorted_child_objects(value) {
        let schema = media
            .get("schema")
            .map(|schema| schema_label(root, schema, source_path, input))
            .transpose()?
            .unwrap_or_else(|| "-".to_string());
        content.push(MediaTypeDoc {
            media_type: media_type.to_string(),
            schema,
            examples: collect_examples(root, media, source_path, input)?,
        });
    }
    Ok(content)
}

fn collect_examples(
    root: &Value,
    media: &Map<String, Value>,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Vec<ExampleDoc>> {
    let mut examples = Vec::new();
    if let Some(value) = media.get("example") {
        examples.push(ExampleDoc { name: "example".to_string(), value: example_value(value) });
    }
    for (name, example) in sorted_child_values(media.get("examples")) {
        let resolved = resolve_node(root, example, source_path, input)?;
        let value = resolved.get("value").unwrap_or(&resolved);
        examples.push(ExampleDoc { name: name.to_string(), value: example_value(value) });
    }
    Ok(examples)
}

fn collect_schemas(
    root: &Value,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Vec<SchemaDoc>> {
    let mut schemas = Vec::new();
    for (name, schema) in sorted_child_values(root.pointer("/components/schemas")) {
        let resolved = resolve_node(root, schema, source_path, input)?;
        let object = resolved.as_object();
        schemas.push(SchemaDoc {
            name: name.to_string(),
            schema: schema_label(root, schema, source_path, input)?,
            description: object
                .and_then(|schema| object_string(schema, "description"))
                .unwrap_or_default()
                .to_string(),
            required: object.map_or_else(Vec::new, |schema| string_array(schema.get("required"))),
            properties: collect_schema_properties(root, &resolved, source_path, input)?,
        });
    }
    Ok(schemas)
}

fn collect_schema_properties(
    root: &Value,
    schema: &Value,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Vec<SchemaPropertyDoc>> {
    let Some(schema_object) = schema.as_object() else {
        return Ok(Vec::new());
    };
    let required = string_array(schema_object.get("required")).into_iter().collect::<BTreeSet<_>>();
    let mut properties = Vec::new();
    for (name, property) in sorted_child_values(schema_object.get("properties")) {
        let property_object = property.as_object();
        properties.push(SchemaPropertyDoc {
            name: name.to_string(),
            schema: schema_label(root, property, source_path, input)?,
            required: required.contains(name),
            description: property_object
                .and_then(|property| object_string(property, "description"))
                .unwrap_or_default()
                .to_string(),
        });
    }
    Ok(properties)
}

fn collect_security_schemes(
    root: &Value,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<BTreeMap<String, SecurityScheme>> {
    let mut schemes = BTreeMap::new();
    for (name, scheme) in sorted_child_values(root.pointer("/components/securitySchemes")) {
        let resolved = resolve_node(root, scheme, source_path, input)?;
        let Some(object) = resolved.as_object() else {
            continue;
        };
        schemes.insert(
            name.to_string(),
            SecurityScheme {
                kind: string_field(object, "type").unwrap_or("security").to_string(),
                description: string_field(object, "description").unwrap_or_default().to_string(),
            },
        );
    }
    Ok(schemes)
}

fn security_requirements(
    value: Option<&Value>,
    schemes: &BTreeMap<String, SecurityScheme>,
) -> Vec<SecurityRequirementDoc> {
    let mut requirements = Vec::new();
    for requirement in value.and_then(Value::as_array).into_iter().flatten() {
        for (name, scopes) in sorted_child_values(Some(requirement)) {
            let scheme = schemes.get(name).cloned().unwrap_or_default();
            requirements.push(SecurityRequirementDoc {
                name: name.to_string(),
                kind: scheme.kind,
                scopes: string_array(Some(scopes)),
                description: scheme.description,
            });
        }
    }
    requirements
}
