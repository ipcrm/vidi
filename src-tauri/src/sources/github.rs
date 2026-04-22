//! GitHub URL normalization.
//!
//! Translates user-entered `github.com` / `gist.github.com` URLs into their
//! raw-content equivalents so relative links inside the rendered markdown
//! resolve against a stable base.

use url::Url;

/// Result of normalizing a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalized {
    /// URL to fetch the markdown from.
    pub fetch_url: String,
    /// URL to use as the document's base (for relative links).
    pub base_url: String,
}

/// Normalize a user-entered URL to a fetch + base pair.
/// Returns the input verbatim when no GitHub-specific rewrite applies.
pub fn normalize(input: &str) -> Normalized {
    if let Ok(url) = Url::parse(input) {
        if let Some(host) = url.host_str() {
            return match host {
                "github.com" => normalize_github_com(&url).unwrap_or_else(|| passthrough(input)),
                "gist.github.com" => normalize_gist(&url).unwrap_or_else(|| passthrough(input)),
                _ => passthrough(input),
            };
        }
    }
    passthrough(input)
}

fn passthrough(s: &str) -> Normalized {
    Normalized {
        fetch_url: s.to_string(),
        base_url: s.to_string(),
    }
}

fn normalize_github_com(url: &Url) -> Option<Normalized> {
    // Paths we understand:
    //   /{owner}/{repo}                               → HEAD/README.md
    //   /{owner}/{repo}/blob/{branch}/{path...}       → raw/{branch}/{path}
    //   /{owner}/{repo}/tree/{branch}/{dir}/README.md (tree URLs are dirs, skip)
    let segs: Vec<&str> = url.path_segments().map(|s| s.collect()).unwrap_or_default();
    let segs: Vec<&str> = segs.into_iter().filter(|s| !s.is_empty()).collect();

    match segs.as_slice() {
        [owner, repo] => {
            let raw = format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/README.md");
            Some(Normalized {
                fetch_url: raw.clone(),
                base_url: raw,
            })
        }
        [owner, repo, "blob", branch, rest @ ..] => {
            let path = rest.join("/");
            let raw = format!("https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}");
            Some(Normalized {
                fetch_url: raw.clone(),
                base_url: raw,
            })
        }
        _ => None,
    }
}

fn normalize_gist(url: &Url) -> Option<Normalized> {
    let segs: Vec<&str> = url.path_segments().map(|s| s.collect()).unwrap_or_default();
    let segs: Vec<&str> = segs.into_iter().filter(|s| !s.is_empty()).collect();

    match segs.as_slice() {
        [user, id] => {
            let raw = format!("https://gist.githubusercontent.com/{user}/{id}/raw");
            Some(Normalized {
                fetch_url: raw.clone(),
                base_url: raw,
            })
        }
        [user, id, "raw", rest @ ..] => {
            let path = rest.join("/");
            let raw = if path.is_empty() {
                format!("https://gist.githubusercontent.com/{user}/{id}/raw")
            } else {
                format!("https://gist.githubusercontent.com/{user}/{id}/raw/{path}")
            };
            Some(Normalized {
                fetch_url: raw.clone(),
                base_url: raw,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_for_raw() {
        let u = "https://raw.githubusercontent.com/o/r/HEAD/README.md";
        let n = normalize(u);
        assert_eq!(n.fetch_url, u);
        assert_eq!(n.base_url, u);
    }

    #[test]
    fn github_blob_rewrite() {
        let n = normalize("https://github.com/o/r/blob/main/docs/a.md");
        assert_eq!(
            n.fetch_url,
            "https://raw.githubusercontent.com/o/r/main/docs/a.md"
        );
        assert_eq!(n.base_url, n.fetch_url);
    }

    #[test]
    fn github_root_repo_to_readme() {
        let n = normalize("https://github.com/o/r");
        assert_eq!(
            n.fetch_url,
            "https://raw.githubusercontent.com/o/r/HEAD/README.md"
        );
    }

    #[test]
    fn gist_short_url() {
        let n = normalize("https://gist.github.com/u/abcdef");
        assert_eq!(
            n.fetch_url,
            "https://gist.githubusercontent.com/u/abcdef/raw"
        );
    }

    #[test]
    fn gist_raw_passthrough_with_path() {
        let u = "https://gist.github.com/u/abcdef/raw/file.md";
        let n = normalize(u);
        assert_eq!(
            n.fetch_url,
            "https://gist.githubusercontent.com/u/abcdef/raw/file.md"
        );
    }

    #[test]
    fn unknown_host_passthrough() {
        let u = "https://example.com/some.md";
        let n = normalize(u);
        assert_eq!(n.fetch_url, u);
    }
}
