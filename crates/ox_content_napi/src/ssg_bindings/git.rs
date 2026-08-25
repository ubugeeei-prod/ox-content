use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, PoisonError};

use napi_derive::napi;

use crate::JsGitContributor;

type ContributorCache = Mutex<HashMap<(String, String), Vec<JsGitContributor>>>;

/// Returns the last git commit timestamp for a file in milliseconds.
#[napi]
pub fn get_git_last_updated(file_path: String, root: Option<String>) -> Option<f64> {
    let root = root.map(PathBuf::from)?;
    let file = PathBuf::from(&file_path);
    let pathspec = file.strip_prefix(&root).ok().and_then(|p| p.to_str()).unwrap_or(&file_path);
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-1", "--format=%ct", "--"])
        .arg(pathspec)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let seconds = String::from_utf8(output.stdout).ok()?.trim().parse::<f64>().ok()?;
    Some(seconds * 1_000.0)
}

fn contributors_cache() -> &'static ContributorCache {
    static CACHE: OnceLock<ContributorCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn git_head(root: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let head = String::from_utf8(output.stdout).ok()?;
    let head = head.trim();
    if head.is_empty() { None } else { Some(head.to_string()) }
}

fn git_pathspec<'a>(file_path: &'a str, root: &std::path::Path) -> &'a str {
    let file = std::path::Path::new(file_path);
    file.strip_prefix(root).ok().and_then(|path| path.to_str()).unwrap_or(file_path)
}

fn parse_git_contributors(stdout: &[u8]) -> Vec<JsGitContributor> {
    let Ok(text) = std::str::from_utf8(stdout) else {
        return Vec::new();
    };
    let mut order: Vec<JsGitContributor> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let (name, email) = match line.split_once('\t') {
            Some((name, email)) => (name.trim(), email.trim()),
            None => (line.trim(), ""),
        };
        if name.is_empty() {
            continue;
        }
        let key = if email.is_empty() {
            format!("name:{}", name.to_ascii_lowercase())
        } else {
            format!("email:{}", email.to_ascii_lowercase())
        };
        if let Some(&existing) = index.get(&key) {
            let commits = order[existing].commits.unwrap_or(0);
            order[existing].commits = Some(commits.saturating_add(1));
            continue;
        }
        index.insert(key, order.len());
        order.push(JsGitContributor {
            name: name.to_string(),
            email: if email.is_empty() { None } else { Some(email.to_string()) },
            commits: Some(1),
        });
    }
    order
}

/// Returns unique git authors for a file. Empty when git is missing or fails.
#[napi]
pub fn get_git_contributors(file_path: String, root: Option<String>) -> Vec<JsGitContributor> {
    let Some(root) = root.filter(|root| !root.is_empty()).map(PathBuf::from) else {
        return Vec::new();
    };
    let spec = git_pathspec(&file_path, &root).to_string();
    let head = git_head(&root);
    if let Some(head) = head.as_ref() {
        let cache = contributors_cache().lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(hit) = cache.get(&(spec.clone(), head.clone())) {
            return hit.clone();
        }
    }

    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["log", "--format=%an%x09%ae", "--"])
        .arg(&spec)
        .output();
    let contributors = match output {
        Ok(output) if output.status.success() => parse_git_contributors(&output.stdout),
        _ => Vec::new(),
    };

    if let Some(head) = head {
        let mut cache = contributors_cache().lock().unwrap_or_else(PoisonError::into_inner);
        cache.insert((spec, head), contributors.clone());
    }
    contributors
}
