use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn temp_root() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("ox-content-docs-graph-{nanos}-{seq}"))
}

pub(super) fn write_external_package(root: &Path, name: &str, declaration: &str) {
    let package_root = root.join("node_modules").join(name);
    fs::create_dir_all(package_root.join("lib")).unwrap();
    let package_json = format!(
        r#"{{
  "name": "{name}",
  "type": "module",
  "exports": {{
    ".": {{
      "types": "./lib/index.d.ts",
      "default": "./lib/index.js"
    }}
  }}
}}"#
    );
    fs::write(package_root.join("package.json"), package_json).unwrap();
    fs::write(package_root.join("lib/index.d.ts"), declaration).unwrap();
}
