//! A small cross-platform terminal file browser for picking a YAML file.
//!
//! Shows directories and `.yaml`/`.yml` files, lets the user enter/leave
//! directories, and never panics on inaccessible directories.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Entry {
    pub label: String,
    pub path: PathBuf,
    pub is_dir: bool,
    /// The synthetic ".." parent entry.
    pub is_parent: bool,
}

#[derive(Debug)]
pub struct FileBrowser {
    pub cwd: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: usize,
    pub error: Option<String>,
}

impl Default for FileBrowser {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut b = FileBrowser {
            cwd,
            entries: Vec::new(),
            selected: 0,
            error: None,
        };
        b.reload();
        b
    }
}

impl FileBrowser {
    fn reload(&mut self) {
        self.entries.clear();
        self.error = None;
        self.selected = 0;

        if let Some(parent) = self.cwd.parent() {
            self.entries.push(Entry {
                label: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                is_parent: true,
            });
        }

        let mut dirs: Vec<Entry> = Vec::new();
        let mut files: Vec<Entry> = Vec::new();
        match std::fs::read_dir(&self.cwd) {
            Ok(rd) => {
                for entry in rd.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = path.is_dir();
                    if is_dir {
                        dirs.push(Entry {
                            label: format!("{name}/"),
                            path,
                            is_dir: true,
                            is_parent: false,
                        });
                    } else if is_yaml(&path) {
                        files.push(Entry {
                            label: name,
                            path,
                            is_dir: false,
                            is_parent: false,
                        });
                    }
                }
            }
            Err(e) => self.error = Some(format!("Cannot read directory: {e}")),
        }
        dirs.sort_by_key(|a| a.label.to_lowercase());
        files.sort_by_key(|a| a.label.to_lowercase());
        self.entries.extend(dirs);
        self.entries.extend(files);
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        let cur = self.selected as isize;
        self.selected = (cur + delta).clamp(0, self.entries.len() as isize - 1) as usize;
    }

    pub fn selected_entry(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }

    /// Act on the selected entry. Returns `Some(file)` when a YAML file is
    /// chosen; otherwise navigates into a directory and returns `None`.
    pub fn activate(&mut self) -> Option<PathBuf> {
        let entry = self.entries.get(self.selected)?.clone();
        if entry.is_dir {
            self.cwd = entry.path;
            self.reload();
            None
        } else {
            Some(entry.path)
        }
    }
}

fn is_yaml(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("yaml") | Some("yml")
    )
}
