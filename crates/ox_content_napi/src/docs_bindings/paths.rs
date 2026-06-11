use std::path::Path;

pub(super) fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
