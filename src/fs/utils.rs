use std::path::Path;

pub fn is_hidden(path: &Path) -> bool {
    path.file_name().unwrap().to_string_lossy().starts_with('.')
}
