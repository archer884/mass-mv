mod paths;
mod rename;
mod template;

use rename::Renamer;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

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
    start: Option<u32>,
}

fn main() -> io::Result<()> {
    let Opt {
        template,
        paths,
        copy,
        force,
        start,
    } = Opt::from_args();

    let mut renamer = start
        .map(|idx| Renamer::with_idx(&template, idx))
        .unwrap_or_else(|| Renamer::new(&template));
    
    let mut paths: Vec<_> = paths.into_iter().flat_map(paths::extract).collect();
    paths.sort();

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
