use std::slice;

use regex::{Match, Regex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Segment {
    /// A literal segment
    Literal(String),

    /// Indicates a numeric segment; the integer indicates the formatting width of the number
    Numeric(usize),

    /// Segment indicating use of the original filename; integer indicates how much of the filename to use
    Filename(usize),
}

pub struct TemplateParser {
    pattern: Regex,
}

impl TemplateParser {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r#"[^\\]?(\{([FfNnOo0])(:\d+)?\})"#).unwrap(),
        }
    }

    pub fn parse(&self, template: &str) -> Template {
        let captures = self.pattern.captures_iter(template);

        let mut segments = Vec::new();
        let mut left = 0;

        let captures = captures.filter_map(|cx| {
            Some(Formatter {
                template: cx.get(1)?,
                specifier: cx.get(2)?.as_str(),
                quantifier: cx.get(3).map(|cx| cx.as_str()),
            })
        });

        for formatter in captures {
            if formatter.template.start() > left {
                segments.push(Segment::Literal(
                    template[left..formatter.template.start()].into(),
                ));
            }

            match formatter.specifier {
                "0" | "n" | "N" => segments.push(Segment::Numeric(formatter.quantifier())),
                "o" | "O" | "f" | "F" => segments.push(Segment::Filename(formatter.quantifier())),
                _ => (),
            }

            left = formatter.template.end();
        }

        if left < template.len() {
            segments.push(Segment::Literal(template[left..].into()));
        }

        Template { segments }
    }
}

struct Formatter<'a> {
    template: Match<'a>,
    specifier: &'a str,
    quantifier: Option<&'a str>,
}

impl Formatter<'_> {
    fn quantifier(&self) -> usize {
        self.quantifier
            .and_then(|s| {
                let s = &s[1..];
                s.parse().ok()
            })
            .unwrap_or(1)
    }
}

#[derive(Clone, Debug)]
pub struct Template {
    segments: Vec<Segment>,
}

impl Template {
    pub fn segments(&self) -> slice::Iter<Segment> {
        self.segments.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::template::{Template, TemplateParser};

    #[test]
    fn can_create_template() {
        let parser = TemplateParser::new();
        let Template { segments } = parser.parse("Moab Vacation {o} {n:4}");
        let expected = vec![
            super::Segment::Literal(String::from("Moab Vacation ")),
            super::Segment::Filename(1),
            super::Segment::Literal(String::from(" ")),
            super::Segment::Numeric(4),
        ];
        assert_eq!(segments, expected);
    }
}
