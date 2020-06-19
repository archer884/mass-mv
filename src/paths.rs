use std::path::PathBuf;
use std::{fs, iter};

pub fn extract(path: impl AsRef<str>) -> Box<dyn Iterator<Item = PathBuf>> {
    let path = path.as_ref();
    match fs::metadata(path) {
        Ok(metadata) => literal_path(path, metadata),
        Err(_) => glob_pattern(path),
    }
}

fn literal_path(path: &str, metadata: fs::Metadata) -> Box<dyn Iterator<Item = PathBuf>> {
    if metadata.is_file() {
        return Box::new(iter::once(path.into()));
    }

    let paths = walkdir::WalkDir::new(path)
        .contents_first(true)
        .into_iter()
        .filter_entry(|entry| {
            entry
                .metadata()
                .map(|meta| meta.file_type().is_file())
                .unwrap_or_default()
        })
        .filter_map(|entry| entry.ok().map(|entry| entry.path().into()));

    Box::new(paths)
}

fn glob_pattern(path: &str) -> Box<dyn Iterator<Item = PathBuf>> {
    let paths = match glob::glob(path) {
        Ok(paths) => paths,
        Err(_) => return Box::new(iter::empty()),
    };

    let paths = paths.filter_map(|item| item.ok()).filter(|candidate| {
        candidate
            .metadata()
            .map(|meta| meta.file_type().is_file())
            .unwrap_or_default()
    });

    Box::new(paths)
}
