use ammonia::Builder;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Sanitize HTML using a strict allow-list tuned for the Visum markdown pipeline.
///
/// This is the single source of truth for what HTML may reach the webview.
pub fn sanitize(html: &str) -> String {
    let builder = builder();
    builder.clean(html).to_string()
}

fn builder() -> &'static Builder<'static> {
    static B: OnceLock<Builder<'static>> = OnceLock::new();
    B.get_or_init(|| {
        let mut b = Builder::default();

        // Tags.
        let tags: HashSet<&str> = [
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "p",
            "br",
            "hr",
            "blockquote",
            "pre",
            "code",
            "ul",
            "ol",
            "li",
            "strong",
            "em",
            "del",
            "a",
            "img",
            "table",
            "thead",
            "tbody",
            "tr",
            "th",
            "td",
            "sup",
            "sub",
            "span",
            "div",
            "section",
            "aside",
            "details",
            "summary",
            "figure",
            "figcaption",
            "mark",
            "kbd",
            "abbr",
            "input",
            // KaTeX MathML:
            "math",
            "mrow",
            "mi",
            "mn",
            "mo",
            "ms",
            "mtext",
            "mspace",
            "mfrac",
            "msup",
            "msub",
            "msubsup",
            "mover",
            "munder",
            "munderover",
            "mpadded",
            "mphantom",
            "semantics",
            "annotation",
        ]
        .into_iter()
        .collect();
        b.tags(tags);

        // Global attributes (applied to every allowed tag).
        let generic: HashSet<&str> = [
            "class",
            "id",
            "data-internal",
            "data-external",
            "data-resolved",
            "data-src",
            "data-tex",
            "data-code",
            "data-anchor",
            "data-md",
        ]
        .into_iter()
        .collect();
        b.generic_attributes(generic);

        // Per-tag attributes.
        let mut tag_attrs: HashMap<&str, HashSet<&str>> = HashMap::new();
        // Don't list "rel" here — ammonia panics if `link_rel` and per-tag `rel` are both set.
        tag_attrs.insert(
            "a",
            ["href", "title", "target", "name"].into_iter().collect(),
        );
        tag_attrs.insert(
            "img",
            ["src", "alt", "title", "loading", "width", "height"]
                .into_iter()
                .collect(),
        );
        tag_attrs.insert(
            "input",
            ["type", "checked", "disabled"].into_iter().collect(),
        );
        tag_attrs.insert(
            "th",
            ["align", "scope", "colspan", "rowspan"]
                .into_iter()
                .collect(),
        );
        tag_attrs.insert("td", ["align", "colspan", "rowspan"].into_iter().collect());
        tag_attrs.insert("ol", ["start"].into_iter().collect());
        tag_attrs.insert("li", ["value"].into_iter().collect());
        tag_attrs.insert("details", ["open"].into_iter().collect());
        tag_attrs.insert("code", ["data-lang"].into_iter().collect());
        tag_attrs.insert("span", ["style"].into_iter().collect());
        tag_attrs.insert("pre", ["style"].into_iter().collect());
        b.tag_attributes(tag_attrs);

        // URL schemes.
        b.url_schemes(
            ["http", "https", "mailto", "tel", "asset"]
                .into_iter()
                .collect(),
        );

        // Link rewriting: add rel to external links only (keep internal anchors clean).
        b.link_rel(Some("noopener noreferrer"));

        // Attribute filter — the central place where per-attribute value rules run.
        b.attribute_filter(move |element, attr, value| match (element, attr) {
            // style: keep only syntect's inline palette.
            (_, "style") => Some(filter_style(value).into()),
            // img src: only permit https, asset, and inline image data URIs.
            ("img", "src") => {
                let v = value.trim();
                if v.starts_with("https://")
                    || v.starts_with("asset://")
                    || v.starts_with("data:image/")
                {
                    Some(value.into())
                } else {
                    None
                }
            }
            ("input", "type") => {
                if value.eq_ignore_ascii_case("checkbox") {
                    Some("checkbox".into())
                } else {
                    None
                }
            }
            _ => Some(value.into()),
        });

        b
    })
}

/// Keep only known-safe declarations from an inline `style` string.
///
/// Syntect emits inline styles of the form `color:#xxxxxx;background-color:#yyyyyy;font-style:italic;font-weight:bold;text-decoration:underline`.
fn filter_style(value: &str) -> String {
    const ALLOWED: &[&str] = &[
        "color",
        "background-color",
        "font-style",
        "font-weight",
        "text-decoration",
    ];
    let mut kept: Vec<String> = Vec::new();
    for decl in value.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        let Some((prop, val)) = decl.split_once(':') else {
            continue;
        };
        let prop = prop.trim().to_ascii_lowercase();
        let val = val.trim();
        if !ALLOWED.contains(&prop.as_str()) {
            continue;
        }
        // Reject anything with a URL, parentheses, or an expression. Colors / keywords only.
        if val.contains('(') || val.contains(')') || val.contains('/') || val.contains('\\') {
            continue;
        }
        kept.push(format!("{prop}:{val}"));
    }
    kept.join(";")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_tags() {
        let html = r#"<p>hi</p><script>alert(1)</script>"#;
        let out = sanitize(html);
        assert!(!out.contains("<script"));
        assert!(out.contains("<p>hi</p>"));
    }

    #[test]
    fn strips_javascript_href() {
        let html = r#"<a href="javascript:alert(1)">x</a>"#;
        let out = sanitize(html);
        assert!(!out.contains("javascript:"));
    }

    #[test]
    fn blocks_remote_http_images() {
        let html = r#"<img src="http://evil/x.png"><img src="https://ok/y.png">"#;
        let out = sanitize(html);
        assert!(!out.contains("http://evil"));
        assert!(out.contains("https://ok/y.png"));
    }

    #[test]
    fn keeps_asset_protocol_images() {
        let html = r#"<img src="asset://localhost/tmp/x.png">"#;
        let out = sanitize(html);
        assert!(out.contains("asset://localhost/tmp/x.png"));
    }

    #[test]
    fn filter_style_drops_url() {
        assert_eq!(
            filter_style("background-color:#fff;color:url('x')"),
            "background-color:#fff"
        );
    }

    #[test]
    fn filter_style_keeps_known_props() {
        assert_eq!(
            filter_style("color:#abc;font-weight:bold;display:none;"),
            "color:#abc;font-weight:bold"
        );
    }

    #[test]
    fn strips_svg_onload() {
        let html = r#"<img src="https://x/y.png" onload="alert(1)">"#;
        let out = sanitize(html);
        assert!(!out.contains("onload"));
    }

    #[test]
    fn checkbox_input_preserved() {
        let html = r#"<input type="checkbox" disabled checked>"#;
        let out = sanitize(html);
        assert!(out.contains("type=\"checkbox\""));
    }

    #[test]
    fn rejects_non_checkbox_input() {
        let html = r#"<input type="text">"#;
        let out = sanitize(html);
        assert!(!out.contains("type=\"text\""));
    }
}
