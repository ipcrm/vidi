//! Full-text search over the markdown files in the currently-open folder.
//!
//! Builds a fresh tantivy in-memory index each time a folder is indexed.
//! Held in `AppState::search` behind a Mutex; frontend calls
//! `search_folder(query)` to query it.
//!
//! Design notes:
//! - The index is volatile: closing the app or switching folders drops it.
//! - Documents: one per markdown file. Fields = `path` (stored + raw string),
//!   `title` (stored + indexed), `body` (indexed only — saves memory).
//! - Index directory uses tantivy's RAM directory to avoid writing anything
//!   to disk.

use crate::error::{AppError, AppResult};
use crate::sources::local;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Field, Schema, TextFieldIndexing, TextOptions, Value, STORED, STRING},
    Index, IndexReader,
};

pub struct SearchIndex {
    root: PathBuf,
    index: Index,
    reader: IndexReader,
    path_field: Field,
    title_field: Field,
    body_field: Field,
    doc_count: usize,
}

pub struct SearchState {
    current: Mutex<Option<SearchIndex>>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
        }
    }

    pub fn build_for(&self, root: &Path) -> AppResult<usize> {
        let idx = build_index(root)?;
        let n = idx.doc_count;
        *self.current.lock().unwrap() = Some(idx);
        Ok(n)
    }

    pub fn clear(&self) {
        *self.current.lock().unwrap() = None;
    }

    pub fn search(&self, query: &str, limit: usize) -> AppResult<Vec<SearchHit>> {
        let guard = self.current.lock().unwrap();
        let Some(idx) = guard.as_ref() else {
            return Err(AppError::InvalidArgument("no folder indexed".into()));
        };
        idx.search(query, limit)
    }

    pub fn active_root(&self) -> Option<PathBuf> {
        self.current
            .lock()
            .unwrap()
            .as_ref()
            .map(|i| i.root.clone())
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub path: PathBuf,
    pub title: String,
    pub score: f32,
    pub snippet: String,
}

fn build_schema() -> (Schema, Field, Field, Field) {
    let mut builder = Schema::builder();

    // path: stored only, exact-match string field.
    let path_field = builder.add_text_field("path", STRING | STORED);

    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("default")
                .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let title_field = builder.add_text_field("title", text_options);

    let body_indexing = TextFieldIndexing::default()
        .set_tokenizer("default")
        .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions);
    let body_options = TextOptions::default()
        .set_indexing_options(body_indexing)
        .set_stored();
    let body_field = builder.add_text_field("body", body_options);

    (builder.build(), path_field, title_field, body_field)
}

fn build_index(root: &Path) -> AppResult<SearchIndex> {
    let (schema, path_field, title_field, body_field) = build_schema();
    let index = Index::create_in_ram(schema);
    let mut writer = index
        .writer(50_000_000)
        .map_err(|e| AppError::Internal(format!("tantivy writer: {e}")))?;

    let tree = local::walk_folder(root)?;
    let mut count = 0usize;
    flatten(&tree.nodes, &mut |path| {
        let body = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return,
        };
        let title = first_h1(&body).unwrap_or_else(|| {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("(untitled)")
                .to_string()
        });

        let mut doc = tantivy::TantivyDocument::default();
        doc.add_text(path_field, path.to_string_lossy().as_ref());
        doc.add_text(title_field, &title);
        doc.add_text(body_field, &body);
        let _ = writer.add_document(doc);
        count += 1;
    });

    writer
        .commit()
        .map_err(|e| AppError::Internal(format!("tantivy commit: {e}")))?;

    let reader = index
        .reader()
        .map_err(|e| AppError::Internal(format!("tantivy reader: {e}")))?;

    Ok(SearchIndex {
        root: root.to_path_buf(),
        index,
        reader,
        path_field,
        title_field,
        body_field,
        doc_count: count,
    })
}

impl SearchIndex {
    fn search(&self, query: &str, limit: usize) -> AppResult<Vec<SearchHit>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let limit = limit.clamp(1, 50);
        let searcher = self.reader.searcher();
        let parser = QueryParser::for_index(&self.index, vec![self.title_field, self.body_field]);
        let parsed = parser.parse_query_lenient(query).0;
        let hits = searcher
            .search(&parsed, &TopDocs::with_limit(limit))
            .map_err(|e| AppError::Internal(format!("tantivy search: {e}")))?;

