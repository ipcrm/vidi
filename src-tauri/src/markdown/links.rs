//! Link + image rewriting.
//!
//! - Classifies each link as internal `.md`, external, anchor, or asset.
//! - Emits `<a>` tags directly as HTML events so we can attach
//!   `data-internal`/`data-external` and the resolved `Source` the frontend
//!   needs to route without re-calling the backend.
//! - Turns external `<img>` into placeholder `<span>`s with `data-src` so
//!   the frontend can prompt-before-loading.
//! - Rewrites local image paths to `asset://` URLs.

use crate::model::{AssetKind, ExternalAsset, LinkResolution, ResolvedBase, Source};
use pulldown_cmark::{CowStr, Event, Tag, TagEnd};
use std::path::PathBuf;
use url::Url;

pub struct LinkRewriter<'a> {
    pub base: &'a ResolvedBase,
}

pub struct RewriteResult<'a> {
    pub events: Vec<Event<'a>>,
    pub external_assets: Vec<ExternalAsset>,
}

enum ImageTarget {
    LocalUrl(String),
    External(String),
}

impl<'a> LinkRewriter<'a> {
    pub fn run(&self, events: Vec<Event<'a>>) -> RewriteResult<'a> {
        let mut out: Vec<Event<'a>> = Vec::with_capacity(events.len());
        let mut external_assets: Vec<ExternalAsset> = Vec::new();

        // Track image depth so we can close placeholder spans on End(Image).
        let mut placeholder_depth: u32 = 0;

        for ev in events {
            match ev {
                Event::Start(Tag::Link {
                    dest_url, title, ..
                }) => {
                    let (new_url, data_attrs) = match resolve(self.base, dest_url.as_ref()) {
                        LinkResolution::InternalDoc { source } => {
                            (dest_url.as_ref().to_string(), internal_attrs(&source))
                        }
                        LinkResolution::Anchor { fragment } => {
                            (format!("#{fragment}"), String::new())
                        }
                        LinkResolution::External { url } => {
                            (url, " data-external=\"true\"".to_string())
                        }
                        LinkResolution::Asset { url, .. } => (url, String::new()),
                    };
                    let title_attr = if title.as_ref().is_empty() {
                        String::new()
                    } else {
                        format!(" title=\"{}\"", escape_attr(title.as_ref()))
                    };
                    out.push(Event::Html(CowStr::Boxed(
                        format!(
                            "<a href=\"{}\"{}{}>",
                            escape_attr(&new_url),
                            title_attr,
                            data_attrs
                        )
                        .into_boxed_str(),
                    )));
                }
                Event::End(TagEnd::Link) => {
                    out.push(Event::Html(CowStr::Borrowed("</a>")));
                }
                Event::Start(Tag::Image {
                    dest_url, title, ..
                }) => {
                    let target = match resolve(self.base, dest_url.as_ref()) {
                        LinkResolution::Asset { url, .. } => ImageTarget::LocalUrl(url),
                        LinkResolution::External { url } => ImageTarget::External(url),
                        LinkResolution::Anchor { .. } => {
                            ImageTarget::LocalUrl(dest_url.to_string())
                        }
                        LinkResolution::InternalDoc { .. } => {
                            // Remote internal image — treat as external (needs prompt).
                            let url = resolve_url_string(self.base, dest_url.as_ref())
                                .unwrap_or_else(|| dest_url.to_string());
                            ImageTarget::External(url)
                        }
                    };

                    match target {
                        ImageTarget::LocalUrl(url) => {
                            let title_attr = if title.as_ref().is_empty() {
                                String::new()
                            } else {
                                format!(" title=\"{}\"", escape_attr(title.as_ref()))
                            };
                            // Emit an explicit <img>. We need to skip the alt-text
                            // inline events that follow — so mark placeholder depth
                            // to swallow them and close on End(Image).
                            out.push(Event::Html(CowStr::Boxed(
                                format!("<img src=\"{}\"{} alt=\"", escape_attr(&url), title_attr)
                                    .into_boxed_str(),
                            )));
                            placeholder_depth += 1;
                            // Inline events between here and End(Image) form the alt
                            // text; we'll emit them into the attribute context. That's
                            // safe because push_html HTML-escapes Text events.
                        }
                        ImageTarget::External(url) => {
                            let placeholder_id = stable_id(&url);
                            external_assets.push(ExternalAsset {
                                kind: AssetKind::Image,
                                url: url.clone(),
                                placeholder_id: placeholder_id.clone(),
                            });
                            placeholder_depth += 1;
                            out.push(Event::Html(CowStr::Boxed(
                                format!(
                                    "<span class=\"image-placeholder\" data-src=\"{}\" data-id=\"{}\" role=\"button\" tabindex=\"0\">",
                                    escape_attr(&url),
                                    escape_attr(&placeholder_id)
                                )
                                .into_boxed_str(),
                            )));
                            out.push(Event::Html(CowStr::Borrowed(
                                "<span class=\"image-placeholder-alt\">",
                            )));
                        }
                    }
                }
                Event::End(TagEnd::Image) => {
                    if placeholder_depth > 0 {
                        placeholder_depth -= 1;
                        // Two distinct closings: <img ...alt=""> vs <span><span>...
                        // We can't tell here which one — emit the combined close that
                        // works for both by using a marker. Simplest: always close
                        // a single element. The LocalUrl path emitted `<img src="..." alt="`
                        // which is an unterminated open, so we need to close the attr
                        // and the tag: `">`.  The External path emitted two <span>s.
                        // We need separate tracking.
                        out.push(Event::Html(CowStr::Borrowed("__VISUM_IMG_END__")));
                    } else {
                        out.push(Event::End(TagEnd::Image));
                    }
                }
                other => out.push(other),
            }
        }

        // Post-process: walk the emitted event stream once more to fix up the
        // `__VISUM_IMG_END__` sentinel — we can tell which kind of image we're
        // closing by looking back for the most recent `image-placeholder` or
        // `<img ` open that hasn't been closed.
        let out = fixup_image_closers(out);

        RewriteResult {
            events: out,
            external_assets,
        }
    }
}

/// Replace `__VISUM_IMG_END__` sentinels with the correct closing HTML based on
/// which kind of image open preceded them.
fn fixup_image_closers(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    // Stack of "is external placeholder" flags.
    let mut stack: Vec<bool> = Vec::new();
    let mut out: Vec<Event<'_>> = Vec::with_capacity(events.len());

    for ev in events {
        match &ev {
            Event::Html(h) => {
                let s = h.as_ref();
                if s == "__VISUM_IMG_END__" {
                    let was_external = stack.pop().unwrap_or(false);
                    if was_external {
                        out.push(Event::Html(CowStr::Borrowed("</span></span>")));
                    } else {
                        // Close the img alt attribute and self-close the element.
                        out.push(Event::Html(CowStr::Borrowed("\" />")));
                    }
                } else if s.starts_with("<span class=\"image-placeholder\"") {
                    stack.push(true);
                    out.push(ev);
                } else if s.starts_with("<img ") {
                    stack.push(false);
                    out.push(ev);
                } else {
                    out.push(ev);
                }
            }
            _ => out.push(ev),
        }
    }

    out
}

/// Classify an href against a base. Pure function — used by both the link pass
/// and the `resolve_link` Tauri command.
pub fn resolve(base: &ResolvedBase, href: &str) -> LinkResolution {
    let trimmed = href.trim();
    if trimmed.is_empty() {
        return LinkResolution::External { url: String::new() };
    }
    if let Some(frag) = trimmed.strip_prefix('#') {
        return LinkResolution::Anchor {
            fragment: frag.to_string(),
        };
    }

    match base {
        ResolvedBase::Folder { file, root } => resolve_folder(file, root, trimmed),
        ResolvedBase::Remote { base_url } => resolve_remote(base_url, trimmed),
        ResolvedBase::Inline => LinkResolution::External {
            url: trimmed.to_string(),
        },
    }
}

fn is_absolute_url(s: &str) -> bool {
    if let Some(idx) = s.find(':') {
        let scheme = &s[..idx];
        !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    } else {
        false
    }
}

fn resolve_folder(file: &std::path::Path, root: &std::path::Path, href: &str) -> LinkResolution {
    if is_absolute_url(href) {
        return LinkResolution::External {
            url: href.to_string(),
        };
    }

    // Split off any anchor fragment.
    let (path_part, anchor) = match href.split_once('#') {
        Some((p, a)) => (p, Some(a.to_string())),
        None => (href, None),
    };

    let candidate: PathBuf = if path_part.is_empty() {
        if let Some(a) = anchor {
            return LinkResolution::Anchor { fragment: a };
        }
        file.to_path_buf()
    } else if path_part.starts_with('/') {
        root.join(path_part.trim_start_matches('/'))
    } else {
        file.parent().unwrap_or(root).join(path_part)
    };

    let canonical = canonicalize_joined(&candidate);

    // Markdown links navigate freely — when the user opens a single file the
    // "root" is just the file's parent directory, so requiring containment
    // would break links to sibling docs in parent directories (e.g. `../api.md`).
    // The worst case here is opening a file the user didn't expect; since this
    // is a local reader with no network fetch of internal docs, that's fine.
    if is_markdown(&canonical) {
        let _ = anchor;
        return LinkResolution::InternalDoc {
            source: Source::LocalFile { path: canonical },
        };
    }

    // Non-markdown (images, etc.) stay scoped to the root so we don't hand
    // out `asset://` URLs for arbitrary files on disk.
    let within_root = canonical.starts_with(root);
    if !within_root {
        return LinkResolution::External {
            url: href.to_string(),
        };
    }
    let url = asset_url(&canonical);
    LinkResolution::Asset {
        url,
        asset_kind: AssetKind::Image,
    }
}

fn resolve_remote(base_url: &str, href: &str) -> LinkResolution {
    let base = match Url::parse(base_url) {
        Ok(b) => b,
        Err(_) => {
            return LinkResolution::External {
                url: href.to_string(),
            }
        }
    };
    let Ok(abs) = base.join(href) else {
        return LinkResolution::External {
            url: href.to_string(),
        };
    };

    let abs_str = abs.to_string();
    if let Some(host) = abs.host_str() {
        let is_known_md_host = matches!(
            host,
            "raw.githubusercontent.com" | "gist.githubusercontent.com"
        ) || base.host_str() == Some(host);
        if is_known_md_host && abs.path().to_ascii_lowercase().ends_with(".md") {
            return LinkResolution::InternalDoc {
                source: Source::Remote { url: abs_str },
            };
        }
    }
    LinkResolution::External { url: abs_str }
}

fn resolve_url_string(base: &ResolvedBase, href: &str) -> Option<String> {
    match base {
        ResolvedBase::Remote { base_url } => {
            let b = Url::parse(base_url).ok()?;
            b.join(href).ok().map(|u| u.to_string())
        }
        _ => None,
    }
}

fn is_markdown(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()),
        Some(e) if e == "md" || e == "markdown"
    )
}

