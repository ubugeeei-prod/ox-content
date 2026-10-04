use crate::Result;
use jsonc_parser::{
    ParseOptions,
    cst::{CstInputValue, CstRootNode},
};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub struct Plan {
    pub file: PathBuf,
    pub original: Option<String>,
    pub content: String,
}

pub fn read_optional(file: &Path) -> Result<Option<String>> {
    match fs::read_to_string(file) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub fn parse(content: Option<&str>) -> Result<Value> {
    Ok(jsonc_parser::parse_to_serde_value::<Value>(
        content.unwrap_or("{}"),
        &ParseOptions::default(),
    )?)
}

pub fn plan(file: &Path, edits: &[(&[&str], Value)]) -> Result<Plan> {
    let original = read_optional(file)?;
    let content = original.as_deref().unwrap_or("{}\n");
    let value = parse(Some(content))
        .map_err(|error| format!("Invalid configuration: {}. {error}", file.display()))?;
    if !value.is_object() {
        return Err(format!("Invalid configuration: {}", file.display()).into());
    }
    let root = CstRootNode::parse(content, &ParseOptions::default())?;
    let mut changed = false;
    for (path, next) in edits {
        let Some((key, parents)) = path.split_last() else {
            continue;
        };
        let mut existing = &value;
        for key in *path {
            existing = &existing[*key];
        }
        if existing == next {
            continue;
        }
        changed = true;
        let mut object = root.object_value_or_set();
        for parent in parents {
            object = object.object_value_or_set(parent);
        }
        if let Some(property) = object.get(key) {
            property.set_value(input(next));
        } else {
            object.append(key, input(next));
        }
    }
    let mut content = if changed { root.to_string() } else { content.to_string() };
    if !content.ends_with('\n') {
        content.push('\n');
    }
    Ok(Plan { file: file.to_path_buf(), original, content })
}

fn input(value: &Value) -> CstInputValue {
    match value {
        Value::Null => CstInputValue::Null,
        Value::Bool(value) => CstInputValue::Bool(*value),
        Value::Number(value) => CstInputValue::Number(value.to_string()),
        Value::String(value) => CstInputValue::String(value.clone()),
        Value::Array(values) => CstInputValue::Array(values.iter().map(input).collect()),
        Value::Object(values) => CstInputValue::Object(
            values.iter().map(|(key, value)| (key.clone(), input(value))).collect(),
        ),
    }
}

pub fn write(plan: &Plan) -> Result<()> {
    if plan.original.as_deref() == Some(&plan.content) {
        return Ok(());
    }
    let parent = plan.file.parent().ok_or("Configuration needs a parent directory")?;
    fs::create_dir_all(parent)?;
    let check = || -> Result<()> {
        if read_optional(&plan.file)? != plan.original {
            return Err(format!(
                "Configuration changed during setup: {}. Run setup again.",
                plan.file.display()
            )
            .into());
        }
        Ok(())
    };
    check()?;
    if let Some(original) = &plan.original {
        let name = plan.file.file_name().ok_or("Missing configuration filename")?.to_string_lossy();
        let mut backup = tempfile::Builder::new()
            .prefix(&format!("{name}.oxct-"))
            .suffix(".bak")
            .tempfile_in(parent)?;
        backup.write_all(original.as_bytes())?;
        backup.keep()?;
        let permissions = fs::metadata(&plan.file)?.permissions();
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        temp.as_file().set_permissions(permissions)?;
        temp.write_all(plan.content.as_bytes())?;
        check()?;
        temp.persist(&plan.file)?;
    } else {
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        temp.write_all(plan.content.as_bytes())?;
        temp.persist_noclobber(&plan.file)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn refuses_malformed_and_concurrent_changes() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        fs::write(&file, "{\"broken\":").unwrap();
        assert!(plan(&file, &[(&["enabled"], json!(true))]).is_err());
        fs::write(&file, "{}\n").unwrap();
        let plan = plan(&file, &[(&["enabled"], json!(true))]).unwrap();
        fs::write(&file, "{\"concurrent\":true}\n").unwrap();
        assert!(write(&plan).unwrap_err().to_string().contains("changed during setup"));
        assert_eq!(fs::read_to_string(&file).unwrap(), "{\"concurrent\":true}\n");
    }
}
