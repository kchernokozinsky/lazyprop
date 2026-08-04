//! State for the YAML editor screen.
//!
//! Keeps the initial snapshot for the whole time a file is open, a
//! source-preserving working [`Document`], the navigation/expansion state, and
//! the bookkeeping for an in-flight crypto operation so its result is applied
//! only to the node that started it (never to whatever is selected when the
//! background job finishes).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::state::Operation;
use crate::text_field::TextField;
use crate::yaml_editor::document::{self, Document, NodeKind, PathSeg, ScalarStyle};
use crate::yaml_editor::file_browser::FileBrowser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlFocus {
    Environments,
    Tree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    Browse,
    Path,
}

/// The "Open YAML file" modal, offering browse and path-entry modes.
#[derive(Debug)]
pub struct OpenModal {
    pub mode: OpenMode,
    pub path_input: TextField,
    pub browser: FileBrowser,
    pub error: Option<String>,
}

impl Default for OpenModal {
    fn default() -> Self {
        Self {
            mode: OpenMode::Browse,
            path_input: TextField::default(),
            browser: FileBrowser::default(),
            error: None,
        }
    }
}

/// A pending destructive action awaiting confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confirm {
    /// Restore the document, discarding unsaved changes.
    Restore,
    /// Overwrite a file that changed on disk since it was opened.
    OverwriteExternal,
}

/// A crypto operation in flight, targeting a specific node by its stable path.
#[derive(Debug, Clone)]
struct Pending {
    op: Operation,
    path: Vec<PathSeg>,
    /// The exact source token we operated on, used to detect a stale result if
    /// the value changed in the meantime.
    original_source: String,
}

#[derive(Debug)]
pub struct YamlEditorState {
    pub file_path: Option<PathBuf>,
    initial_content: String,
    doc: Document,
    expanded: HashSet<Vec<PathSeg>>,
    selected_path: Option<Vec<PathSeg>>,
    pub focus: YamlFocus,
    pub editing: Option<TextField>,
    pub reveal: bool,
    pub crypto_in_progress: bool,
    message: Option<(String, bool)>,
    pending: Option<Pending>,
    /// Content hash of the file on disk at open / last save, for detecting
    /// external modification before overwriting.
    disk_hash: Option<u64>,
    pub open_modal: Option<OpenModal>,
    pub confirm: Option<Confirm>,
}

impl Default for YamlEditorState {
    fn default() -> Self {
        Self {
            file_path: None,
            initial_content: String::new(),
            doc: Document::parse(""),
            expanded: HashSet::new(),
            selected_path: None,
            focus: YamlFocus::Tree,
            editing: None,
            reveal: false,
            crypto_in_progress: false,
            message: None,
            pending: None,
            disk_hash: None,
            open_modal: None,
            confirm: None,
        }
    }
}

