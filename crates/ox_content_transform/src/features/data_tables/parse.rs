use rustc_hash::FxHashSet;
use serde_json::{Map, Value};

pub(super) struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub(super) fn parse_csv(source: &str) -> Result<TableData, String> {
    table_from_records(parse_csv_records(source)?, "csv-table")
}

pub(super) fn parse_json(source: &str) -> Result<TableData, String> {
    let value: Value = serde_json::from_str(source.trim())
        .map_err(|error| format!("json-table is not valid JSON: {error}"))?;
    match value {
        Value::Array(items) => parse_json_array(items),
        Value::Object(object) => parse_json_object(object),
        _ => Err(
            "json-table must be an array of objects, an array of arrays, or an object with headers and rows."
                .to_string(),
        ),
    }
}

fn table_from_records(records: Vec<Vec<String>>, kind: &str) -> Result<TableData, String> {
    if records.is_empty() {
        return Err(format!(
            "{kind} is empty. Add a header row or import a non-empty {kind} source."
        ));
    }
    let width = records.iter().map(Vec::len).max().unwrap_or(0);
    if width == 0 || records[0].iter().all(String::is_empty) {
        return Err(format!("{kind} header row is empty."));
    }
    let mut records = records.into_iter();
    let mut headers = records.next().unwrap_or_default();
    headers.resize(width, String::new());
    let rows = records
        .map(|mut row| {
            row.resize(width, String::new());
            row
        })
        .collect();
    Ok(TableData { headers, rows })
}

fn parse_csv_records(source: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut chars = source.chars().peekable();
    let mut line = 1usize;
    let mut in_quotes = false;
    let mut quote_line = 1usize;

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                if ch == '\n' {
                    line += 1;
                }
                field.push(ch);
            }
            continue;
        }
        match ch {
            '"' if field.is_empty() => {
                in_quotes = true;
                quote_line = line;
            }
            ',' => record.push(std::mem::take(&mut field)),
            '\n' => {
                finish_csv_record(&mut records, &mut record, &mut field);
                line += 1;
            }
            '\r' => {}
            _ => field.push(ch),
        }
    }
    if in_quotes {
        return Err(format!("csv-table has an unclosed quote starting at line {quote_line}."));
    }
    if !field.is_empty() || !record.is_empty() {
        finish_csv_record(&mut records, &mut record, &mut field);
    }
    Ok(records)
}

fn finish_csv_record(records: &mut Vec<Vec<String>>, record: &mut Vec<String>, field: &mut String) {
    record.push(std::mem::take(field));
    if record.iter().any(|cell| !cell.is_empty()) {
        records.push(std::mem::take(record));
    } else {
        record.clear();
    }
}

fn parse_json_array(items: Vec<Value>) -> Result<TableData, String> {
    if items.is_empty() {
        return Err("json-table array is empty.".to_string());
    }
    match items.first() {
        Some(Value::Object(_)) => objects_to_table(items),
        Some(Value::Array(_)) => arrays_to_table(items),
        _ => Err("json-table array items must be objects or arrays.".to_string()),
    }
}

fn parse_json_object(object: Map<String, Value>) -> Result<TableData, String> {
    let headers = object.get("headers").or_else(|| object.get("columns")).ok_or_else(|| {
        "json-table object must include a \"headers\" or \"columns\" array.".to_string()
    })?;
    let rows = object
        .get("rows")
        .ok_or_else(|| "json-table object must include a \"rows\" array.".to_string())?;
    let headers = json_string_list(headers, "headers")?;
    if headers.is_empty() {
        return Err("json-table header row is empty.".to_string());
    }
    Ok(TableData { headers: headers.clone(), rows: json_rows(rows, headers.len())? })
}

fn objects_to_table(items: Vec<Value>) -> Result<TableData, String> {
    let mut headers = Vec::new();
    let mut seen = FxHashSet::default();
    let mut maps = Vec::with_capacity(items.len());
    for (index, item) in items.into_iter().enumerate() {
        let Value::Object(map) = item else {
            return Err(format!("json-table item {index} must be an object."));
        };
        for key in map.keys() {
            if seen.insert(key.clone()) {
                headers.push(key.clone());
            }
        }
        maps.push(map);
    }
    if headers.is_empty() {
        return Err("json-table objects have no keys.".to_string());
    }
    let rows = maps
        .into_iter()
        .map(|map| {
            headers.iter().map(|key| json_cell(map.get(key).unwrap_or(&Value::Null))).collect()
        })
        .collect();
    Ok(TableData { headers, rows })
}

fn arrays_to_table(items: Vec<Value>) -> Result<TableData, String> {
    let mut records = Vec::with_capacity(items.len());
    for (index, item) in items.into_iter().enumerate() {
        let Value::Array(cells) = item else {
            return Err(format!("json-table item {index} must be an array."));
        };
        records.push(cells.iter().map(json_cell).collect());
    }
    table_from_records(records, "json-table")
}

fn json_rows(value: &Value, width: usize) -> Result<Vec<Vec<String>>, String> {
    let Value::Array(items) = value else {
        return Err("json-table \"rows\" must be an array.".to_string());
    };
    let mut rows = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let mut row: Vec<String> = match item {
            Value::Array(cells) => cells.iter().map(json_cell).collect(),
            _ => return Err(format!("json-table row {index} must be an array.")),
        };
        row.resize(width, String::new());
        rows.push(row);
    }
    Ok(rows)
}

fn json_string_list(value: &Value, field: &str) -> Result<Vec<String>, String> {
    let Value::Array(items) = value else {
        return Err(format!("json-table \"{field}\" must be an array."));
    };
    Ok(items.iter().map(json_cell).collect())
}

fn json_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}
