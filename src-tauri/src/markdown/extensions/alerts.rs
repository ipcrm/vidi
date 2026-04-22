//! GitHub-style alert rewriter.
//!
//! pulldown-cmark with `ENABLE_GFM` parses `> [!NOTE]` style alerts and emits
//! `Tag::BlockQuote(Some(BlockQuoteKind::...))`. This pass transforms those
//! into semantic `<aside class="alert alert-...">` wrappers.

use pulldown_cmark::{BlockQuoteKind, CowStr, Event, Tag, TagEnd};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl AlertKind {
    pub fn from_cmark(k: BlockQuoteKind) -> Self {
        match k {
            BlockQuoteKind::Note => AlertKind::Note,
            BlockQuoteKind::Tip => AlertKind::Tip,
            BlockQuoteKind::Important => AlertKind::Important,
            BlockQuoteKind::Warning => AlertKind::Warning,
            BlockQuoteKind::Caution => AlertKind::Caution,
        }
    }

    pub fn css_suffix(self) -> &'static str {
        match self {
            AlertKind::Note => "note",
            AlertKind::Tip => "tip",
            AlertKind::Important => "important",
            AlertKind::Warning => "warning",
            AlertKind::Caution => "caution",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            AlertKind::Note => "Note",
            AlertKind::Tip => "Tip",
            AlertKind::Important => "Important",
            AlertKind::Warning => "Warning",
            AlertKind::Caution => "Caution",
        }
    }
}

/// Rewrite alert blockquotes in the event stream.
pub fn rewrite(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut out: Vec<Event<'_>> = Vec::with_capacity(events.len() + 4);
    // Stack of Option<AlertKind> — None means a normal blockquote we didn't convert.
    let mut stack: Vec<Option<AlertKind>> = Vec::new();

    for ev in events {
        match ev {
            Event::Start(Tag::BlockQuote(Some(k))) => {
                let kind = AlertKind::from_cmark(k);
                let html = format!(
                    "<aside class=\"alert alert-{suf}\"><p class=\"alert-title\">{title}</p>",
                    suf = kind.css_suffix(),
                    title = kind.title()
                );
                out.push(Event::Html(CowStr::Boxed(html.into_boxed_str())));
                stack.push(Some(kind));
            }
            Event::Start(Tag::BlockQuote(None)) => {
                stack.push(None);
                out.push(Event::Start(Tag::BlockQuote(None)));
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                let top = stack.pop().unwrap_or(None);
                match top {
                    Some(_) => out.push(Event::Html(CowStr::Borrowed("</aside>"))),
                    None => out.push(Event::End(TagEnd::BlockQuote(None))),
                }
            }
            other => out.push(other),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_suffixes() {
        assert_eq!(AlertKind::Note.css_suffix(), "note");
        assert_eq!(AlertKind::Warning.css_suffix(), "warning");
    }

    #[test]
    fn from_cmark_maps_correctly() {
        assert_eq!(
            AlertKind::from_cmark(BlockQuoteKind::Important),
            AlertKind::Important
        );
        assert_eq!(
            AlertKind::from_cmark(BlockQuoteKind::Caution),
            AlertKind::Caution
        );
    }
}
