use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::{
    options::Opts,
    template::{Segment, Template},
};

#[derive(Debug)]
pub struct Renamer {
    idx: u32,
    template: Template,
    pattern: Option<Regex>,
}

impl<'a> Renamer {
    pub fn from_options(options: &mut Opts) -> Self {
        Self {
            idx: options.start,
            template: Template::new(&options.template),
            pattern: options.pattern.take(),
        }
    }

    pub fn rename(&mut self, path: &Path) -> PathBuf {
        let stem = self.context(path).to_string();
        let mut result = path.with_file_name(stem);

        if let Some(extension) = path.extension() {
            result.set_extension(extension);
        }

        self.idx += 1;
        result
    }

    fn context<'p>(&'p self, path: &'p Path) -> RenameContext {
        RenameContext {
            idx: self.idx,
            path,
            template: &self.template,
            pattern: self.pattern.as_ref(),
        }
    }
}

pub struct RenameContext<'a> {
    idx: u32,
    path: &'a Path,
    template: &'a Template,
    pattern: Option<&'a Regex>,
}

impl RenameContext<'_> {
    fn format_filename(&self, f: &mut fmt::Formatter, width: usize) -> fmt::Result {
        let name = self
            .path
            .file_stem()
            .expect("Must be a filename")
            .to_string_lossy();

        let name = self.extract_name(&name);
        match width {
            1 => f.write_str(name),
            n => f.write_str(&name[..n]),
        }
    }

    fn extract_name<'a>(&self, text: &'a str) -> &'a str {
        self.pattern
            .and_then(|x| x.captures(text))
            .and_then(|x| x.get(1).or_else(|| x.get(0)))
            .map_or(text, |x| x.as_str())
    }
}

impl Display for RenameContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in self.template.segments() {
            match segment {
                Segment::Literal(s) => f.write_str(s)?,
                Segment::Numeric(width) => write!(f, "{:0width$}", self.idx, width = width)?,
                Segment::Filename(width) => self.format_filename(f, *width)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn rename_works() {
        let files = &[
            "f4240.jpg",
            "f4241.jpg",
            "f4242.jpg",
            "f4243.jpg",
            "f4244.jpg",
            "f4245.jpg",
            "f4246.jpg",
            "f4247.jpg",
            "f4248.jpg",
            "f4249.jpg",
        ];

        let expected = &[
            Path::new("Fuzzy Bear 001-f42 (original).jpg"),
            Path::new("Fuzzy Bear 002-f42 (original).jpg"),
            Path::new("Fuzzy Bear 003-f42 (original).jpg"),
            Path::new("Fuzzy Bear 004-f42 (original).jpg"),
            Path::new("Fuzzy Bear 005-f42 (original).jpg"),
            Path::new("Fuzzy Bear 006-f42 (original).jpg"),
            Path::new("Fuzzy Bear 007-f42 (original).jpg"),
            Path::new("Fuzzy Bear 008-f42 (original).jpg"),
            Path::new("Fuzzy Bear 009-f42 (original).jpg"),
            Path::new("Fuzzy Bear 010-f42 (original).jpg"),
        ];

        let mut renamer = super::Renamer {
            idx: 1,
            template: super::Template::new("Fuzzy Bear {{nnn}}-{{ooo}} (original)"),
            pattern: None,
        };

        let actual = files
            .into_iter()
            .cloned()
            .map(|x| renamer.rename(x.as_ref()));

        for (actual, &expected) in actual.zip(expected) {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn rename_works_with_offset_idx() {
        let files = &[
            "f4240.jpg",
            "f4241.jpg",
            "f4242.jpg",
            "f4243.jpg",
            "f4244.jpg",
            "f4245.jpg",
            "f4246.jpg",
            "f4247.jpg",
            "f4248.jpg",
            "f4249.jpg",
        ];

        let expected = &[
            Path::new("Fuzzy Bear 021-f42 (original).jpg"),
            Path::new("Fuzzy Bear 022-f42 (original).jpg"),
            Path::new("Fuzzy Bear 023-f42 (original).jpg"),
            Path::new("Fuzzy Bear 024-f42 (original).jpg"),
            Path::new("Fuzzy Bear 025-f42 (original).jpg"),
            Path::new("Fuzzy Bear 026-f42 (original).jpg"),
            Path::new("Fuzzy Bear 027-f42 (original).jpg"),
            Path::new("Fuzzy Bear 028-f42 (original).jpg"),
            Path::new("Fuzzy Bear 029-f42 (original).jpg"),
            Path::new("Fuzzy Bear 030-f42 (original).jpg"),
        ];

        let mut renamer = super::Renamer {
            idx: 21,
            template: super::Template::new("Fuzzy Bear {{nnn}}-{{ooo}} (original)"),
            pattern: None,
        };

        let actual = files
            .into_iter()
            .cloned()
            .map(|x| renamer.rename(x.as_ref()));

        for (actual, &expected) in actual.zip(expected) {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn rename_works_with_filename_pattern() {
        let files = &[
            "Highlander S05E01 Prophecy.mp4",
            "Highlander S04E22 One Minute to Midnight.mp4",
        ];

        let expected = &[
            Path::new("S05E01 Prophecy.mp4"),
            Path::new("S05E02 One Minute to Midnight.mp4"),
        ];

        let mut renamer = super::Renamer {
            idx: 1,
            template: super::Template::new("S05E{{00}} {{f}}"),
            pattern: regex::Regex::new(r#".*S\d\dE\d\d (.+)"#).ok(),
        };

        let actual = files
            .into_iter()
            .cloned()
            .map(|x| renamer.rename(x.as_ref()));

        for (actual, &expected) in actual.zip(expected) {
            assert_eq!(actual, expected);
        }
    }
}
