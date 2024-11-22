use std::{cell::Cell, rc::Rc};

use html2text::render::{TaggedLine, TextDecorator};

// TODO: Find a way to avoid wrapping inline links
// and find a better way of postfix URLs - another PR?
// See https://github.com/jugglerchris/rust-html2text/issues/37
// Also find a way to display > like markdown block quotes
// Also find a way to add blank lines where borders were inserted

#[derive(Clone, Debug)]
pub struct PlainDecorator {
    nlinks: Rc<Cell<usize>>,
    // links: Vec<String>
}
impl PlainDecorator {
    /// Create a new `PlainDecorator`.
    pub fn new() -> PlainDecorator {
        PlainDecorator {
            nlinks: Rc::new(Cell::new(0)),
            // links: Vec::new()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum TextMode {
    Link,
    #[default]
    Default,
}
impl TextDecorator for PlainDecorator {
    type Annotation = TextMode;

    fn decorate_link_start(&mut self, _url: &str) -> (String, Self::Annotation) {
        self.nlinks.set(self.nlinks.get() + 1);
        // self.links.push(url.into());
        (("[").to_string(), TextMode::Link)
    }

    fn decorate_link_end(&mut self) -> String {
        // ("]").to_string()
        format!("][{}]", self.nlinks.get())
        // format!("]({})", self.links.pop().unwrap_or_default())
    }

    fn decorate_em_start(&self) -> (String, Self::Annotation) {
        ("*".to_string(), TextMode::default())
    }

    fn decorate_em_end(&self) -> String {
        "*".to_string()
    }

    fn decorate_strong_start(&self) -> (String, Self::Annotation) {
        ("**".to_string(), TextMode::default())
    }

    fn decorate_strong_end(&self) -> String {
        "**".to_string()
    }

    fn decorate_strikeout_start(&self) -> (String, Self::Annotation) {
        ("".to_string(), TextMode::default())
    }

    fn decorate_strikeout_end(&self) -> String {
        "".to_string()
    }

    fn decorate_code_start(&self) -> (String, Self::Annotation) {
        ("`".to_string(), TextMode::default())
    }

    fn decorate_code_end(&self) -> String {
        "`".to_string()
    }

    fn decorate_preformat_first(&self) -> Self::Annotation {
        TextMode::default()
    }
    fn decorate_preformat_cont(&self) -> Self::Annotation {
        TextMode::default()
    }

    fn decorate_image(&mut self, _src: &str, title: &str) -> (String, Self::Annotation) {
        if title == "MusicBrainz" {
            ("".to_string(), TextMode::default())
        } else {
            (format!("[{}]", title), TextMode::default())
        }
    }

    fn header_prefix(&self, level: usize) -> String {
        "#".repeat(level) + " "
    }

    fn quote_prefix(&self) -> String {
        "> ".to_string()
    }

    fn unordered_item_prefix(&self) -> String {
        "* ".to_string()
    }

    fn ordered_item_prefix(&self, i: i64) -> String {
        format!("{}. ", i)
    }

    fn finalise(&mut self, links: Vec<String>) -> Vec<TaggedLine<Self::Annotation>> {
        links
            .into_iter()
            .enumerate()
            .map(|(idx, s)| {
                TaggedLine::from_string(format!("[{}]: {}", idx + 1, s), &TextMode::default())
            })
            .collect()
        // Vec::new()
    }

    fn make_subblock_decorator(&self) -> Self {
        self.clone()
    }

    fn decorate_superscript_start(&self) -> (String, Self::Annotation) {
        ("^{".into(), Default::default())
    }

    fn decorate_superscript_end(&self) -> String {
        "}".into()
    }
}
