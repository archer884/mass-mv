use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process,
};

mod options;
mod paths;
mod rename;
mod template;

use options::{ExecutionMode, Options, SortMode};
use rename::Renamer;

fn main() -> io::Result<()> {
    let mut options = Options::from_args();
    let mut renamer = Renamer::from_options(&mut options);

    let paths = options.paths.iter().flat_map(paths::extract);
    let paths = sort_paths(options.sort, paths)?;
    let new_paths: Vec<_> = paths.iter().map(|x| renamer.rename(x)).collect();

    if let Some(conflict) = has_conflict(&paths, &new_paths) {
        eprintln!(
            "Move operation would result in data loss:\n\n    {}",
            conflict.display()
        );
        process::exit(1);
    }

    match options.execution {
        ExecutionMode::Copy => do_copy(&paths, new_paths)?,
        ExecutionMode::Move => do_rename(&paths, new_paths)?,
        ExecutionMode::Preview => preview(&paths, new_paths)?,
    }

    Ok(())
}

fn has_conflict<'a, P: AsRef<Path> + 'a>(paths: &'a [P], new_paths: &'a [P]) -> Option<&'a Path> {
    use std::collections::HashMap;

    let existing_paths: HashMap<_, _> = paths
        .iter()
        .enumerate()
        .map(|(idx, x)| (x.as_ref(), idx))
        .collect();

    new_paths
        .iter()
        .enumerate()
        .filter_map(|(idx, x)| {
            let &existing_idx = existing_paths.get(x.as_ref())?;
            if idx < existing_idx {
                Some(x.as_ref())
            } else {
                None
            }
        })
        .next()
}

fn do_rename(paths: &[PathBuf], new_paths: impl IntoIterator<Item = PathBuf>) -> io::Result<()> {
    let handle = io::stdout();
    let mut handle = handle.lock();
    let mut count = 0;

    for (from, to) in paths.iter().zip(new_paths) {
        fs::rename(from, &to)?;
        format_op(&mut handle, from, &to)?;
        count += 1;
    }

    println!("Moved {} files", count);
    Ok(())
}

fn do_copy(paths: &[PathBuf], new_paths: impl IntoIterator<Item = PathBuf>) -> io::Result<()> {
    let handle = io::stdout();
    let mut handle = handle.lock();
    let mut count = 0;

    for (from, to) in paths.iter().zip(new_paths) {
        fs::copy(from, &to)?;
        format_op(&mut handle, from, &to)?;
        count += 1;
    }

    println!("Copied {} files", count);
    Ok(())
}

fn preview(paths: &[PathBuf], new_paths: impl IntoIterator<Item = PathBuf>) -> io::Result<()> {
    let handle = io::stdout();
    let mut handle = handle.lock();
    let mut count = 0;

    for (from, to) in paths.iter().zip(new_paths) {
        format_op(&mut handle, from, &to)?;
        count += 1;
    }

    println!("Would rename {} files", count);
    Ok(())
}

fn format_op(writer: &mut io::StdoutLock, from: &Path, to: &Path) -> io::Result<()> {
    writeln!(writer, "{}\n -> {}", from.display(), to.display())
}

fn sort_paths(sort: SortMode, paths: impl Iterator<Item = PathBuf>) -> io::Result<Vec<PathBuf>> {
    use std::fs::Metadata;
    use std::time::SystemTime;

    fn collect_with_meta(
        paths: impl Iterator<Item = PathBuf>,
        extractor: impl Fn(Metadata) -> io::Result<SystemTime>,
    ) -> io::Result<Vec<(PathBuf, SystemTime)>> {
        paths
            .map(|x| x.metadata().and_then(&extractor).map(|y| (x, y)))
            .collect()
    }

    match sort {
        SortMode::Created => {
            let mut with_meta = collect_with_meta(paths, |x| x.created())?;
            with_meta.sort_unstable_by_key(|x| x.1);
            Ok(with_meta.into_iter().map(|x| x.0).collect())
        }

        SortMode::Modified => {
            let mut with_meta = collect_with_meta(paths, |x| x.modified())?;
            with_meta.sort_unstable_by_key(|x| x.1);
            Ok(with_meta.into_iter().map(|x| x.0).collect())
        }

        SortMode::Path => {
            let mut paths: Vec<_> = paths.collect();
            paths.sort_unstable();
            Ok(paths)
        }
    }
}
