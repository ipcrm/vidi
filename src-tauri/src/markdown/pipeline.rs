//! Markdown → sanitized HTML pipeline.

use super::extensions::{alerts, emoji, heading_ids, math, mermaid, raw_html_images};
use super::links::LinkRewriter;
use super::{highlight, sanitize};
use crate::model::{ExternalAsset, RenderOptions, RenderedDoc, ResolvedBase, Theme, TocEntry};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

pub struct RenderInput<'a> {
    pub text: &'a str,
    pub base: ResolvedBase,
    pub options: RenderOptions,
}

pub fn render(input: RenderInput<'_>) -> RenderedDoc {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    opts.insert(Options::ENABLE_MATH);
    opts.insert(Options::ENABLE_GFM);

    let parser = Parser::new_ext(input.text, opts);
    let events: Vec<Event<'_>> = parser.collect();

    // 1. Alerts.
    let events = alerts::rewrite(events);

    // 2. Heading ids — slugify text so `[x](#section)` anchors work.
    let events = heading_ids::rewrite(events);

    // 3. Emoji.
    let events = emoji::rewrite(events);

    // 3. Math placeholders.
    let enable_math = input.options.enable_math.unwrap_or(true);
    let (events, has_math_raw) = math::rewrite(events);
    let has_math = enable_math && has_math_raw;

    // 4. Mermaid placeholders.
    let enable_mermaid = input.options.enable_mermaid.unwrap_or(true);
    let (events, has_mermaid_raw) = mermaid::rewrite(events);
    let has_mermaid = enable_mermaid && has_mermaid_raw;

    // 5. Syntax highlighting.
    let theme_key = match input.options.theme {
        Theme::Dark => "base16-ocean.dark",
        _ => "InspiredGitHub",
    };
    let events = highlight::rewrite(events, theme_key);

    // 6. Link + image rewriting (markdown image/link syntax).
    let rewriter = LinkRewriter { base: &input.base };
    let rewritten = rewriter.run(events);
    let events = rewritten.events;
    let external_assets: Vec<ExternalAsset> = rewritten.external_assets;

    // 6b. Rewrite `<img src="...">` in raw HTML blocks too — READMEs commonly
    // use `<p align="center"><img src="./foo.svg"></p>` for layout, and those
    // bypass the markdown image pass entirely.
    let events = raw_html_images::rewrite(events, &input.base);

    // 7. Extract TOC + title + word count before HTML render (we use a separate pass).
    let (title, toc) = extract_toc(&events);
    let word_count = estimate_word_count(&events);

    // 8. Emit HTML.
    let mut raw_html = String::new();
    pulldown_cmark::html::push_html(&mut raw_html, events.into_iter());

    // 9. Sanitize.
    let html = sanitize::sanitize(&raw_html);

    RenderedDoc {
        html,
        title,
        toc,
        word_count,
        has_math,
        has_mermaid,
        external_assets,
        base: input.base,
    }
}

