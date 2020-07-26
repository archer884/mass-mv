use regex::Regex;
use std::slice;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Segment {
    /// A literal segment
    Literal(String),

    /// Indicates a numeric segment; the integer indicates the formatting width of the number
    Numeric(usize),

    /// Segment indicating use of the original filename; integer indicates how much of the filename to use
    Filename(usize),
}

#[derive(Clone, Debug)]
pub struct Template {
    segments: Vec<Segment>,
}

impl Template {
    pub fn new(s: &str) -> Template {
        let pattern = Regex::new(r#"\{\{(.+?)\}\}"#).unwrap();
        let captures = pattern.captures_iter(s);

        let mut segments = Vec::new();
        let mut left = 0;

        for cap in captures {
            let full_match = cap.get(0).unwrap();
            if full_match.start() > left {
                segments.push(Segment::Literal(s[left..full_match.start()].into()));
            }

            let token = cap.get(1).unwrap().as_str();
            match &token[..1] {
                "0" | "n" | "N" => segments.push(Segment::Numeric(token.len())),
                "o" | "O" | "f" | "F" => segments.push(Segment::Filename(token.len())),
                _ => (),
            }

            left = full_match.end();
        }

        if left < s.len() {
            segments.push(Segment::Literal(s[left..].to_string()))
        }

        Template { segments }
    }

    pub fn segments(&self) -> slice::Iter<Segment> {
        self.segments.iter()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn can_create_template() {
        let super::Template { segments } = super::Template::new("Moab Vacation {{o}} {{nnnn}}");
        let expected = vec![
            super::Segment::Literal(String::from("Moab Vacation ")),
            super::Segment::Filename(1),
            super::Segment::Literal(String::from(" ")),
            super::Segment::Numeric(4),
        ];

        assert_eq!(segments, expected);
    }
}
