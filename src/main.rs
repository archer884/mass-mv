mod paths;
mod rename;
mod template;

use rename::Renamer;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::{fs, str};
use structopt::StructOpt;

#[derive(Copy, Clone, Debug)]
enum SortKind {
    Created,
    Modified,

    // Do we need this for anything!?
    Standard,
}

impl str::FromStr for SortKind {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "c" | "created" => Ok(SortKind::Created),
            "m" | "modified" => Ok(SortKind::Modified),
            "s" | "standard" => Ok(SortKind::Standard),

            _ => Err(io::Error::new(io::ErrorKind::Other, "Invalid sort")),
        }
    }
}

#[derive(Clone, Debug, StructOpt)]
struct Opt {
    /// Rename templates are used to replace the file stem using replacement
    /// tokens. Available replacement tokens include n and o to replace with
    /// the number and original filename.
    ///
    /// Use nn for [01, 02, ...] and nnn for [001, 002, ...] etc. The same
    /// thing works with filenames: oooo for "foobar" will cause "foob"
    /// to be included in the filename.
    ///
    /// Enclose replacement tokens in /{}, e.g. /{nnn}.
    /// Tokens include [0, n] (numeric) and [f, o] (filename).
    template: String,

    /// Paths (glob patterns or specific files) to be moved.
    paths: Vec<String>,

    /// Perform copy instead of rename
    #[structopt(short, long)]
    copy: bool,

    /// Perform rename
    #[structopt(short, long)]
    force: bool,

    /// Allows users to set an arbitrary starting point for numbering.
    #[structopt(short, long)]
    offset: Option<u32>,

    /// Set sorting type
    #[structopt(short, long)]
    sort: Option<SortKind>,
}

fn main() -> io::Result<()> {
    let Opt {
        template,
        paths,
        copy,
        force,
        offset,
        sort,
    } = Opt::from_args();

    let mut renamer = offset
        .map(|idx| Renamer::with_idx(&template, idx))
        .unwrap_or_else(|| Renamer::new(&template));

    let paths = paths.into_iter().flat_map(paths::extract);
    let paths = sort_paths(sort.unwrap_or(SortKind::Standard), paths)?;

    let new_paths = paths.iter().map(|x| renamer.rename(x));

    if force {
        do_rename(&paths, new_paths)?;
    } else if copy {
        do_copy(&paths, new_paths)?;
    } else {
        preview(&paths, new_paths)?;
    }

    Ok(())
}

fn do_rename(paths: &[PathBuf], new_paths: impl Iterator<Item = PathBuf>) -> io::Result<()> {
    let handle = io::stdout();
    let mut handle = handle.lock();
    let mut count = 0;

    for (from, to) in paths.iter().zip(new_paths) {
        format_op(&mut handle, from, &to)?;
        fs::rename(from, to)?;
        count += 1;
    }

    println!("Moved {} files", count);
    Ok(())
}

fn do_copy(paths: &[PathBuf], new_paths: impl Iterator<Item = PathBuf>) -> io::Result<()> {
    let handle = io::stdout();
    let mut handle = handle.lock();
    let mut count = 0;

    for (from, to) in paths.iter().zip(new_paths) {
        format_op(&mut handle, from, &to)?;
        fs::copy(from, to)?;
        count += 1;
    }

    println!("Copied {} files", count);
    Ok(())
}

fn preview(paths: &[PathBuf], new_paths: impl Iterator<Item = PathBuf>) -> io::Result<()> {
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

fn sort_paths<'a>(
    sort: SortKind,
    paths: impl Iterator<Item = PathBuf>,
) -> io::Result<Vec<PathBuf>> {
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
        SortKind::Created => {
            let mut with_meta = collect_with_meta(paths, |x| x.created())?;
            with_meta.sort_unstable_by_key(|x| x.1);
            Ok(with_meta.into_iter().map(|x| x.0).collect())
        }

        SortKind::Modified => {
            let mut with_meta = collect_with_meta(paths, |x| x.modified())?;
            with_meta.sort_unstable_by_key(|x| x.1);
            Ok(with_meta.into_iter().map(|x| x.0).collect())
        }

        SortKind::Standard => {
            let mut paths: Vec<_> = paths.collect();
            paths.sort_unstable();
            Ok(paths)
        }
    }
}