fn canonicalize_joined(path: &std::path::Path) -> PathBuf {
    let mut components: Vec<std::path::Component<'_>> = Vec::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if let Some(last) = components.last() {
                    if !matches!(last, std::path::Component::RootDir) {
                        components.pop();
                        continue;
                    }
                }
                components.push(comp);
            }
            other => components.push(other),
        }
    }
    let mut out = PathBuf::new();
    for c in components {
        out.push(c.as_os_str());
    }
    out
}

fn asset_url(path: &std::path::Path) -> String {
    // Match Tauri's `convertFileSrc` helper — encodeURIComponent-style
    // encoding (space → %20, /  → %2F). Using form-urlencoded here breaks
    // paths with spaces because form encoding emits `+` for space, which
    // Tauri's asset protocol doesn't accept.
    let s = path.to_string_lossy();
    let mut out = String::from("asset://localhost/");
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            _ => out.push_str(&format!("%{:02X}", *b)),
        }
    }
    out
}

fn stable_id(url: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in url.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("img-{h:016x}")
}

fn internal_attrs(source: &Source) -> String {
    let j = serde_json::to_string(source).unwrap_or_else(|_| "{}".into());
    format!(
        " data-internal=\"true\" data-resolved=\"{}\"",
        escape_attr(&j)
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn folder_base() -> ResolvedBase {
        ResolvedBase::Folder {
            file: PathBuf::from("/root/a/b.md"),
            root: PathBuf::from("/root"),
        }
    }

    #[test]
    fn folder_relative_md_is_internal() {
        let r = resolve(&folder_base(), "./c.md");
        match r {
            LinkResolution::InternalDoc {
                source: Source::LocalFile { path },
            } => assert_eq!(path, PathBuf::from("/root/a/c.md")),
            _ => panic!("expected InternalDoc"),
        }
    }

    #[test]
    fn folder_parent_md_is_internal() {
        let r = resolve(&folder_base(), "../d.md");
        match r {
            LinkResolution::InternalDoc {
                source: Source::LocalFile { path },
            } => assert_eq!(path, PathBuf::from("/root/d.md")),
            _ => panic!("expected InternalDoc"),
        }
    }

    #[test]
    fn folder_rooted_md_is_internal() {
        let r = resolve(&folder_base(), "/top.md");
        match r {
            LinkResolution::InternalDoc {
                source: Source::LocalFile { path },
            } => assert_eq!(path, PathBuf::from("/root/top.md")),
            _ => panic!("expected InternalDoc"),
        }
    }

    #[test]
    fn folder_external_absolute_is_external() {
        let r = resolve(&folder_base(), "https://example.com");
        match r {
            LinkResolution::External { url } => assert_eq!(url, "https://example.com"),
            _ => panic!("expected External"),
        }
    }

    #[test]
    fn folder_escape_root_markdown_is_internal() {
        // Markdown links outside the root still resolve as internal docs — lets
        // users navigate sibling files when they open a single file directly.
        let r = resolve(&folder_base(), "../../outside.md");
        match r {
            LinkResolution::InternalDoc {
                source: Source::LocalFile { path },
            } => assert_eq!(path, PathBuf::from("/outside.md")),
            _ => panic!("expected InternalDoc"),
        }
    }

    #[test]
    fn folder_escape_root_non_markdown_is_external() {
        // Non-markdown assets stay scoped to root so we don't produce
        // `asset://` URLs for arbitrary files on disk.
        let r = resolve(&folder_base(), "../../outside.png");
        match r {
            LinkResolution::External { .. } => {}
            _ => panic!("expected External"),
        }
    }

    #[test]
    fn folder_non_markdown_is_asset() {
        let r = resolve(&folder_base(), "./diagram.png");
        match r {
            LinkResolution::Asset { url, .. } => {
                assert!(url.starts_with("asset://localhost/"));
                // URL-component encoded: diagram.png survives as-is in the slug.
                assert!(url.contains("diagram.png"));
            }
            _ => panic!("expected Asset"),
        }
    }

    #[test]
    fn anchor_only() {
        let r = resolve(&folder_base(), "#section");
        match r {
            LinkResolution::Anchor { fragment } => assert_eq!(fragment, "section"),
            _ => panic!(),
        }
    }

    #[test]
    fn remote_same_host_md_is_internal() {
        let base = ResolvedBase::Remote {
            base_url: "https://raw.githubusercontent.com/o/r/HEAD/docs/a.md".into(),
        };
        let r = resolve(&base, "./b.md");
        match r {
            LinkResolution::InternalDoc {
                source: Source::Remote { url },
            } => assert_eq!(url, "https://raw.githubusercontent.com/o/r/HEAD/docs/b.md"),
            _ => panic!(),
        }
    }

    #[test]
    fn remote_other_host_is_external() {
        let base = ResolvedBase::Remote {
            base_url: "https://raw.githubusercontent.com/o/r/HEAD/docs/a.md".into(),
        };
        let r = resolve(&base, "https://evil.example/x.md");
        match r {
            LinkResolution::External { .. } => {}
            _ => panic!(),
        }
    }

    #[test]
    fn canonicalize_parent_dir() {
        assert_eq!(
            canonicalize_joined(&PathBuf::from("/root/a/../b.md")),
            PathBuf::from("/root/b.md")
        );
    }

    #[test]
    fn asset_url_uses_uri_component_encoding() {
        let u = asset_url(&PathBuf::from("/Users/x/a b/c.png"));
        // Space must be %20, not '+'; slashes must be %2F.
        assert_eq!(
            u,
            "asset://localhost/%2FUsers%2Fx%2Fa%20b%2Fc.png"
        );
    }
}
