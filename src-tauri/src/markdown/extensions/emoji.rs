//! Replace `:shortcode:` runs in prose with actual emoji characters.
//!
//! Skips `Event::Text` that is nested inside a `CodeBlock` or inline `Code`.

use pulldown_cmark::{CowStr, Event, Tag, TagEnd};

pub fn rewrite(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut out = Vec::with_capacity(events.len());
    let mut code_depth: u32 = 0;

    for ev in events {
        match &ev {
            Event::Start(Tag::CodeBlock(_)) => {
                code_depth += 1;
                out.push(ev);
            }
            Event::End(TagEnd::CodeBlock) => {
                code_depth = code_depth.saturating_sub(1);
                out.push(ev);
            }
            Event::Text(t) if code_depth == 0 => {
                let replaced = replace_shortcodes(t.as_ref());
                if replaced == t.as_ref() {
                    out.push(ev);
                } else {
                    out.push(Event::Text(CowStr::Boxed(replaced.into_boxed_str())));
                }
            }
            _ => out.push(ev),
        }
    }

    out
}

fn replace_shortcodes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b':' {
            // Find a closing colon within a reasonable distance.
            if let Some(end) = find_closing_colon(bytes, i + 1) {
                let shortcode = &input[i + 1..end];
                if is_valid_shortcode(shortcode) {
                    if let Some(e) = emojis::get_by_shortcode(shortcode) {
                        out.push_str(e.as_str());
                        i = end + 1;
                        continue;
                    }
                }
            }
        }
        // Push the next char (handle UTF-8 boundaries).
        let ch_len = match bytes[i] {
            b if b < 0x80 => 1,
            b if b < 0xC0 => 1, // malformed continuation; copy byte as-is
            b if b < 0xE0 => 2,
            b if b < 0xF0 => 3,
            _ => 4,
        };
        let end = (i + ch_len).min(bytes.len());
        out.push_str(&input[i..end]);
        i = end;
    }

    out
}

fn find_closing_colon(bytes: &[u8], from: usize) -> Option<usize> {
    // Shortcodes are alphanumeric + `-` + `_` + `+`, bounded and short.
    const MAX: usize = 64;
    let hi = (from + MAX).min(bytes.len());
    for (j, b) in bytes.iter().enumerate().take(hi).skip(from) {
        if *b == b':' {
            return Some(j);
        }
        if !(b.is_ascii_alphanumeric() || *b == b'-' || *b == b'_' || *b == b'+') {
            return None;
        }
    }
    None
}

fn is_valid_shortcode(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '+')
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

    #[test]
    fn replaces_known_shortcodes() {
        let out = replace_shortcodes("hello :smile: world");
        assert!(out.contains('\u{1F604}') || out.contains("😄"));
        assert!(!out.contains(":smile:"));
    }

    #[test]
    fn leaves_unknown_shortcodes() {
        let out = replace_shortcodes("hi :notarealemoji: bye");
        assert_eq!(out, "hi :notarealemoji: bye");
    }

    #[test]
    fn preserves_stray_colons() {
        let out = replace_shortcodes("time: 12:34:56");
        assert_eq!(out, "time: 12:34:56");
    }

    #[test]
    fn skips_inside_code_blocks() {
        let events = vec![
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(
                pulldown_cmark::CowStr::Borrowed("rust"),
            ))),
            Event::Text(pulldown_cmark::CowStr::Borrowed(":smile:")),
            Event::End(TagEnd::CodeBlock),
        ];
        let out = rewrite(events);
        // Text should be unchanged.
        if let Event::Text(t) = &out[1] {
            assert_eq!(t.as_ref(), ":smile:");
        } else {
            panic!("expected text");
        }
    }
}
