//! Rewrite `src` attributes on `<img>` tags that appear inside raw HTML
//! blocks / inline HTML.
//!
//! READMEs commonly use `<p align="center"><img src="./docs/x.svg"></p>` for
//! centered/sized images. Pulldown-cmark emits these as `Event::Html` /
//! `Event::InlineHtml` and they bypass the markdown image rewriter entirely,
//! so the relative `src` reaches the sanitizer unchanged and gets stripped
//! (because we only whitelist absolute `https://` / `asset://` / `data:image/`
//! for img src).
//!
//! This pass walks HTML events, finds img tags, and resolves relative src
//! URLs through the same resolver the markdown pipeline uses.

use crate::markdown::links::resolve;
use crate::model::{LinkResolution, ResolvedBase};
use pulldown_cmark::{CowStr, Event};

pub fn rewrite<'a>(events: Vec<Event<'a>>, base: &ResolvedBase) -> Vec<Event<'a>> {
    let mut out = Vec::with_capacity(events.len());
    for ev in events {
        match ev {
            Event::Html(s) => {
                let rewritten = rewrite_html_string(s.as_ref(), base);
                if rewritten == s.as_ref() {
                    out.push(Event::Html(s));
                } else {
                    out.push(Event::Html(CowStr::Boxed(rewritten.into_boxed_str())));
                }
            }
            Event::InlineHtml(s) => {
                let rewritten = rewrite_html_string(s.as_ref(), base);
                if rewritten == s.as_ref() {
                    out.push(Event::InlineHtml(s));
                } else {
                    out.push(Event::InlineHtml(CowStr::Boxed(
                        rewritten.into_boxed_str(),
                    )));
                }
            }
            other => out.push(other),
        }
    }
    out
}

/// Walk a raw HTML string and rewrite `src="..."` in every `<img` tag.
fn rewrite_html_string(html: &str, base: &ResolvedBase) -> String {
    let mut out = String::with_capacity(html.len());
    let mut pos = 0;

    while pos < html.len() {
        let Some(rel) = find_img_open(&html[pos..]) else {
            out.push_str(&html[pos..]);
            break;
        };
        let start = pos + rel;
        out.push_str(&html[pos..start]);

        // Find closing `>` (ignoring those inside quoted values).
        let Some(end) = find_tag_close(&html[start..]) else {
            out.push_str(&html[start..]);
            break;
        };
        let tag_end = start + end + 1;
        let tag = &html[start..tag_end];
        out.push_str(&rewrite_img_tag(tag, base));
        pos = tag_end;
    }

    out
}

/// Find the byte offset of the next `<img` open tag — either `<img>`,
/// `<img ...>`, or `<img/>`. Case-insensitive.
fn find_img_open(s: &str) -> Option<usize> {
    let lower = s.to_ascii_lowercase();
    let mut i = 0;
    while let Some(rel) = lower[i..].find("<img") {
        let at = i + rel;
        // Ensure the next char is whitespace, `>`, or `/` (not e.g. `<images`).
        match lower.as_bytes().get(at + 4) {
            Some(c)
                if c.is_ascii_whitespace() || *c == b'>' || *c == b'/' =>
            {
                return Some(at)
            }
            None => return Some(at),
            _ => {
                i = at + 4;
            }
        }
    }
    None
}