fn extract_toc(events: &[Event<'_>]) -> (Option<String>, Vec<TocEntry>) {
    let mut toc = Vec::new();
    let mut title: Option<String> = None;
    let mut current: Option<(u8, String, String)> = None; // (level, text, id)

    for ev in events {
        match ev {
            Event::Start(Tag::Heading { level, id, .. }) => {
                let lvl = heading_level(*level);
                // After heading_ids::rewrite, every heading has an id.
                let existing_id = id.as_ref().map(|c| c.to_string()).unwrap_or_default();
                current = Some((lvl, String::new(), existing_id));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((lvl, text, id)) = current.take() {
                    let anchor = if id.is_empty() { slugify(&text) } else { id };
                    if lvl == 1 && title.is_none() {
                        title = Some(text.clone());
                    }
                    toc.push(TocEntry {
                        level: lvl,
                        text,
                        anchor,
                    });
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some((_, ref mut text, _)) = current {
                    text.push_str(t.as_ref());
                }
            }
            _ => {}
        }
    }

    (title, toc)
}

fn heading_level(h: HeadingLevel) -> u8 {
    match h {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            for l in c.to_lowercase() {
                out.push(l);
            }
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn estimate_word_count(events: &[Event<'_>]) -> usize {
    let mut n = 0usize;
    let mut in_code = 0u32;
    for ev in events {
        match ev {
            Event::Start(Tag::CodeBlock(_)) => in_code += 1,
            Event::End(TagEnd::CodeBlock) => in_code = in_code.saturating_sub(1),
            Event::Text(t) if in_code == 0 => {
                n += t.split_whitespace().count();
            }
            _ => {}
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{RenderOptions, ResolvedBase};

    fn render_inline(text: &str) -> RenderedDoc {
        render(RenderInput {
            text,
            base: ResolvedBase::Inline,
            options: RenderOptions::default(),
        })
    }

    #[test]
    fn basic_paragraph() {
        let doc = render_inline("hello **world**");
        assert!(doc.html.contains("<p>"));
        assert!(doc.html.contains("<strong>"));
        assert_eq!(doc.word_count, 2);
    }

    #[test]
    fn title_is_first_h1() {
        let doc = render_inline("# Hello\n\n## Sub\n\nBody.");
        assert_eq!(doc.title.as_deref(), Some("Hello"));
        assert_eq!(doc.toc.len(), 2);
        assert_eq!(doc.toc[0].level, 1);
        assert_eq!(doc.toc[1].level, 2);
        assert_eq!(doc.toc[1].anchor, "sub");
    }

    #[test]
    fn gfm_table_renders() {
        let md = "| a | b |\n|---|---|\n| 1 | 2 |\n";
        let doc = render_inline(md);
        assert!(doc.html.contains("<table>"));
        assert!(doc.html.contains("<th>a</th>"));
        assert!(doc.html.contains("<td>2</td>"));
    }

    #[test]
    fn tasklist_renders() {
        let md = "- [x] done\n- [ ] todo\n";
        let doc = render_inline(md);
        assert!(doc.html.contains("type=\"checkbox\""));
    }

    #[test]
    fn strike_renders() {
        let doc = render_inline("~~nope~~");
        assert!(doc.html.contains("<del>"));
    }

    #[test]
    fn code_block_highlighted() {
        let doc = render_inline("```rust\nfn main(){}\n```");
        assert!(doc.html.contains("<pre"));
        assert!(doc.html.contains("language-rust"));
    }

    #[test]
    fn script_is_stripped() {
        let doc = render_inline("ok<script>alert(1)</script>");
        assert!(!doc.html.contains("<script"));
        assert!(doc.html.contains("ok"));
    }

    #[test]
    fn javascript_href_stripped() {
        let doc = render_inline("[x](javascript:alert(1))");
        assert!(!doc.html.contains("javascript:"));
    }

    #[test]
    fn alert_note_produces_aside() {
        let doc = render_inline("> [!NOTE]\n> hello");
        assert!(doc.html.contains("<aside"));
        assert!(doc.html.contains("alert-note"));
        assert!(doc.html.contains("hello"));
    }

    #[test]
    fn alert_warning_kind() {
        let doc = render_inline("> [!WARNING]\n> careful");
        assert!(doc.html.contains("alert-warning"));
    }

    #[test]
    fn plain_blockquote_is_not_an_alert() {
        let doc = render_inline("> a plain quote");
        assert!(!doc.html.contains("<aside"));
        assert!(doc.html.contains("<blockquote"));
    }

    #[test]
    fn internal_md_link_has_data_attrs() {
        use std::path::PathBuf;
        let doc = render(RenderInput {
            text: "see [other](./other.md) for more",
            base: ResolvedBase::Folder {
                file: PathBuf::from("/root/a.md"),
                root: PathBuf::from("/root"),
            },
            options: RenderOptions::default(),
        });
        assert!(doc.html.contains("data-internal=\"true\""));
        assert!(doc.html.contains("data-resolved="));
        // JSON values are HTML-encoded in the attribute — browser decodes on read.
        assert!(doc.html.contains("&quot;kind&quot;:&quot;localFile&quot;"));
    }

    #[test]
    fn heading_gets_slug_id() {
        let doc = render_inline("## Install the thing\n\nhello");
        assert!(
            doc.html.contains("id=\"install-the-thing\""),
            "html: {}",
            doc.html
        );
    }

    #[test]
    fn anchor_link_target_id_matches() {
        // Anchor href `#install-the-thing` must match the heading id we emit.
        let doc = render_inline("## Install the thing\n\n[go](#install-the-thing)");
        assert!(doc.html.contains("id=\"install-the-thing\""));
        assert!(doc.html.contains("href=\"#install-the-thing\""));
    }

    #[test]
    fn emoji_shortcode_replaced() {
        let doc = render_inline("hi :smile:");
        assert!(!doc.html.contains(":smile:"));
    }

    #[test]
    fn math_inline_placeholder() {
        let doc = render_inline("an inline $x^2$ formula");
        assert!(doc.html.contains("math-inline"));
        assert!(doc.has_math);
    }

    #[test]
    fn math_display_placeholder() {
        let doc = render_inline("$$\n\\frac{1}{2}\n$$");
        assert!(doc.html.contains("math-display"));
        assert!(doc.has_math);
    }

    #[test]
    fn mermaid_placeholder() {
        let doc = render_inline("```mermaid\ngraph TD; A-->B;\n```");
        assert!(doc.html.contains("mermaid-source"));
        assert!(doc.has_mermaid);
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Section 1.2 - Intro!"), "section-1-2-intro");
    }
}
