use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

mod iter;
mod options;
mod paths;
mod rename;
mod template;

use either::Either;
use iter::{Forward, Operation, Reverse};
use options::{ExecutionMode, Opts, SortMode};
use rename::Renamer;

use crate::iter::{DataTracker, MultimodeConflict};

fn main() {
    let mut opts = Opts::from_args();
    if let Err(e) = run(&mut opts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(opts: &mut Opts) -> anyhow::Result<()> {
    let paths = opts.paths.iter().flat_map(paths::extract);
    let from = sort_paths(opts.sort, paths)?;
    let mut renamer = Renamer::new(opts, Some(from.len()));
    let to: Vec<_> = from.iter().map(|x| renamer.rename(x)).collect();
    let operations = select_iteration_mode(&from, &to)?;

    match opts.execution {
        ExecutionMode::Copy => do_copy(operations)?,
        ExecutionMode::Move => do_rename(operations)?,
        ExecutionMode::Preview => preview(operations)?,
    }

    Ok(())
}

fn select_iteration_mode<'a, P: AsRef<Path> + 'a>(
    from: &'a [P],
    to: &'a [P],
) -> anyhow::Result<Either<Forward<'a, P>, Reverse<'a, P>>> {
    let mut data = DataTracker::new(from);

    let mut iteration = Forward::new(from, to);
    let forward_iteration_result = data.check_iteration(&mut iteration);
    if forward_iteration_result.is_ok() {
        iteration.reset();
        return Ok(Either::Left(iteration));
    }

    let mut iteration = Reverse::new(from, to);
    let reverse_iteration_result = data.check_iteration(&mut iteration);
    if reverse_iteration_result.is_ok() {
        iteration.reset();
        return Ok(Either::Right(iteration));
    }

    Err(anyhow::anyhow!(MultimodeConflict::new(
        forward_iteration_result.unwrap_err(),
        reverse_iteration_result.unwrap_err()
    )))
}

fn do_copy<'a>(operations: impl Iterator<Item = Operation<'a>>) -> io::Result<()> {
    let handle = io::stdout();
    let mut handle = handle.lock();
    let mut count = 0;

    for op in operations {
        fs::copy(op.from, op.to)?;
        format_op(&mut handle, &op)?;
        count += 1;
    }

    println!("Copied {} files", count);
    Ok(())
}

fn do_rename<'a>(operations: impl Iterator<Item = Operation<'a>>) -> io::Result<()> {
    let handle = io::stdout();
    let mut handle = handle.lock();
    let mut count = 0;

    for op in operations {
        fs::rename(op.from, op.to)?;
        format_op(&mut handle, &op)?;
        count += 1;
    }

    println!("Moved {} files", count);
    Ok(())
}

fn preview<'a>(operations: impl Iterator<Item = Operation<'a>>) -> io::Result<()> {
    let handle = io::stdout();
    let mut handle = handle.lock();
    let mut count = 0;

    for op in operations {
        format_op(&mut handle, &op)?;
        count += 1;
    }

    println!("Would rename {} files", count);
    Ok(())
}

fn format_op(writer: &mut io::StdoutLock, op: &Operation<'_>) -> io::Result<()> {
    const MAX_FORMATTED_LEN: usize = 80;

    let formatted = format!("{} -> {}", op.from.display(), op.to.display());
    if formatted.len() > MAX_FORMATTED_LEN {
        writeln!(writer, "{}\n -> {}", op.from.display(), op.to.display())
    } else {
        writeln!(writer, "{}", formatted)
    }
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
