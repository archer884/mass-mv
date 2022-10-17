use clap::Parser;
use regex::Regex;

#[derive(Copy, Clone, Debug)]
pub enum SortMode {
    /// Sort by created date
    Created,

    /// Sort by modified date
    Modified,

    /// Sort by path (default)
    Path,
}

#[derive(Copy, Clone, Debug)]
pub enum ExecutionMode {
    Copy,
    Move,
    Preview,
}

#[derive(Clone, Debug)]
pub struct Args {
    pub template: String,
    pub paths: Vec<String>,
    pub pattern: Option<Regex>,
    pub start: u32,
    pub execution: ExecutionMode,
    pub sort: SortMode,
}

impl Args {
    pub fn parse() -> Self {
        use clap::ArgGroup;

        #[derive(Clone, Debug, Parser)]
        struct Template {
            /// Rename templates are used to replace the file stem using replacement tokens. Available replacement tokens include n and o to replace with the number and original filename.
            ///
            /// Use n:2 for [01, 02, ...] and n:3 for [001, 002, ...] etc. The same thing works with filenames: o:4 for "foobar" will cause "foob" to be included in the filename.
            ///
            /// Enclose replacement tokens in {}, e.g. {n}. Tokens include [0, n] (numeric) and [f, o] (filename).
            template: String,

            /// Paths (glob patterns or specific files) to be moved
            paths: Vec<String>,

            /// Use a regular expression to select part of the original filename.
            ///
            /// If the provided regular expression includes a capture group, the content of the capture group will be used. Otherwise, replacement templates will make use of the whole match.
            #[structopt(long)]
            pattern: Option<Regex>,

            /// Start numbering at something other than 1.
            #[structopt(short, long)]
            start: Option<u32>,

            #[command(flatten)]
            execution_opts: ExecutionOptions,

            #[command(flatten)]
            sort_opts: SortOptions,
        }

        #[derive(Clone, Debug, Parser)]
        struct ExecutionOptions {
            /// Copy files
            #[structopt(long)]
            copy: bool,

            /// Rename files
            #[structopt(short, long)]
            force: bool,
        }

        impl ExecutionOptions {
            fn into_enum(self) -> ExecutionMode {
                if self.copy {
                    ExecutionMode::Copy
                } else if self.force {
                    ExecutionMode::Move
                } else {
                    ExecutionMode::Preview
                }
            }
        }

        #[derive(Clone, Debug, Parser)]
        #[command(group = ArgGroup::new("sort"))]
        struct SortOptions {
            /// Sort files by created date when renaming.
            #[structopt(short, long, group = "sort")]
            created: bool,

            /// Sort files by modified date when renaming.
            #[structopt(short, long, group = "sort")]
            modified: bool,

            /// Sort files by path when renaming. (Default)
            #[structopt(short, long, group = "sort")]
            path: bool,
        }

        impl SortOptions {
            fn into_enum(self) -> SortMode {
                if self.created {
                    SortMode::Created
                } else if self.modified {
                    SortMode::Modified
                } else {
                    SortMode::Path
                }
            }
        }

        let Template {
            template,
            paths,
            pattern,
            start,
            execution_opts,
            sort_opts,
        } = Parser::parse();

        Args {
            template,
            paths,
            pattern,
            start: start.unwrap_or(1),
            execution: execution_opts.into_enum(),
            sort: sort_opts.into_enum(),
        }
    }
}
