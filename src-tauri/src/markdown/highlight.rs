//! Syntax highlighting via `syntect`, producing inline-styled `<span>`s
//! wrapped in `<pre><code class="language-xxx">`.
//!
//! Runs as an event-stream pass after the mermaid pass (mermaid blocks are
//! already replaced with placeholder HTML).

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use std::sync::OnceLock;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    html::{styled_line_to_highlighted_html, IncludeBackground},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

pub fn rewrite<'a>(events: Vec<Event<'a>>, theme_key: &str) -> Vec<Event<'a>> {
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0;

    while i < events.len() {
        if let Event::Start(Tag::CodeBlock(kind)) = &events[i] {
            // Collect code block contents until End(CodeBlock).
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

            let lang = match kind {
                CodeBlockKind::Fenced(l) => l.as_ref().to_string(),
                CodeBlockKind::Indented => String::new(),
            };

            let html = render_code_block(&body, &lang, theme_key);
            out.push(Event::Html(CowStr::Boxed(html.into_boxed_str())));

            i = j.saturating_add(1);
            continue;
        }
        out.push(events[i].clone());
        i += 1;
    }

    out
}

fn render_code_block(code: &str, lang: &str, theme_key: &str) -> String {
    let ss = syntax_set();
    let ts = theme_set();
    let theme = ts
        .themes
        .get(theme_key)
        .or_else(|| ts.themes.get("InspiredGitHub"))
        .or_else(|| ts.themes.values().next())
        .expect("syntect has at least one theme");

    let syntax = (if !lang.is_empty() {
        ss.find_syntax_by_token(lang)
            .or_else(|| ss.find_syntax_by_name(lang))
    } else {
        None
    })
    .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut hl = HighlightLines::new(syntax, theme);
    let mut inner = String::new();
    for line in LinesWithEndings::from(code) {
        match hl.highlight_line(line, ss) {
            Ok(ranges) => {
                if let Ok(html) = styled_line_to_highlighted_html(&ranges, IncludeBackground::No) {
                    inner.push_str(&html);
                }
            }
            Err(_) => {
                inner.push_str(&escape_text(line));
            }
        }
    }

    let class = if lang.is_empty() {
        String::new()
    } else {
        format!(" class=\"language-{}\"", escape_attr(lang))
    };
    format!("<pre class=\"code-block\"><code{class}>{inner}</code></pre>")
}

fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_set() -> &'static ThemeSet {
    static TS: OnceLock<ThemeSet> = OnceLock::new();
    TS.get_or_init(ThemeSet::load_defaults)
}

fn escape_attr(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn escape_text(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            c => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_pre_code_wrapper() {
        let html = render_code_block("fn x() {}\n", "rust", "InspiredGitHub");
        assert!(html.starts_with("<pre"));
        assert!(html.contains("<code class=\"language-rust\">"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn unknown_language_falls_back_to_plain() {
        let html = render_code_block("hello\n", "zzzzzz", "InspiredGitHub");
        assert!(html.contains("hello"));
    }

    #[test]
    fn empty_lang_no_class() {
        let html = render_code_block("x\n", "", "InspiredGitHub");
        assert!(html.contains("<code>"));
        assert!(!html.contains("class=\"language-"));
    }
}
