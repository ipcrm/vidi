use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Source {
    LocalFile { path: PathBuf },
    Remote { url: String },
    LocalFolder { root: PathBuf },
}

impl Source {
    /// Stable string key used for keying persistence maps (reading positions, etc.).
    pub fn key(&self) -> String {
        match self {
            Source::LocalFile { path } => format!("file:{}", path.display()),
            Source::Remote { url } => format!("url:{url}"),
            Source::LocalFolder { root } => format!("folder:{}", root.display()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RenderOptions {
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub enable_math: Option<bool>,
    #[serde(default)]
    pub enable_mermaid: Option<bool>,
    #[serde(default)]
    pub base_override: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenderedDoc {
    pub html: String,
    pub title: Option<String>,
    pub toc: Vec<TocEntry>,
    pub word_count: usize,
    pub has_math: bool,
    pub has_mermaid: bool,
    pub external_assets: Vec<ExternalAsset>,
    pub base: ResolvedBase,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TocEntry {
    pub level: u8,
    pub text: String,
    pub anchor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ResolvedBase {
    Folder { file: PathBuf, root: PathBuf },
    Remote { base_url: String },
    Inline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAsset {
    pub kind: AssetKind,
    pub url: String,
    pub placeholder_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AssetKind {
    Image,
    Iframe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileTree {
    pub root: PathBuf,
    pub nodes: Vec<FileNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecentFile {
    pub source: Source,
    pub title: String,
    pub opened_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: String,
    pub source: Source,
    pub label: String,
    pub anchor: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReadingPosition {
    pub scroll_ratio: f32,
    #[serde(default)]
    pub anchor: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: Theme,
    pub font_scale: f32,
    pub measure_ch: u16,
    pub trusted_hosts: Vec<String>,
    pub confirm_always: bool,
    pub drop_caps: bool,
    pub enable_math: bool,
    pub enable_mermaid: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            font_scale: 1.0,
            measure_ch: 92,
            trusted_hosts: Vec::new(),
            confirm_always: true,
            drop_caps: false,
            enable_math: true,
            enable_mermaid: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "resolution", rename_all = "camelCase")]
pub enum LinkResolution {
    InternalDoc { source: Source },
    Anchor { fragment: String },
    External { url: String },
    Asset { url: String, asset_kind: AssetKind },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFetch {
    pub text: String,
    pub final_url: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatchHandle {
    pub id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_key_roundtrip() {
        let s = Source::LocalFile {
            path: PathBuf::from("/tmp/a.md"),
        };
        assert_eq!(s.key(), "file:/tmp/a.md");

        let s = Source::Remote {
            url: "https://example.com/x.md".into(),
        };
        assert_eq!(s.key(), "url:https://example.com/x.md");
    }

    #[test]
    fn default_settings_safe() {
        let s = Settings::default();
        assert!(s.confirm_always);
        assert_eq!(s.measure_ch, 92);
        assert!(s.enable_math);
        assert!(s.enable_mermaid);
        assert!(s.trusted_hosts.is_empty());
    }

    #[test]
    fn source_serde_tagged() {
        let s = Source::Remote {
            url: "https://x".into(),
        };
        let j = serde_json::to_string(&s).unwrap();
        assert!(j.contains("\"kind\":\"remote\""));
        let back: Source = serde_json::from_str(&j).unwrap();
        assert_eq!(back, s);
    }
}
