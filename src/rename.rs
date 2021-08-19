use std::{
    fmt::{self, Display},
    iter,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::{
    options::Opts,
    template::{Segment, Template, TemplateParser},
};

#[derive(Debug)]
pub struct Renamer {
    idx: u32,
    count: Option<usize>,
    template: Template,
    pattern: Option<Regex>,
}

impl<'a> Renamer {
    pub fn new(options: &mut Opts, count: Option<usize>) -> Self {
        let parser = TemplateParser::new();
        Self {
            idx: options.start,
            count,
            template: parser.parse(&options.template),
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
            width: get_width(self.count),
            path,
            template: &self.template,
            pattern: self.pattern.as_ref(),
        }
    }
}

pub struct RenameContext<'a> {
    idx: u32,
    width: Option<usize>,
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
                Segment::Numeric(width) => write!(
                    f,
                    "{:0width$}",
                    self.idx,
                    width = width.max(&self.width.unwrap_or_default())
                )?,
                Segment::Filename(width) => self.format_filename(f, *width)?,
            }
        }
        Ok(())
    }
}

fn get_width(count: Option<usize>) -> Option<usize> {
    let count = count?;
    let mut witness_pairs = iter::successors(Some((1usize, 10usize)), |(width, witness)| {
        witness.checked_mul(10).map(|witness| (width + 1, witness))
    });

    witness_pairs.find_map(|witness| {
        if witness.1 > count {
            Some(witness.0)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::template::TemplateParser;

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

        let parser = TemplateParser::new();
        let mut renamer = super::Renamer {
            idx: 1,
            count: None,
            template: parser.parse("Fuzzy Bear {n:3}-{o:3} (original)"),
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

        let parser = TemplateParser::new();
        let mut renamer = super::Renamer {
            idx: 21,
            count: None,
            template: parser.parse("Fuzzy Bear {n:3}-{o:3} (original)"),
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

        let parser = TemplateParser::new();
        let mut renamer = super::Renamer {
            idx: 1,
            count: None,
            template: parser.parse("S05E{0:2} {f}"),
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

    #[test]
    fn get_width() {
        assert_eq!(Some(1), super::get_width(Some(1)));
        assert_eq!(Some(3), super::get_width(Some(300)));
        assert_eq!(Some(9), super::get_width(Some(987456321)));
    }
}