impl YamlEditorState {
    pub fn is_open(&self) -> bool {
        self.file_path.is_some()
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn dirty(&self) -> bool {
        self.is_open() && self.doc.raw() != self.initial_content
    }

    pub fn message(&self) -> Option<(&str, bool)> {
        self.message.as_ref().map(|(m, e)| (m.as_str(), *e))
    }

    fn set_msg(&mut self, text: impl Into<String>, is_error: bool) {
        self.message = Some((text.into(), is_error));
    }

    /// Report a message (e.g. a pre-flight crypto error) and clear the busy
    /// flag. Never include secret values in `text`.
    pub fn report(&mut self, text: impl Into<String>, is_error: bool) {
        self.crypto_in_progress = false;
        self.set_msg(text, is_error);
    }

    // --- opening -----------------------------------------------------------

    /// Validate and load a YAML file from a user-supplied path. On any failure
    /// the currently open document is left untouched.
    pub fn open_path(&mut self, input: &str) -> Result<(), String> {
        let path = resolve_path(input)?;
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        if !path.is_file() {
            return Err("Path is not a regular file".to_string());
        }
        match path.extension().and_then(|e| e.to_str()) {
            Some("yaml") | Some("yml") => {}
            _ => return Err("Not a .yaml/.yml file".to_string()),
        }
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        Document::validate(&content).map_err(|e| format!("Invalid YAML: {e}"))?;
        self.load_content(path, content);
        Ok(())
    }

    fn load_content(&mut self, path: PathBuf, content: String) {
        self.disk_hash = Some(hash(&content));
        self.initial_content = content.clone();
        self.doc = Document::parse(&content);
        self.file_path = Some(path);
        self.expanded.clear();
        // Expand top-level containers by default.
        for &root in self.doc.roots() {
            if let Some(n) = self.doc.node(root) {
                if n.kind != NodeKind::Scalar {
                    self.expanded.insert(n.path.clone());
                }
            }
        }
        self.selected_path = self
            .visible()
            .first()
            .map(|&id| self.doc.nodes()[id].path.clone());
        self.editing = None;
        self.pending = None;
        self.set_msg(
            format!("Opened {}", self.file_path.as_ref().unwrap().display()),
            false,
        );
    }

    // --- navigation --------------------------------------------------------

    /// Node ids in display order, honouring collapsed containers.
    pub fn visible(&self) -> Vec<usize> {
        let mut out = Vec::new();
        for &root in self.doc.roots() {
            self.push_visible(root, &mut out);
        }
        out
    }

    fn push_visible(&self, id: usize, out: &mut Vec<usize>) {
        out.push(id);
        let node = &self.doc.nodes()[id];
        if node.kind != NodeKind::Scalar && self.expanded.contains(&node.path) {
            for &child in &node.children {
                self.push_visible(child, out);
            }
        }
    }

    pub fn selected_id(&self) -> Option<usize> {
        let path = self.selected_path.as_ref()?;
        self.doc.find_by_path(path)
    }

    pub fn selected_index(&self) -> Option<usize> {
        let sel = self.selected_id()?;
        self.visible().iter().position(|&id| id == sel)
    }

    pub fn move_selection(&mut self, delta: isize) {
        let vis = self.visible();
        if vis.is_empty() {
            return;
        }
        let cur = self.selected_index().unwrap_or(0) as isize;
        let next = (cur + delta).clamp(0, vis.len() as isize - 1) as usize;
        self.selected_path = Some(self.doc.nodes()[vis[next]].path.clone());
    }

    fn selected_is_container(&self) -> bool {
        self.selected_id()
            .map(|id| self.doc.nodes()[id].kind != NodeKind::Scalar)
            .unwrap_or(false)
    }

    pub fn expand_selected(&mut self) {
        if let Some(id) = self.selected_id() {
            let node = &self.doc.nodes()[id];
            if node.kind != NodeKind::Scalar {
                self.expanded.insert(node.path.clone());
            }
        }
    }

    /// Collapse the selected container, or move to the parent if it's a leaf or
    /// already collapsed.
    pub fn collapse_or_parent(&mut self) {
        if let Some(id) = self.selected_id() {
            let node = self.doc.nodes()[id].clone();
            if node.kind != NodeKind::Scalar && self.expanded.contains(&node.path) {
                self.expanded.remove(&node.path);
                return;
            }
            if let Some(parent) = node.parent {
                self.selected_path = Some(self.doc.nodes()[parent].path.clone());
            }
        }
    }

    pub fn toggle_or_edit(&mut self) -> ToggleResult {
        if self.selected_is_container() {
            if let Some(id) = self.selected_id() {
                let path = self.doc.nodes()[id].path.clone();
                if self.expanded.contains(&path) {
                    self.expanded.remove(&path);
                } else {
                    self.expanded.insert(path);
                }
            }
            ToggleResult::Toggled
        } else {
            ToggleResult::EditScalar
        }
    }

    pub fn selected_path_string(&self) -> Option<String> {
        self.selected_path
            .as_ref()
            .map(|p| document::path_to_string(p))
    }

    /// (source, style, kind) of the selected node.
    pub fn selected_info(&self) -> Option<(String, ScalarStyle, NodeKind)> {
        let id = self.selected_id()?;
        let node = &self.doc.nodes()[id];
        let src = self.doc.value_source(id).unwrap_or("").to_string();
        Some((src, node.style, node.kind))
    }

    pub fn is_expanded(&self, path: &[PathSeg]) -> bool {
        self.expanded.contains(path)
    }

    // --- editing -----------------------------------------------------------

    /// Enter edit mode for the selected scalar, if possible.
    pub fn begin_edit(&mut self) -> Result<(), String> {
        let id = self.selected_id().ok_or("Nothing selected")?;
        let node = &self.doc.nodes()[id];
        if node.kind != NodeKind::Scalar {
            return Err("Cannot edit a mapping or sequence".to_string());
        }
        if !node.is_editable_scalar() {
            return Err("This value cannot be edited in place".to_string());
        }
        let logical = self.doc.logical_value(id).unwrap_or_default();
        self.editing = Some(TextField::from_text(&logical));
        Ok(())
    }

    pub fn cancel_edit(&mut self) {
        self.editing = None;
    }

    /// Apply the in-progress manual edit to the selected scalar.
    pub fn apply_edit(&mut self) -> Result<(), String> {
        let field = self.editing.take().ok_or("Not editing")?;
        let id = self.selected_id().ok_or("Nothing selected")?;
        let new_source = document::serialize_scalar(&field.value());
        let text = self.doc.replace_scalar_source(id, &new_source)?;
        self.doc = Document::parse(&text);
        self.set_msg("Value updated.", false);
        Ok(())
    }

    // --- crypto ------------------------------------------------------------

    /// Prepare a crypto operation on the selected scalar. Returns the value to
    /// pass to the tool (plaintext for encrypt, unwrapped ciphertext for
    /// decrypt) and records the target so the async result lands on the right
    /// node. `env_selected` must already be verified by the caller.
    pub fn begin_crypto(&mut self, op: Operation) -> Result<String, String> {
        if self.crypto_in_progress {
            return Err("A crypto operation is already running".to_string());
        }
        let id = self.selected_id().ok_or("Nothing selected")?;
        let node = self.doc.nodes()[id].clone();
        if node.kind != NodeKind::Scalar {
            return Err("Select a scalar value, not a mapping or sequence".to_string());
        }
        if !node.is_editable_scalar() {
            return Err("This value cannot be modified in place".to_string());
        }
        let logical = self.doc.logical_value(id).unwrap_or_default();
        if logical.is_empty() {
            return Err("The value is empty".to_string());
        }
        let send = match op {
            Operation::Encrypt => logical,
            Operation::Decrypt => document::unwrap_cipher(&logical),
        };
        let original_source = self.doc.value_source(id).unwrap_or("").to_string();
        self.pending = Some(Pending {
            op,
            path: node.path.clone(),
            original_source,
        });
        self.crypto_in_progress = true;
        self.set_msg(
            match op {
                Operation::Encrypt => "Encrypting…",
                Operation::Decrypt => "Decrypting…",
            },
            false,
        );
        Ok(send)
    }

    /// Apply (or discard as stale) a completed crypto result.
    pub fn finish_crypto(&mut self, outcome: Result<String, String>) {
        self.crypto_in_progress = false;
        let Some(pending) = self.pending.take() else {
            return;
        };
        let output = match outcome {
            Ok(v) => v,
            Err(e) => {
                // Do not include the value in the error.
                self.set_msg(format!("Operation failed: {}", first_line(&e)), true);
                return;
            }
        };
        // Locate the node by its stable path and verify it hasn't changed.
        let Some(id) = self.doc.find_by_path(&pending.path) else {
            self.set_msg("Result ignored: the target node no longer exists.", true);
            return;
        };
        if self.doc.value_source(id) != Some(pending.original_source.as_str()) {
            self.set_msg("Result ignored: the value changed while running.", true);
            return;
        }
        let new_source = match pending.op {
            Operation::Encrypt => document::serialize_scalar(&document::wrap_cipher(&output)),
            Operation::Decrypt => document::serialize_scalar(&output),
        };
        match self.doc.replace_scalar_source(id, &new_source) {
            Ok(text) => {
                self.doc = Document::parse(&text);
                self.set_msg(
                    match pending.op {
                        Operation::Encrypt => "Encrypted.",
                        Operation::Decrypt => "Decrypted.",
                    },
                    false,
                );
            }
            Err(e) => self.set_msg(format!("Could not apply result: {e}"), true),
        }
    }

    // --- save / restore ----------------------------------------------------

    /// Whether the file changed on disk since it was opened or last saved.
    pub fn externally_modified(&self) -> bool {
        match (&self.file_path, self.disk_hash) {
            (Some(p), Some(h)) => std::fs::read_to_string(p)
                .map(|c| hash(&c) != h)
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Save the working document atomically to the original file.
    pub fn save(&mut self) -> Result<(), String> {
        let path = self.file_path.clone().ok_or("No file open")?;
        Document::validate(self.doc.raw()).map_err(|e| format!("Invalid YAML: {e}"))?;
        atomic_write(&path, self.doc.raw())?;
        self.initial_content = self.doc.raw().to_string();
        self.disk_hash = Some(hash(self.doc.raw()));
        self.set_msg("Saved.", false);
        Ok(())
    }

    /// Restore the working document to the exact initial snapshot. Nothing is
    /// written to disk.
    pub fn restore(&mut self) {
        self.doc = Document::parse(&self.initial_content);
        self.editing = None;
        self.pending = None;
        // Keep the selection valid.
        if self.selected_id().is_none() {
            self.selected_path = self
                .visible()
                .first()
                .map(|&id| self.doc.nodes()[id].path.clone());
        }
        self.set_msg("Restored to the initially opened content.", false);
    }

    // --- open modal & confirmations ---------------------------------------

    pub fn open_dialog(&mut self) {
        self.open_modal = Some(OpenModal::default());
    }

    pub fn close_dialog(&mut self) {
        self.open_modal = None;
    }

    /// Handle activation inside the open modal (Enter). Returns true when a file
    /// was opened (so the modal can close).
    pub fn modal_activate(&mut self) -> bool {
        let Some(modal) = self.open_modal.as_mut() else {
            return false;
        };
        let candidate = match modal.mode {
            OpenMode::Browse => modal.browser.activate(),
            OpenMode::Path => Some(std::path::PathBuf::from(modal.path_input.value())),
        };
        let Some(path) = candidate else {
            return false; // navigated into a directory
        };
        let input = path.to_string_lossy().to_string();
        match self.open_path(&input) {
            Ok(()) => {
                self.open_modal = None;
                true
            }
            Err(e) => {
                if let Some(m) = self.open_modal.as_mut() {
                    m.error = Some(e);
                }
                false
            }
        }
    }

    /// Restore, asking for confirmation first if there are unsaved changes.
    pub fn request_restore(&mut self) {
        if self.dirty() {
            self.confirm = Some(Confirm::Restore);
        } else {
            self.restore();
        }
    }

    /// Save, asking to confirm if the file changed on disk since it was opened.
    pub fn request_save(&mut self) {
        if !self.is_open() {
            self.set_msg("No file open.", true);
            return;
        }
        if self.externally_modified() {
            self.confirm = Some(Confirm::OverwriteExternal);
            self.set_msg("File changed on disk. Confirm to overwrite.", true);
            return;
        }
        if let Err(e) = self.save() {
            self.set_msg(format!("Save failed: {e}"), true);
        }
    }

    pub fn confirm_yes(&mut self) {
        match self.confirm.take() {
            Some(Confirm::Restore) => self.restore(),
            Some(Confirm::OverwriteExternal) => {
                if let Err(e) = self.save() {
                    self.set_msg(format!("Save failed: {e}"), true);
                }
            }
            None => {}
        }
    }

    pub fn confirm_no(&mut self) {
        self.confirm = None;
    }
}

pub enum ToggleResult {
    Toggled,
    EditScalar,
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").to_string()
}

/// Expand `~` and normalize a user-supplied path.
fn resolve_path(input: &str) -> Result<PathBuf, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Enter a path".to_string());
    }
    if let Some(rest) = trimmed.strip_prefix('~') {
        let home = directories::BaseDirs::new()
            .map(|b| b.home_dir().to_path_buf())
            .ok_or("Cannot resolve home directory")?;
        let rest = rest.strip_prefix(['/', '\\']).unwrap_or(rest);
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(trimmed))
}

fn hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// Write `content` to `path` via a temp file in the same directory, then rename
/// over the original (atomic where the platform supports it).
fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp = dir.join(format!(
        ".{}.lazyprop.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("yaml")
    ));
    std::fs::write(&tmp, content).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let _ = std::fs::set_permissions(
                &tmp,
                std::fs::Permissions::from_mode(meta.permissions().mode()),
            );
        }
    }
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        e.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "database:\n  username: admin\n  password: secret\nservers:\n  - host: server-one\n    port: 8081\n";

    fn open_sample() -> (YamlEditorState, tempfile_path::Temp) {
        let tmp = tempfile_path::Temp::new(SAMPLE);
        let mut st = YamlEditorState::default();
        st.open_path(tmp.path.to_str().unwrap()).unwrap();
        (st, tmp)
    }

    #[test]
    fn visible_respects_expand_collapse() {
        let (mut st, _t) = open_sample();
        // Top-level containers expanded by default -> children visible.
        let count_expanded = st.visible().len();
        // Collapse `database`.
        st.selected_path = Some(vec![PathSeg::Key("database".into())]);
        st.collapse_or_parent();
        assert!(st.visible().len() < count_expanded);
    }

    #[test]
    fn dirty_and_restore_roundtrip() {
        let (mut st, _t) = open_sample();
        assert!(!st.dirty());
        st.selected_path = Some(vec![
            PathSeg::Key("database".into()),
            PathSeg::Key("password".into()),
        ]);
        st.editing = Some(TextField::from_text("changed"));
        st.apply_edit().unwrap();
        assert!(st.dirty());
        assert!(st.doc().raw().contains("password: changed"));
        st.restore();
        assert!(!st.dirty());
        assert!(st.doc().raw().contains("password: secret"));
    }

    #[test]
    fn crypto_result_lands_on_correct_node() {
        let (mut st, _t) = open_sample();
        st.selected_path = Some(vec![
            PathSeg::Key("database".into()),
            PathSeg::Key("password".into()),
        ]);
        let sent = st.begin_crypto(Operation::Encrypt).unwrap();
        assert_eq!(sent, "secret");
        // Move selection elsewhere while "running".
        st.selected_path = Some(vec![PathSeg::Key("servers".into())]);
        st.finish_crypto(Ok("CIPHER".to_string()));
        // Applied to password (the initiating node), not to servers.
        assert!(st.doc().raw().contains("password: \"![CIPHER]\""));
        assert!(st.doc().raw().contains("username: admin"));
    }

    #[test]
    fn stale_crypto_result_is_ignored() {
        let (mut st, _t) = open_sample();
        st.selected_path = Some(vec![
            PathSeg::Key("database".into()),
            PathSeg::Key("password".into()),
        ]);
        st.begin_crypto(Operation::Encrypt).unwrap();
        // Value changes underneath the running op.
        st.editing = Some(TextField::from_text("rotated"));
        st.apply_edit().unwrap();
        st.finish_crypto(Ok("CIPHER".to_string()));
        assert!(st.doc().raw().contains("password: rotated"));
        assert!(!st.doc().raw().contains("CIPHER"));
        assert!(st.message().unwrap().1); // error/ignored message
    }

    #[test]
    fn decrypt_unwraps_before_sending() {
        let src = "db:\n  password: \"![CIPHER]\"\n";
        let tmp = tempfile_path::Temp::new(src);
        let mut st = YamlEditorState::default();
        st.open_path(tmp.path.to_str().unwrap()).unwrap();
        st.selected_path = Some(vec![
            PathSeg::Key("db".into()),
            PathSeg::Key("password".into()),
        ]);
        let sent = st.begin_crypto(Operation::Decrypt).unwrap();
        assert_eq!(sent, "CIPHER");
        st.finish_crypto(Ok("secret".to_string()));
        assert!(st.doc().raw().contains("password: secret"));
    }

    #[test]
    fn save_and_reopen() {
        let (mut st, tmp) = open_sample();
        st.selected_path = Some(vec![
            PathSeg::Key("database".into()),
            PathSeg::Key("password".into()),
        ]);
        st.editing = Some(TextField::from_text("newpass"));
        st.apply_edit().unwrap();
        st.save().unwrap();
        assert!(!st.dirty());
        let on_disk = std::fs::read_to_string(&tmp.path).unwrap();
        assert!(on_disk.contains("password: newpass"));
    }

    // Minimal temp-file helper (avoids a new dependency).
    mod tempfile_path {
        use std::path::PathBuf;
        pub struct Temp {
            pub path: PathBuf,
        }
        impl Temp {
            pub fn new(content: &str) -> Self {
                let path = std::env::temp_dir().join(format!(
                    "lazyprop_yaml_{}_{}.yaml",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                ));
                std::fs::write(&path, content).unwrap();
                Temp { path }
            }
        }
        impl Drop for Temp {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}
