//! Convert pulldown-cmark math events into HTML placeholders that the
//! frontend hydrates with KaTeX.
//!
//! Requires `pulldown_cmark::Options::ENABLE_MATH` on the parser.

use pulldown_cmark::{CowStr, Event};

pub fn rewrite(events: Vec<Event<'_>>) -> (Vec<Event<'_>>, bool) {
    let mut out = Vec::with_capacity(events.len());
    let mut has_math = false;

    for ev in events {
        match ev {
            Event::InlineMath(tex) => {
                has_math = true;
                out.push(Event::Html(CowStr::Boxed(
                    format!(
                        "<span class=\"math math-inline\" data-tex=\"{}\"></span>",
                        html_escape_attr(tex.as_ref())
                    )
                    .into_boxed_str(),
                )));
            }
            Event::DisplayMath(tex) => {
                has_math = true;
                out.push(Event::Html(CowStr::Boxed(
                    format!(
                        "<div class=\"math math-display\" data-tex=\"{}\"></div>",
                        html_escape_attr(tex.as_ref())
                    )
                    .into_boxed_str(),
                )));
            }
            _ => out.push(ev),
        }
    }

    (out, has_math)
}

fn html_escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_quotes_and_angle_brackets() {
        assert_eq!(
            html_escape_attr(r#"a < "b" > c"#),
            "a &lt; &quot;b&quot; &gt; c"
        );
    }

    #[test]
    fn inline_math_becomes_span_placeholder() {
        let events = vec![Event::InlineMath(CowStr::Borrowed("x^2"))];
        let (out, has) = rewrite(events);
        assert!(has);
        match &out[0] {
            Event::Html(h) => assert!(h.contains("math-inline") && h.contains("x^2")),
            _ => panic!("expected Html"),
        }
    }

    #[test]
    fn display_math_becomes_div_placeholder() {
        let events = vec![Event::DisplayMath(CowStr::Borrowed("\\frac{1}{2}"))];
        let (out, has) = rewrite(events);
        assert!(has);
        match &out[0] {
            Event::Html(h) => assert!(h.contains("math-display") && h.contains("frac")),
            _ => panic!("expected Html"),
        }
    }
}
