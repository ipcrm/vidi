//! Replace ```mermaid fenced code blocks with placeholder divs so the
//! frontend can lazy-load mermaid.js and render them.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

pub fn rewrite(events: Vec<Event<'_>>) -> (Vec<Event<'_>>, bool) {
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0;
    let mut has_mermaid = false;

    while i < events.len() {
        // Look for a mermaid fenced block: Start(CodeBlock(Fenced("mermaid"))) ... End(CodeBlock)
        if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) = &events[i] {
            if lang.as_ref().trim().eq_ignore_ascii_case("mermaid") {
                // Collect all Text events until End(CodeBlock).
                let mut body = String::new();
                let mut j = i + 1;
                while j < events.len() {
                    match &events[j] {
                        Event::Text(t) => body.push_str(t.as_ref()),
                        Event::End(TagEnd::CodeBlock) => break,
                        _ => {}
                    }
                    j += 1;
                }

                has_mermaid = true;
                let encoded = STANDARD.encode(body.as_bytes());
                let html = format!("<div class=\"mermaid-source\" data-code=\"{encoded}\"></div>");
                out.push(Event::Html(CowStr::Boxed(html.into_boxed_str())));

                // Advance past End(CodeBlock).
                i = j.saturating_add(1);
                continue;
            }
        }
        out.push(events[i].clone());
        i += 1;
    }

    (out, has_mermaid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn replaces_mermaid_block_with_placeholder() {
        let events = vec![
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(
                "mermaid",
            )))),
            Event::Text(CowStr::Borrowed("graph TD; A-->B;")),
            Event::End(TagEnd::CodeBlock),
        ];
        let (out, has) = rewrite(events);
        assert!(has);
        assert_eq!(out.len(), 1);
        match &out[0] {
            Event::Html(h) => {
                assert!(h.contains("mermaid-source"));
                let b64 = STANDARD.encode("graph TD; A-->B;");
                assert!(h.contains(&b64));
            }
            _ => panic!("expected Html placeholder"),
        }
    }

    #[test]
    fn leaves_non_mermaid_blocks() {
        let events = vec![
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(
                "rust",
            )))),
            Event::Text(CowStr::Borrowed("fn x(){}")),
            Event::End(TagEnd::CodeBlock),
        ];
        let (out, has) = rewrite(events);
        assert!(!has);
        assert_eq!(out.len(), 3);
    }
}
