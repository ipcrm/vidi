//! Local-folder walker: builds a pruned tree of markdown files.

use crate::error::{AppError, AppResult};
use crate::model::{FileNode, FileTree};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn walk_folder(root: &Path) -> AppResult<FileTree> {
    if !root.exists() {
        return Err(AppError::NotFound(format!(
            "folder does not exist: {}",
            root.display()
        )));
    }
    if !root.is_dir() {
        return Err(AppError::InvalidArgument(format!(
            "not a directory: {}",
            root.display()
        )));
    }

    // Collect all markdown file paths first, then assemble into a tree.
    let mut md_paths: Vec<PathBuf> = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|e| {
            let name = e.file_name();
            !matches!(
                name.to_str(),
                Some(".git") | Some("node_modules") | Some(".DS_Store")
            )
        })
        .build();

    for entry in walker {
        let Ok(entry) = entry else { continue };
        let Some(ft) = entry.file_type() else {
            continue;
        };
        if !ft.is_file() {
            continue;
        }
        let path = entry.path();
        if is_markdown(path) {
            md_paths.push(path.to_path_buf());
        }
    }

    md_paths.sort();
    let nodes = build_tree(root, &md_paths);

    Ok(FileTree {
        root: root.to_path_buf(),
        nodes,
    })
}

fn is_markdown(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase()),
        Some(e) if e == "md" || e == "markdown"
    )
}

fn build_tree(root: &Path, files: &[PathBuf]) -> Vec<FileNode> {
    #[derive(Default)]
    struct Dir {
        children: HashMap<String, Dir>,
        files: Vec<PathBuf>,
    }

    let mut top = Dir::default();

    for path in files {
        let rel = match path.strip_prefix(root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let comps: Vec<String> = rel
            .components()
            .filter_map(|c| c.as_os_str().to_str().map(str::to_string))
            .collect();
        if comps.is_empty() {
            continue;
        }
        let mut cursor = &mut top;
        for dir in &comps[..comps.len() - 1] {
            cursor = cursor.children.entry(dir.clone()).or_default();
        }
        cursor.files.push(path.clone());
    }

    fn to_nodes(dir: Dir, dir_path: PathBuf) -> Vec<FileNode> {
        let mut out = Vec::new();

        let mut dirs: Vec<(String, Dir)> = dir.children.into_iter().collect();
        dirs.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, sub) in dirs {
            let sub_path = dir_path.join(&name);
            let children = to_nodes(sub, sub_path.clone());
            if children.is_empty() {
                continue; // prune empty directories
            }
            out.push(FileNode {
                path: sub_path,
                name,
                is_dir: true,
                children,
            });
        }

        let mut files = dir.files;
        files.sort();
        for f in files {
            let name = f
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            out.push(FileNode {
                path: f,
                name,
                is_dir: false,
                children: Vec::new(),
            });
        }

        out
    }

    to_nodes(top, root.to_path_buf())
}

pub fn read_markdown_file(path: &Path) -> AppResult<String> {
    if !path.exists() {
        return Err(AppError::NotFound(format!(
            "file not found: {}",
            path.display()
        )));
    }
    if !is_markdown(path) {
        return Err(AppError::InvalidArgument(format!(
            "not a markdown file: {}",
            path.display()
        )));
    }
    let bytes = std::fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_tree() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("docs/sub")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(root.join("README.md"), "# hi").unwrap();
        fs::write(root.join("docs/a.md"), "a").unwrap();
        fs::write(root.join("docs/sub/b.markdown"), "b").unwrap();
        fs::write(root.join("docs/not-md.txt"), "skip").unwrap();
        fs::write(root.join(".git/HEAD"), "ref").unwrap();
        fs::write(root.join("node_modules/c.md"), "c").unwrap();
        tmp
    }

    #[test]
    fn walker_only_markdown_and_excludes_git_and_node_modules() {
        let tmp = setup_tree();
        let tree = walk_folder(tmp.path()).unwrap();

        // Flatten all file paths.
        fn collect(nodes: &[FileNode], out: &mut Vec<String>) {
            for n in nodes {
                if n.is_dir {
                    collect(&n.children, out);
                } else {
                    out.push(n.name.clone());
                }
            }
        }
        let mut names = Vec::new();
        collect(&tree.nodes, &mut names);

        assert!(names.contains(&"README.md".to_string()));
        assert!(names.contains(&"a.md".to_string()));
        assert!(names.contains(&"b.markdown".to_string()));
        assert!(!names.contains(&"not-md.txt".to_string()));
        assert!(!names.contains(&"c.md".to_string())); // node_modules excluded
    }

    #[test]
    fn read_missing_file_is_not_found() {
        let err = read_markdown_file(Path::new("/no/such/file.md")).unwrap_err();
        matches!(err, AppError::NotFound(_));
    }

    #[test]
    fn read_non_markdown_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("x.txt");
        std::fs::write(&p, "hi").unwrap();
        let err = read_markdown_file(&p).unwrap_err();
        matches!(err, AppError::InvalidArgument(_));
    }

    #[test]
    fn read_existing_md() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("x.md");
        std::fs::write(&p, "# hello").unwrap();
        let body = read_markdown_file(&p).unwrap();
        assert_eq!(body, "# hello");
    }
}
