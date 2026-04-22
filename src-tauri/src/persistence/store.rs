//! Atomic JSON-backed KV store.
//!
//! One file per collection — `settings.json`, `recents.json`, `bookmarks.json`,
//! `positions.json`. Writes go through a temp file + `persist` to avoid
//! torn-write corruption.

use crate::error::{AppError, AppResult};
use crate::model::{Bookmark, ReadingPosition, RecentFile, Settings, Source};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_RECENTS: usize = 50;

#[derive(Debug, Serialize, Deserialize)]
struct Envelope<T> {
    version: u32,
    data: T,
}

pub struct Store {
    dir: PathBuf,
    settings: Mutex<Settings>,
    recents: Mutex<Vec<RecentFile>>,
    bookmarks: Mutex<Vec<Bookmark>>,
    positions: Mutex<HashMap<String, ReadingPosition>>,
}

impl Store {
    pub fn open(dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(dir)?;
        let settings = load_or_default(&dir.join("settings.json"))?;
        let recents = load_or_default(&dir.join("recents.json"))?;
        let bookmarks = load_or_default(&dir.join("bookmarks.json"))?;
        let positions = load_or_default(&dir.join("positions.json"))?;

        Ok(Self {
            dir: dir.to_path_buf(),
            settings: Mutex::new(settings),
            recents: Mutex::new(recents),
            bookmarks: Mutex::new(bookmarks),
            positions: Mutex::new(positions),
        })
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn set_settings(&self, s: Settings) -> AppResult<()> {
        *self.settings.lock().unwrap() = s.clone();
        write_atomic(&self.dir.join("settings.json"), &s)
    }

    pub fn recents(&self) -> Vec<RecentFile> {
        self.recents.lock().unwrap().clone()
    }

    pub fn push_recent(&self, r: RecentFile) -> AppResult<()> {
        let mut list = self.recents.lock().unwrap();
        list.retain(|x| x.source != r.source);
        list.insert(0, r);
        if list.len() > MAX_RECENTS {
            list.truncate(MAX_RECENTS);
        }
        let snapshot = list.clone();
        drop(list);
        write_atomic(&self.dir.join("recents.json"), &snapshot)
    }

    pub fn bookmarks(&self) -> Vec<Bookmark> {
        self.bookmarks.lock().unwrap().clone()
    }

    pub fn add_bookmark(&self, b: Bookmark) -> AppResult<Bookmark> {
        let mut list = self.bookmarks.lock().unwrap();
        list.push(b.clone());
        let snapshot = list.clone();
        drop(list);
        write_atomic(&self.dir.join("bookmarks.json"), &snapshot)?;
        Ok(b)
    }

    pub fn remove_bookmark(&self, id: &str) -> AppResult<()> {
        let mut list = self.bookmarks.lock().unwrap();
        list.retain(|b| b.id != id);
        let snapshot = list.clone();
        drop(list);
        write_atomic(&self.dir.join("bookmarks.json"), &snapshot)
    }

    pub fn reading_position(&self, source: &Source) -> Option<ReadingPosition> {
        self.positions.lock().unwrap().get(&source.key()).cloned()
    }

    pub fn set_reading_position(&self, source: Source, pos: ReadingPosition) -> AppResult<()> {
        let mut map = self.positions.lock().unwrap();
        map.insert(source.key(), pos);
        let snapshot = map.clone();
        drop(map);
        write_atomic(&self.dir.join("positions.json"), &snapshot)
    }
}

fn load_or_default<T: DeserializeOwned + Default>(path: &Path) -> AppResult<T> {
    if !path.exists() {
        return Ok(T::default());
    }
    let bytes = std::fs::read(path)?;
    let env: Envelope<T> = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return Ok(T::default()), // corrupted file — start fresh
    };
    Ok(env.data)
}

fn write_atomic<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    let env = Envelope {
        version: 1,
        data: value,
    };
    let json = serde_json::to_vec_pretty(&env)?;
    let dir = path
        .parent()
        .ok_or_else(|| AppError::InvalidArgument(format!("invalid path: {}", path.display())))?;
    std::fs::create_dir_all(dir)?;
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    {
        use std::io::Write;
        tmp.as_file_mut().write_all(&json)?;
        tmp.as_file_mut().sync_data()?;
    }
    tmp.persist(path).map_err(|e| AppError::Io(e.error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AssetKind, Theme};

    fn open_tmp() -> (Store, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path()).unwrap();
        (store, tmp)
    }

    #[test]
    fn defaults_are_readable() {
        let (s, _t) = open_tmp();
        let settings = s.settings();
        assert_eq!(settings.theme, Theme::System);
        assert!(s.recents().is_empty());
        assert!(s.bookmarks().is_empty());
    }

    #[test]
    fn recent_roundtrip() {
        let (s, _t) = open_tmp();
        let src = Source::LocalFile {
            path: std::path::PathBuf::from("/a.md"),
        };
        s.push_recent(RecentFile {
            source: src.clone(),
            title: "A".into(),
            opened_at: 1,
        })
        .unwrap();
        s.push_recent(RecentFile {
            source: Source::LocalFile {
                path: std::path::PathBuf::from("/b.md"),
            },
            title: "B".into(),
            opened_at: 2,
        })
        .unwrap();
        // Push A again — should move to front, not duplicate.
        s.push_recent(RecentFile {
            source: src.clone(),
            title: "A".into(),
            opened_at: 3,
        })
        .unwrap();
        let list = s.recents();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].source, src);
    }

    #[test]
    fn persists_across_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        {
            let s = Store::open(tmp.path()).unwrap();
            s.add_bookmark(Bookmark {
                id: "b1".into(),
                source: Source::Remote {
                    url: "https://x".into(),
                },
                label: "X".into(),
                anchor: None,
                created_at: 1,
            })
            .unwrap();
        }
        let s2 = Store::open(tmp.path()).unwrap();
        let list = s2.bookmarks();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "b1");
        // Kind coverage so AssetKind import isn't unused.
        let _ = AssetKind::Image;
    }

    #[test]
    fn reading_position_keyed_by_source() {
        let (s, _t) = open_tmp();
        let src = Source::LocalFile {
            path: std::path::PathBuf::from("/x.md"),
        };
        assert!(s.reading_position(&src).is_none());
        s.set_reading_position(
            src.clone(),
            ReadingPosition {
                scroll_ratio: 0.5,
                anchor: None,
                updated_at: 1,
            },
        )
        .unwrap();
        let got = s.reading_position(&src).unwrap();
        assert!((got.scroll_ratio - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn corrupted_file_falls_back_to_default() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("settings.json"), b"not json").unwrap();
        let s = Store::open(tmp.path()).unwrap();
        assert_eq!(s.settings().measure_ch, 92);
    }
}
