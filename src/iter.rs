use std::{
    collections::HashMap,
    error, fmt,
    path::{Path, PathBuf},
};

pub struct DataTracker<'a> {
    paths: HashMap<&'a Path, bool>,
}

impl<'a> DataTracker<'a> {
    pub fn new(paths: &'a [impl AsRef<Path>]) -> Self {
        Self {
            paths: paths.iter().map(|path| (path.as_ref(), true)).collect(),
        }
    }

    /// Reset file states to conflict
    pub fn reset(&mut self) {
        self.paths.iter_mut().for_each(|kv| *kv.1 = true);
    }

    pub fn check_iteration(
        &mut self,
        iteration: impl Iterator<Item = Operation<'a>>,
    ) -> Result<(), Conflict> {
        self.reset(); // Just in case

        for operation in iteration {
            if let Some(from) = self.paths.get_mut(operation.from) {
                *from = false;
            }

            let is_conflict = self.paths.get(operation.to).copied().unwrap_or_default();
            if is_conflict {
                return Err(operation.into_conflict());
            }
        }
        Ok(())
    }
}

pub struct Operation<'a> {
    pub from: &'a Path,
    pub to: &'a Path,
}

impl Operation<'_> {
    pub fn into_conflict(self) -> Conflict {
        Conflict {
            from: self.from.into(),
            to: self.to.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Conflict {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl fmt::Display for Conflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "move conflict\n  {}\n  {}",
            self.from.display(),
            self.to.display()
        )
    }
}

impl error::Error for Conflict {}

#[derive(Clone, Debug)]
pub struct MultimodeConflict {
    pub forward: Conflict,
    pub reverse: Conflict,
}

impl MultimodeConflict {
    pub fn new(forward: Conflict, reverse: Conflict) -> Self {
        Self { forward, reverse }
    }
}

impl fmt::Display for MultimodeConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}\n{}\ntoo many conflicts", self.forward, self.reverse)
    }
}

impl error::Error for MultimodeConflict {}

pub struct Forward<'a, T> {
    idx: usize,
    from: &'a [T],
    to: &'a [T],
}

impl<'a, T> Forward<'a, T> {
    pub fn new(from: &'a [T], to: &'a [T]) -> Self {
        Self { idx: 0, from, to }
    }

    pub fn reset(&mut self) {
        self.idx = 0;
    }
}

impl<'a, T: AsRef<Path>> Iterator for Forward<'a, T> {
    type Item = Operation<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.from.len() {
            return None;
        }

        let operation = Operation {
            from: self.from[self.idx].as_ref(),
            to: self.to[self.idx].as_ref(),
        };

        self.idx += 1;
        Some(operation)
    }
}

pub struct Reverse<'a, T> {
    idx: usize,
    from: &'a [T],
    to: &'a [T],
}

impl<'a, T> Reverse<'a, T> {
    pub fn new(from: &'a [T], to: &'a [T]) -> Self {
        Self {
            idx: from.len(),
            from,
            to,
        }
    }

    pub fn reset(&mut self) {
        self.idx = self.from.len();
    }
}

impl<'a, T: AsRef<Path>> Iterator for Reverse<'a, T> {
    type Item = Operation<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            return None;
        }

        let operation = Operation {
            from: self.from[self.idx - 1].as_ref(),
            to: self.to[self.idx - 1].as_ref(),
        };

        self.idx -= 1;
        Some(operation)
    }
}

#[cfg(test)]
mod tests {
    use super::{DataTracker, Forward, Reverse};

    #[test]
    fn forward() {
        let from = &["a.txt", "b.txt"];
        let to = &["1.txt", "2.txt"];

        let iterator = Forward::new(from, to);
        assert_eq!(2, iterator.count());
    }

    #[test]
    fn reverse() {
        let from = &["a.txt", "b.txt"];
        let to = &["1.txt", "2.txt"];

        let iterator = Reverse::new(from, to);
        assert_eq!(2, iterator.count());
    }

    #[test]
    fn must_reverse_rename() {
        let from = &["00", "01", "02"];
        let to = &["01", "02", "03"];
        let mut tracker = DataTracker::new(from);
        assert!(tracker.check_iteration(Forward::new(from, to)).is_err());
        assert!(tracker.check_iteration(Reverse::new(from, to)).is_ok());
    }

    #[test]
    fn must_forward_rename() {
        let from = &["01", "02", "03"];
        let to = &["00", "01", "02"];
        let mut tracker = DataTracker::new(from);
        assert!(tracker.check_iteration(Forward::new(from, to)).is_ok());
        assert!(tracker.check_iteration(Reverse::new(from, to)).is_err());
    }
}
