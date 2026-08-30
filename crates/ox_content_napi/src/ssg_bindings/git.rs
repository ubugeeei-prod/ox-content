use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use napi_derive::napi;

use crate::JsGitContributor;

type ContributorCache = Mutex<HashMap<(String, String), Vec<JsGitContributor>>>;
type LastmodCache = Mutex<HashMap<(String, String), Arc<HashMap<String, f64>>>>;
type HeadCache = Mutex<HashMap<String, (std::time::Instant, Option<String>)>>;

/// How long a HEAD reading is reused before git is asked again.
const HEAD_FRESHNESS: std::time::Duration = std::time::Duration::from_secs(1);

fn head_cache() -> &'static HeadCache {
    static CACHE: OnceLock<HeadCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// HEAD for `root`, re-read at most once a second.
///
/// `git rev-parse HEAD` is a process spawn, and it keys every cache lookup
/// here — so without this the caches still paid one spawn per file, which is
/// most of what they exist to avoid. A build finishes well inside the window;
/// a dev server picks up a new commit within a second of it landing.
fn cached_git_head(root: &std::path::Path) -> Option<String> {
    let key = root.to_string_lossy().into_owned();
    let now = std::time::Instant::now();

    if let Some((read_at, head)) =
        head_cache().lock().unwrap_or_else(PoisonError::into_inner).get(&key)
        && now.duration_since(*read_at) < HEAD_FRESHNESS
    {
        return head.clone();
    }

    let head = git_head(root);
    head_cache().lock().unwrap_or_else(PoisonError::into_inner).insert(key, (now, head.clone()));
    head
}

fn lastmod_cache() -> &'static LastmodCache {
    static CACHE: OnceLock<LastmodCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Last commit time, in seconds, for every file under `root` that git knows.
///
/// This used to be one `git log -1 -- <path>` per file. A page git has never
/// seen is the expensive case, not the cheap one: git cannot answer "never"
/// without walking the whole history, so every new page cost a full walk plus
/// a process spawn. A 400-page site spent 20 s in the sitemap step against
/// 0.6 s for the rest of the build. One walk for the whole subtree costs about
/// half a second here and does not grow with the page count.
///
/// `-z` with a NUL-prefixed timestamp makes the stream unambiguous for any
/// path bytes: an empty field means the next field is a timestamp, and
/// everything else is a path belonging to the timestamp before it.
fn build_lastmod_map(root: &std::path::Path) -> HashMap<String, f64> {
    let mut map = HashMap::new();

    let Ok(output) = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-z", "--format=%x00%ct", "--name-only", "--relative", "--", "."])
        .output()
    else {
        return map;
    };
    if !output.status.success() {
        return map;
    }

    let mut seconds: Option<f64> = None;
    let mut next_is_timestamp = false;
    for field in output.stdout.split(|byte| *byte == 0) {
        if field.is_empty() {
            next_is_timestamp = true;
            continue;
        }
        if next_is_timestamp {
            next_is_timestamp = false;
            seconds = std::str::from_utf8(field).ok().and_then(|value| value.trim().parse().ok());
            continue;
        }
        let Some(seconds) = seconds else {
            continue;
        };
        // The first path of a commit carries the newline that ended the
        // format line.
        let path = field.strip_prefix(b"\n").unwrap_or(field);
        let Ok(path) = std::str::from_utf8(path) else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        // git walks newest first, so the first time a path appears is its
        // last commit.
        map.entry(path.to_string()).or_insert(seconds);
    }

    map
}

fn lastmod_map(root: &std::path::Path) -> Option<Arc<HashMap<String, f64>>> {
    // HEAD keys the cache the way it keys the contributor cache: a build never
    // moves it, and a dev server that outlives a commit picks up the new one.
    let head = cached_git_head(root)?;
    let key = (root.to_string_lossy().into_owned(), head);

    if let Some(hit) = lastmod_cache().lock().unwrap_or_else(PoisonError::into_inner).get(&key) {
        return Some(Arc::clone(hit));
    }

    let map = Arc::new(build_lastmod_map(root));
    lastmod_cache().lock().unwrap_or_else(PoisonError::into_inner).insert(key, Arc::clone(&map));
    Some(map)
}

/// Returns the last git commit timestamp for a file in milliseconds.
#[napi]
pub fn get_git_last_updated(file_path: String, root: Option<String>) -> Option<f64> {
    let root = root.filter(|root| !root.is_empty()).map(PathBuf::from)?;
    let pathspec = git_pathspec(&file_path, &root);
    let seconds = *lastmod_map(&root)?.get(pathspec)?;
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
    let head = cached_git_head(&root);
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