        use tantivy::TantivyDocument;
        let mut out = Vec::with_capacity(hits.len());
        for (score, addr) in hits {
            let Ok(doc) = searcher.doc::<TantivyDocument>(addr) else {
                continue;
            };
            let path = doc
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .map(PathBuf::from)
                .unwrap_or_default();
            let title = doc
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let body = doc
                .get_first(self.body_field)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            out.push(SearchHit {
                path,
                title,
                score,
                snippet: snippet_around(body, query),
            });
        }
        Ok(out)
    }
}

/// Walk a FileNode tree and invoke `cb` for each file path.
fn flatten(nodes: &[crate::model::FileNode], cb: &mut dyn FnMut(&Path)) {
    for n in nodes {
        if n.is_dir {
            flatten(&n.children, cb);
        } else {
            cb(&n.path);
        }
    }
}

fn first_h1(md: &str) -> Option<String> {
    for line in md.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("# ") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// Return up to ~160 characters of context around the first query-term match.
fn snippet_around(body: &str, query: &str) -> String {
    const RADIUS: usize = 80;
    let lower_body = body.to_lowercase();
    let terms: Vec<&str> = query.split_whitespace().collect();
    let first_term = terms.iter().find(|t| !t.is_empty()).copied().unwrap_or("");
    if first_term.is_empty() {
        return body.chars().take(160).collect();
    }
    let lower_term = first_term.to_lowercase();
    let Some(byte_idx) = lower_body.find(&lower_term) else {
        return body.chars().take(160).collect();
    };
    let start_byte = byte_idx.saturating_sub(RADIUS);
    let end_byte = (byte_idx + lower_term.len() + RADIUS).min(body.len());
    let start = floor_char_boundary(body, start_byte);
    let end = floor_char_boundary(body, end_byte);
    let mut s = body[start..end].replace('\n', " ");
    s = s.trim().to_string();
    if start > 0 {
        s = format!("…{s}");
    }
    if end < body.len() {
        s.push('…');
    }
    s
}

fn floor_char_boundary(s: &str, i: usize) -> usize {
    let mut i = i.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup() -> tempfile::TempDir {
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join("a.md"),
            "# Alpha\n\nRouting rocket guidance.\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("b.md"),
            "# Beta\n\nMonkeys eat rocket fuel.\n",
        )
        .unwrap();
        fs::create_dir(tmp.path().join("sub")).unwrap();
        fs::write(
            tmp.path().join("sub/c.md"),
            "# Gamma\n\nRoute planning notes.\n",
        )
        .unwrap();
        tmp
    }

    #[test]
    fn builds_index_and_finds_docs() {
        let tmp = setup();
        let st = SearchState::new();
        let n = st.build_for(tmp.path()).unwrap();
        assert_eq!(n, 3);

        let hits = st.search("rocket", 10).unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits
            .iter()
            .all(|h| h.snippet.contains("rocket") || h.snippet.contains("Rocket")));

        let alpha = st.search("alpha", 10).unwrap();
        assert!(alpha.iter().any(|h| h.path.ends_with("a.md")));
    }

    #[test]
    fn missing_returns_empty() {
        let tmp = setup();
        let st = SearchState::new();
        st.build_for(tmp.path()).unwrap();
        assert!(st.search("xylophones", 10).unwrap().is_empty());
    }

    #[test]
    fn empty_query_returns_empty() {
        let tmp = setup();
        let st = SearchState::new();
        st.build_for(tmp.path()).unwrap();
        assert!(st.search("  ", 10).unwrap().is_empty());
    }

    #[test]
    fn search_without_index_errors() {
        let st = SearchState::new();
        match st.search("anything", 10) {
            Err(AppError::InvalidArgument(_)) => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn switch_folder_rebuilds() {
        let tmp1 = setup();
        let tmp2 = tempdir().unwrap();
        fs::write(tmp2.path().join("x.md"), "# Xray\n\nAnother topic.\n").unwrap();

        let st = SearchState::new();
        st.build_for(tmp1.path()).unwrap();
        assert!(!st.search("rocket", 10).unwrap().is_empty());

        st.build_for(tmp2.path()).unwrap();
        assert!(st.search("rocket", 10).unwrap().is_empty());
        assert_eq!(st.search("xray", 10).unwrap().len(), 1);
    }

    #[test]
    fn snippet_truncates_and_includes_ellipses() {
        let body = "alpha ".to_string() + &"word ".repeat(200) + "rocket boost";
        let snip = snippet_around(&body, "rocket");
        assert!(snip.contains("rocket"));
        assert!(snip.starts_with('…'));
    }
}
