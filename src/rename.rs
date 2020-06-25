use crate::template::{Segment, Template};
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

pub struct Renamer {
    idx: u32,
    template: Template,
}

impl Renamer {
    pub fn new(template: &str) -> Self {
        Self {
            idx: 1,
            template: Template::new(template),
        }
    }

    pub fn with_idx(template: &str, idx: u32) -> Self {
        Self {
            idx,
            template: Template::new(template),
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

    fn context<'a>(&'a self, path: &'a Path) -> RenameContext {
        RenameContext {
            idx: self.idx,
            path,
            template: &self.template,
        }
    }
}

pub struct RenameContext<'a> {
    idx: u32,
    path: &'a Path,
    template: &'a Template,
}

impl Display for RenameContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in self.template.segments() {
            match segment {
                Segment::Literal(s) => f.write_str(s)?,
                Segment::Numeric(width) => write!(f, "{:0width$}", self.idx, width = width)?,
                Segment::Filename(width) => format_filename(f, self.path, *width)?,
            }
        }
        Ok(())
    }
}

fn format_filename(f: &mut fmt::Formatter, path: &Path, width: usize) -> fmt::Result {
    let name = path
        .file_stem()
        .expect("Must be a filename")
        .to_string_lossy();

    match width {
        1 => f.write_str(&name),
        n => f.write_str(&name[..n]),
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

        let mut renamer = super::Renamer::new("Fuzzy Bear /{nnn}-/{ooo} (original)");
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

        let mut renamer = super::Renamer::with_idx("Fuzzy Bear /{nnn}-/{ooo} (original)", 21);
        let actual = files
            .into_iter()
            .cloned()
            .map(|x| renamer.rename(x.as_ref()));

        for (actual, &expected) in actual.zip(expected) {
            assert_eq!(actual, expected);
        }
    }
}