/// Find the offset of the `>` that closes the tag starting at index 0 of `s`.
/// Ignores `>` that appear inside quoted attribute values.
fn find_tag_close(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        match (quote, b) {
            (None, b'"') | (None, b'\'') => quote = Some(b),
            (Some(q), c) if c == q => quote = None,
            (None, b'>') => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Rewrite the `src="..."` attribute in a single `<img ...>` tag.
fn rewrite_img_tag(tag: &str, base: &ResolvedBase) -> String {
    let lower = tag.to_ascii_lowercase();
    let Some((value_start, value_end)) = find_attr_value(&lower, tag, "src") else {
        return tag.to_string();
    };
    let original_src = &tag[value_start..value_end];

    // Skip URLs that already have a scheme we don't need to resolve.
    if is_absolute_scheme(original_src) {
        return tag.to_string();
    }

    // Anchor-only hrefs are nonsense for images — leave alone.
    if original_src.starts_with('#') {
        return tag.to_string();
    }

    let resolved = match resolve(base, original_src) {
        LinkResolution::Asset { url, .. } => url,
        LinkResolution::External { url } => url,
        // Images shouldn't resolve as internal docs or anchors; if they do,
        // fall back to the original url.
        _ => return tag.to_string(),
    };

    let mut out = String::with_capacity(tag.len() + resolved.len());
    out.push_str(&tag[..value_start]);
    out.push_str(&resolved);
    out.push_str(&tag[value_end..]);
    out
}

/// Find byte range of the value of an attribute named `name` (lowercase).
/// Returns (value_start, value_end) where value is the INNER string
/// (between the quotes).
fn find_attr_value(
    lower: &str,
    original: &str,
    name: &str,
) -> Option<(usize, usize)> {
    let _ = original;
    // Look for whitespace + name + '='
    let needle = format!("{name}=");
    let mut search_from = 0;
    while let Some(idx) = lower[search_from..].find(&needle) {
        let at = search_from + idx;
        // Must be preceded by whitespace or the '<img' itself (not another attr name).
        if at == 0 {
            search_from = at + needle.len();
            continue;
        }
        let prev = lower.as_bytes()[at - 1];
        if !prev.is_ascii_whitespace() {
            search_from = at + needle.len();
            continue;
        }
        let after_eq = at + needle.len();
        let bytes = lower.as_bytes();
        if after_eq >= bytes.len() {
            return None;
        }
        let quote = bytes[after_eq];
        if quote != b'"' && quote != b'\'' {
            // Unquoted — value runs until whitespace or '>'. Rare; skip.
            return None;
        }
        let value_start = after_eq + 1;
        let end_rel = lower[value_start..].find(quote as char)?;
        let value_end = value_start + end_rel;
        return Some((value_start, value_end));
    }
    None
}

fn is_absolute_scheme(s: &str) -> bool {
    // Matches `scheme:` or `//` (protocol-relative).
    if s.starts_with("//") {
        return true;
    }
    if let Some(idx) = s.find(':') {
        let scheme = &s[..idx];
        !scheme.is_empty()
            && scheme.chars().all(|c| {
                c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.'
            })
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ResolvedBase;
    use pulldown_cmark::Event;
    use std::path::PathBuf;

    fn folder_base() -> ResolvedBase {
        ResolvedBase::Folder {
            file: PathBuf::from("/root/a.md"),
            root: PathBuf::from("/root"),
        }
    }

    #[test]
    fn rewrites_relative_src_to_asset_url() {
        let html = r#"<p align="center"><img src="./logo.svg" alt="logo"></p>"#;
        let out = rewrite_html_string(html, &folder_base());
        assert!(out.contains("asset://localhost/"));
        assert!(out.contains("logo.svg"));
        assert!(!out.contains("src=\"./logo.svg\""));
    }

    #[test]
    fn leaves_https_src_alone() {
        let html = r#"<img src="https://example.com/x.png">"#;
        let out = rewrite_html_string(html, &folder_base());
        assert_eq!(out, html);
    }

    #[test]
    fn leaves_asset_src_alone() {
        let html = r#"<img src="asset://localhost/%2Fa.png">"#;
        let out = rewrite_html_string(html, &folder_base());
        assert_eq!(out, html);
    }

    #[test]
    fn handles_mixed_case_tag() {
        let html = r#"<IMG SRC="./logo.svg" ALT="x">"#;
        let out = rewrite_html_string(html, &folder_base());
        assert!(out.contains("asset://localhost/"));
    }

    #[test]
    fn handles_single_quoted_src() {
        let html = r#"<img src='./logo.svg'>"#;
        let out = rewrite_html_string(html, &folder_base());
        assert!(out.contains("asset://localhost/"));
    }

    #[test]
    fn handles_img_with_width_height() {
        let html = r#"<img width="120" src="./logo.svg" height="80" alt="x">"#;
        let out = rewrite_html_string(html, &folder_base());
        assert!(out.contains("asset://localhost/"));
        assert!(out.contains("width=\"120\""));
        assert!(out.contains("height=\"80\""));
    }

    #[test]
    fn handles_self_closing_tag() {
        let html = r#"<img src="./logo.svg"/>"#;
        let out = rewrite_html_string(html, &folder_base());
        assert!(out.contains("asset://localhost/"));
    }

    #[test]
    fn rewrites_through_event_api() {
        let events = vec![Event::Html(CowStr::Borrowed(
            r#"<img src="./logo.svg" alt="x">"#,
        ))];
        let out = rewrite(events, &folder_base());
        match &out[0] {
            Event::Html(s) => assert!(s.contains("asset://localhost/")),
            _ => panic!(),
        }
    }

    #[test]
    fn leaves_non_img_html_untouched() {
        let html = r#"<p class="note">hello</p>"#;
        let out = rewrite_html_string(html, &folder_base());
        assert_eq!(out, html);
    }
}
