use crate::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub(crate) struct Workspace {
    pub(crate) root: PathBuf,
    pub(crate) theme_colors_dir: PathBuf,
    pub(crate) theme_skins_dir: PathBuf,
    pub(crate) version: String,
}

impl Workspace {
    pub(crate) fn discover() -> Result<Self> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.join("../..").canonicalize()?;
        let scripts_dir = root.join("tools/scripts");
        let version = package_version(&root.join("npm/vite-plugin-ox-content/package.json"))?;

        Ok(Self {
            theme_colors_dir: scripts_dir.join("theme-colors"),
            theme_skins_dir: scripts_dir.join("theme-skins"),
            root,
            version,
        })
    }

    pub(crate) fn read_json<T: DeserializeOwned>(&self, path: impl AsRef<Path>) -> Result<T> {
        let text = fs::read_to_string(path.as_ref())?;
        Ok(serde_json::from_str(&text)?)
    }

    pub(crate) fn read_to_string(&self, path: impl AsRef<Path>) -> Result<String> {
        Ok(fs::read_to_string(path.as_ref())?)
    }

    pub(crate) fn write(&self, path: impl AsRef<Path>, content: impl AsRef<str>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content.as_ref())?;
        Ok(())
    }

    pub(crate) fn write_json<T: Serialize>(&self, path: impl AsRef<Path>, value: &T) -> Result<()> {
        let mut output = serde_json::to_string_pretty(value)?;
        output.push('\n');
        self.write(path, output)
    }

    pub(crate) fn remove_dir(&self, path: impl AsRef<Path>) -> Result<()> {
        match fs::remove_dir_all(path.as_ref()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(serde::Deserialize)]
struct PackageVersion {
    version: String,
}

fn package_version(path: &Path) -> Result<String> {
    let manifest = fs::read_to_string(path)?;
    Ok(serde_json::from_str::<PackageVersion>(&manifest)?.version)
}
