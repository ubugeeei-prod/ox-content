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

    fn planned(original: &str, edits: &[(&[&str], Value)]) -> String {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        fs::write(&file, original).unwrap();
        plan(&file, edits).unwrap().content
    }

    #[test]
    fn missing_files_start_from_an_empty_object() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nested/settings.json");
        assert_eq!(read_optional(&file).unwrap(), None);
        let plan = plan(&file, &[(&["a", "b", "c"], json!(1)), (&["flag"], json!(true))]).unwrap();
        assert_eq!(plan.original, None);
        assert_eq!(
            parse(Some(&plan.content)).unwrap(),
            json!({"a": {"b": {"c": 1}}, "flag": true})
        );
        assert!(plan.content.ends_with("}\n"));
        assert!(!file.exists(), "planning never writes");
        write(&plan).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), plan.content);
        assert_eq!(fs::read_dir(file.parent().unwrap()).unwrap().count(), 1);
        assert_eq!(parse(None).unwrap(), json!({}));
    }

    #[test]
    fn satisfied_edits_leave_the_text_byte_for_byte() {
        for original in [
            "{\"enabled\":true}\n",
            "{\n\t// keep\n\t\"enabled\" : true , /* odd spacing */\n}\n",
            "{ \"a\": { \"b\": [1, 2] }, \"enabled\": true }\n",
        ] {
            let edits: &[(&[&str], Value)] = &[(&["enabled"], json!(true))];
            assert_eq!(planned(original, edits), original);
        }
        assert_eq!(
            planned("{\"a\":{\"b\":[1,2]}}\n", &[(&["a", "b"], json!([1, 2]))]),
            "{\"a\":{\"b\":[1,2]}}\n"
        );
        // Only the missing final newline is added.
        assert_eq!(
            planned("{\"enabled\":true}", &[(&["enabled"], json!(true))]),
            "{\"enabled\":true}\n"
        );
        assert_eq!(planned("{}\n", &[]), "{}\n");
    }

    #[test]
    fn edits_replace_values_in_place_and_keep_everything_else() {
        let original = "{\n  // first\n  \"keep\": [1, 2],\n  \"mode\": \"old\", // trailing\n  \"nested\": { \"inner\": 1 },\n}\n";
        let content = planned(
            original,
            &[
                (&["mode"], json!("new")),
                (&["nested", "inner"], json!(2)),
                (&["nested", "added"], json!(false)),
                (&["list"], json!(["a", {"b": 1.5}])),
            ],
        );
        for kept in ["// first", "// trailing", "\"keep\": [1, 2]"] {
            assert!(content.contains(kept), "Lost {kept}: {content}");
        }
        assert_eq!(
            parse(Some(&content)).unwrap(),
            json!({
                "keep": [1, 2],
                "mode": "new",
                "nested": {"inner": 2, "added": false},
                "list": ["a", {"b": 1.5}]
            })
        );
        // Applying the same edits to the result changes nothing further.
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        fs::write(&file, &content).unwrap();
        let again =
            plan(&file, &[(&["mode"], json!("new")), (&["nested", "inner"], json!(2))]).unwrap();
        assert_eq!(again.original.as_deref(), Some(again.content.as_str()));
    }

    #[test]
    fn non_object_documents_are_rejected_with_their_path() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        for content in ["[]", "null", "42", "\"text\"", "{\"a\":}", "{} trailing", ""] {
            fs::write(&file, content).unwrap();
            let error = plan(&file, &[(&["enabled"], json!(true))]).err().unwrap().to_string();
            assert!(error.starts_with("Invalid configuration"), "{content:?}: {error}");
            assert!(error.contains("settings.json"), "{content:?}: {error}");
        }
        assert!(read_optional(dir.path()).is_err(), "a directory is not a missing file");
    }

    #[test]
    fn merges_back_up_the_original_and_unchanged_plans_write_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        fs::write(&file, "{\"enabled\":false}\n").unwrap();
        let names = || {
            let mut names: Vec<_> = fs::read_dir(dir.path())
                .unwrap()
                .map(|entry| entry.unwrap().file_name().into_string().unwrap())
                .collect();
            names.sort();
            names
        };
        let unchanged = plan(&file, &[(&["enabled"], json!(false))]).unwrap();
        write(&unchanged).unwrap();
        assert_eq!(names(), ["settings.json"]);
        let merge = plan(&file, &[(&["enabled"], json!(true))]).unwrap();
        write(&merge).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "{\"enabled\":true}\n");
        let names = names();
        assert_eq!(names.len(), 2, "{names:?}");
        let backup = names.iter().find(|name| name.as_str() != "settings.json").unwrap();
        assert!(backup.starts_with("settings.json.oxct-") && backup.ends_with(".bak"), "{backup}");
        assert_eq!(fs::read_to_string(dir.path().join(backup)).unwrap(), "{\"enabled\":false}\n");
    }

    #[test]
    fn files_that_appear_or_vanish_after_planning_are_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        let create = plan(&file, &[(&["enabled"], json!(true))]).unwrap();
        fs::write(&file, "{\"mine\":1}\n").unwrap();
        assert!(write(&create).unwrap_err().to_string().contains("changed during setup"));
        assert_eq!(fs::read_to_string(&file).unwrap(), "{\"mine\":1}\n");
        let merge = plan(&file, &[(&["enabled"], json!(true))]).unwrap();
        fs::remove_file(&file).unwrap();
        assert!(write(&merge).unwrap_err().to_string().contains("changed during setup"));
        assert!(!file.exists());
        assert_eq!(
            fs::read_dir(dir.path()).unwrap().count(),
            0,
            "no backup or temporary file is left"
        );
    }

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
