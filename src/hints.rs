//! Centralised contextual keyboard hints.
//!
//! Every screen, pane and modal describes what it can do as a list of
//! [`KeyHint`]s built from a [`HintContext`]. A single renderer
//! ([`render_footer_line`]) turns that list into the one-line footer, dropping
//! the least important hints first when the terminal is narrow. Popups use
//! [`popup_action_line`] so their actions are always shown.
//!
//! This is the single source of truth for *displayed* shortcuts. The keys shown
//! here mirror the handlers in `yaml_editor::input` and the default keybindings
//! in `.config/config.json`; keep them in sync when either changes.

use ratatui::text::{Line, Span};

use crate::theme;

/// How important a hint is when space runs out. Critical hints are never
/// dropped; secondary hints go first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HintPriority {
    Secondary,
    Primary,
    Critical,
}

/// One keyboard hint: the key(s) to press and what they do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHint {
    pub key: String,
    pub description: String,
    pub priority: HintPriority,
}

impl KeyHint {
    pub fn new(key: &str, description: &str, priority: HintPriority) -> Self {
        Self {
            key: key.to_string(),
            description: description.to_string(),
            priority,
        }
    }
    pub fn critical(key: &str, description: &str) -> Self {
        Self::new(key, description, HintPriority::Critical)
    }
    pub fn primary(key: &str, description: &str) -> Self {
        Self::new(key, description, HintPriority::Primary)
    }
    pub fn secondary(key: &str, description: &str) -> Self {
        Self::new(key, description, HintPriority::Secondary)
    }

    /// Display width of "`key` `description`" in cells.
    fn width(&self) -> usize {
        self.key.chars().count() + 1 + self.description.chars().count()
    }
}

/// What the selected YAML node is, which decides the crypto/edit hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlSelectionKind {
    /// An editable, non-encrypted scalar (can be encrypted or edited).
    ScalarPlain,
    /// A scalar wrapped in `![...]` (can be decrypted or edited).
    ScalarEncrypted,
    /// A scalar the editor cannot edit in place (flow/anchor/block).
    ScalarUneditable,
    /// A mapping or sequence.
    Container { expanded: bool },
}

/// Which pane has focus on the YAML screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlHintFocus {
    Environments,
    Tree,
}

/// Everything the YAML footer needs to pick the right hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YamlHints {
    pub focus: YamlHintFocus,
    pub file_open: bool,
    pub crypto_in_progress: bool,
    pub dirty: bool,
    pub env_selected: bool,
    pub selection: Option<YamlSelectionKind>,
}

/// The kind of confirmation dialog currently open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationKind {
    /// Restore, discarding unsaved changes (y/n).
    Restore,
    /// Overwrite a file changed on disk (y/n).
    OverwriteExternal,
    /// Unsaved changes when quitting (save/discard/cancel).
    UnsavedQuit,
    /// Unsaved changes when opening another file (save/discard/cancel).
    UnsavedOpen,
    /// Delete an environment (y/n).
    DeleteEnv,
}

/// The full context that determines which hints are shown. A modal/popup
/// context is chosen over the screen behind it, so background hints never leak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintContext {
    /// Main screen, normal navigation.
    Main,
    /// Main screen, typing into the value field.
    MainEditing,
    /// Main screen, typing a search filter.
    MainSearching,
    /// Playground, moving between fields.
    Playground,
    /// Playground, typing into a text field.
    PlaygroundEditing,
    /// About screen (guide browser).
    About,
    /// YAML screen (non-modal).
    Yaml(YamlHints),
    /// YAML scalar edit mode.
    YamlEditing,
    /// File browser inside the open-file modal.
    FileBrowser { on_dir: bool, on_yaml: bool },
    /// Manual path entry inside the open-file modal.
    PathInput,
    /// A confirmation / unsaved-changes popup.
    Confirmation(ConfirmationKind),
    /// The add/edit environment form.
    EnvForm,
    /// The terminal is too small to render the UI.
    TooSmall,
}

/// The contextual hints for a given context, most important first.
pub fn contextual_hints(ctx: &HintContext) -> Vec<KeyHint> {
    use HintContext as C;
    match ctx {
        C::Main => vec![
            KeyHint::primary("E", "Encrypt"),
            KeyHint::primary("D", "Decrypt"),
            KeyHint::secondary("Ctrl+Y", "Copy"),
            KeyHint::secondary("/", "Search"),
            KeyHint::secondary("A", "Add"),
            KeyHint::secondary("Enter", "Edit"),
            KeyHint::secondary("X", "Delete"),
            KeyHint::secondary("R", "Reveal"),
            KeyHint::primary("W/S", "Move"),
            KeyHint::secondary("Tab", "Focus"),
            KeyHint::critical("Q", "Quit"),
        ],
        C::MainEditing => vec![
            KeyHint::critical("Esc", "Done"),
            KeyHint::primary("Tab", "Focus"),
        ],
        C::MainSearching => vec![
            KeyHint::critical("Esc", "Done"),
            KeyHint::primary("Tab", "Focus"),
        ],
        C::Playground => vec![
            KeyHint::primary("Tab/↑/↓", "Move"),
            KeyHint::primary("←/→", "Change"),
            KeyHint::primary("Enter", "Generate"),
            KeyHint::secondary("Ctrl+Y", "Copy"),
            KeyHint::critical("Esc", "Back"),
        ],
        C::PlaygroundEditing => vec![
            KeyHint::primary("Enter", "Generate"),
            KeyHint::critical("Esc", "Back"),
        ],
        C::About => vec![
            KeyHint::primary("←/→", "Guide"),
            KeyHint::primary("↑/↓", "Scroll"),
            KeyHint::critical("Esc", "Back"),
        ],
        C::YamlEditing => vec![
            KeyHint::primary("Enter", "Apply"),
            KeyHint::critical("Esc", "Cancel"),
            KeyHint::secondary("←/→", "Cursor"),
        ],
        C::FileBrowser { on_dir, on_yaml } => {
            let open = if *on_dir {
                "Open directory"
            } else if *on_yaml {
                "Open file"
            } else {
                "Open"
            };
            vec![
                KeyHint::primary("↑/↓", "Navigate"),
                KeyHint::primary("Enter", open),
                KeyHint::secondary("Tab", "Type path"),
                KeyHint::critical("Esc", "Cancel"),
            ]
        }
        C::PathInput => vec![
            KeyHint::primary("Enter", "Open"),
            KeyHint::secondary("Tab", "Browse"),
            KeyHint::critical("Esc", "Cancel"),
        ],
        C::Confirmation(kind) => confirmation_hints(*kind),
        C::EnvForm => vec![
            KeyHint::primary("Tab/↑/↓", "Move"),
            KeyHint::primary("←/→", "Change"),
            KeyHint::critical("Enter", "Save"),
            KeyHint::critical("Esc", "Cancel"),
        ],
        C::TooSmall => vec![KeyHint::critical("Q", "Quit")],
        C::Yaml(y) => yaml_hints(y),
    }
}

fn confirmation_hints(kind: ConfirmationKind) -> Vec<KeyHint> {
    use ConfirmationKind as K;
    match kind {
        K::UnsavedQuit | K::UnsavedOpen => vec![
            KeyHint::critical("S", "Save"),
            KeyHint::critical("D", "Discard"),
            KeyHint::critical("Esc", "Cancel"),
        ],
        K::Restore => vec![
            KeyHint::critical("Y", "Restore"),
            KeyHint::critical("Esc", "Cancel"),
        ],
        K::OverwriteExternal => vec![
            KeyHint::critical("Y", "Overwrite"),
            KeyHint::critical("Esc", "Cancel"),
        ],
        K::DeleteEnv => vec![
            KeyHint::critical("Y", "Delete"),
            KeyHint::critical("Esc", "Cancel"),
        ],
    }
}

fn yaml_hints(y: &YamlHints) -> Vec<KeyHint> {
    // No file open: only opening a file and leaving are valid.
    if !y.file_open {
        return vec![
            KeyHint::primary("Ctrl+O", "Open file"),
            KeyHint::secondary("Tab", "Focus"),
            KeyHint::critical("Esc", "Back"),
        ];
    }

    // Environment pane focused: only environment actions.
    if y.focus == YamlHintFocus::Environments {
        return vec![
            KeyHint::primary("W/S", "Select environment"),
            KeyHint::secondary("A", "Add environment"),
            KeyHint::primary("Tab", "Next pane"),
            KeyHint::critical("Q", "Quit"),
        ];
    }

    // Tree focused: hints depend on the selected node.
    let mut hints = vec![KeyHint::primary("W/S", "Navigate")];
    match y.selection {
        Some(YamlSelectionKind::Container { expanded }) => {
            if expanded {
                hints.push(KeyHint::primary("←/Enter", "Collapse"));
                hints.push(KeyHint::secondary("→", "Child"));
            } else {
                hints.push(KeyHint::primary("→/Enter", "Expand"));
                hints.push(KeyHint::secondary("←", "Parent"));
            }
        }
        Some(YamlSelectionKind::ScalarPlain) => {
            hints.push(KeyHint::primary("Enter", "Edit"));
            if y.env_selected && !y.crypto_in_progress {
                hints.push(KeyHint::primary("E", "Encrypt"));
            }
        }
        Some(YamlSelectionKind::ScalarEncrypted) => {
            hints.push(KeyHint::primary("Enter", "Edit"));
            if y.env_selected && !y.crypto_in_progress {
                hints.push(KeyHint::primary("D", "Decrypt"));
            }
            hints.push(KeyHint::secondary("R", "Reveal"));
        }
        Some(YamlSelectionKind::ScalarUneditable) => {}
        None => {}
    }
    hints.push(KeyHint::primary("Tab", "Next pane"));
    if y.crypto_in_progress {
        hints.push(KeyHint::secondary("…", "Working"));
    }
    hints.push(KeyHint::secondary("Ctrl+O", "Open"));
    if y.dirty {
        hints.push(KeyHint::secondary("Ctrl+S", "Save"));
        hints.push(KeyHint::secondary("Ctrl+R", "Restore"));
    }
    hints.push(KeyHint::critical("Q", "Quit"));
    hints
}

// --- rendering -------------------------------------------------------------

const SEP: &str = "   ";

/// Total width of a set of hints joined with separators.
fn total_width(hints: &[KeyHint]) -> usize {
    if hints.is_empty() {
        return 0;
    }
    let items: usize = hints.iter().map(KeyHint::width).sum();
    items + SEP.chars().count() * (hints.len() - 1)
}

/// Drop the lowest-priority, right-most hints until the set fits in `width`
/// cells. Critical hints are never dropped. Ordering of survivors is preserved.
pub fn fit_to_width(mut hints: Vec<KeyHint>, width: usize) -> Vec<KeyHint> {
    // Leading space in the footer.
    let avail = width.saturating_sub(1);
    for cutoff in [HintPriority::Secondary, HintPriority::Primary] {
        while total_width(&hints) > avail {
            // Remove the last removable hint at or below this cutoff.
            let Some(pos) = hints.iter().rposition(|h| h.priority <= cutoff) else {
                break;
            };
            hints.remove(pos);
        }
        if total_width(&hints) <= avail {
            break;
        }
    }
    hints
}

/// Render hints as a footer line, trimming to fit `width`.
pub fn render_footer_line(hints: Vec<KeyHint>, width: usize) -> Line<'static> {
    let hints = fit_to_width(hints, width);
    spans_for(&hints)
}

/// Render hints for a popup's action row (never trims — the popup is sized to
/// fit these). Falls back to a compact join if extremely narrow.
pub fn popup_action_line(hints: &[KeyHint]) -> Line<'static> {
    spans_for(hints)
}

/// Pack hints onto as few rows as fit in `width` cells, wrapping to new rows
/// rather than dropping any hint. Always returns at least one row (which may
/// itself overflow if a single hint is wider than `width`).
pub fn action_lines(hints: &[KeyHint], width: usize) -> Vec<Line<'static>> {
    if hints.is_empty() {
        return vec![Line::from(Span::raw(""))];
    }
    let avail = width.saturating_sub(1).max(1);
    let sep_w = SEP.chars().count();
    let mut rows: Vec<Vec<KeyHint>> = Vec::new();
    let mut cur: Vec<KeyHint> = Vec::new();
    let mut cur_w = 0usize;
    for h in hints {
        let add = if cur.is_empty() {
            h.width()
        } else {
            sep_w + h.width()
        };
        if !cur.is_empty() && cur_w + add > avail {
            rows.push(std::mem::take(&mut cur));
            cur_w = 0;
        }
        cur_w += if cur.is_empty() {
            h.width()
        } else {
            sep_w + h.width()
        };
        cur.push(h.clone());
    }
    if !cur.is_empty() {
        rows.push(cur);
    }
    rows.iter().map(|r| spans_for(r)).collect()
}

fn spans_for(hints: &[KeyHint]) -> Line<'static> {
    let mut spans = vec![Span::raw(" ")];
    for (i, h) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(SEP, theme::hint()));
        }
        spans.push(Span::styled(h.key.clone(), theme::key()));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(h.description.clone(), theme::hint()));
    }
    Line::from(spans)
}

/// Minimum width in cells needed to show every hint on one line.
pub fn min_action_width(hints: &[KeyHint]) -> usize {
    total_width(hints) + 2 // a leading and trailing space
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(hints: &[KeyHint]) -> Vec<String> {
        hints.iter().map(|h| h.key.clone()).collect()
    }
    fn descs(hints: &[KeyHint]) -> Vec<String> {
        hints.iter().map(|h| h.description.clone()).collect()
    }
    fn has_desc(hints: &[KeyHint], d: &str) -> bool {
        hints.iter().any(|h| h.description == d)
    }

    fn yaml(
        focus: YamlHintFocus,
        selection: Option<YamlSelectionKind>,
        file_open: bool,
    ) -> Vec<KeyHint> {
        contextual_hints(&HintContext::Yaml(YamlHints {
            focus,
            file_open,
            crypto_in_progress: false,
            dirty: false,
            env_selected: true,
            selection,
        }))
    }

    #[test]
    fn no_numeric_screen_shortcuts_anywhere() {
        let contexts = [
            HintContext::Main,
            HintContext::MainEditing,
            HintContext::MainSearching,
            HintContext::Playground,
            HintContext::PlaygroundEditing,
            HintContext::About,
            HintContext::YamlEditing,
            HintContext::PathInput,
            HintContext::FileBrowser {
                on_dir: false,
                on_yaml: true,
            },
            HintContext::Confirmation(ConfirmationKind::UnsavedQuit),
            HintContext::Yaml(YamlHints {
                focus: YamlHintFocus::Tree,
                file_open: true,
                crypto_in_progress: false,
                dirty: true,
                env_selected: true,
                selection: Some(YamlSelectionKind::ScalarPlain),
            }),
        ];
        for ctx in &contexts {
            for h in contextual_hints(ctx) {
                assert!(
                    !h.key.chars().any(|c| ('1'..='4').contains(&c)),
                    "numeric screen shortcut leaked in {ctx:?}: {}",
                    h.key
                );
            }
        }
    }

    #[test]
    fn yaml_no_file_hides_save_crypto_edit() {
        let h = yaml(YamlHintFocus::Tree, None, false);
        assert!(!has_desc(&h, "Save"));
        assert!(!has_desc(&h, "Encrypt"));
        assert!(!has_desc(&h, "Decrypt"));
        assert!(!has_desc(&h, "Edit"));
        assert!(has_desc(&h, "Open file"));
    }

    #[test]
    fn yaml_env_focus_only_env_actions() {
        let h = yaml(YamlHintFocus::Environments, None, true);
        assert!(has_desc(&h, "Select environment"));
        assert!(!has_desc(&h, "Encrypt"));
        assert!(!has_desc(&h, "Navigate")); // no tree navigation
    }

    #[test]
    fn yaml_container_hides_scalar_actions() {
        let collapsed = yaml(
            YamlHintFocus::Tree,
            Some(YamlSelectionKind::Container { expanded: false }),
            true,
        );
        assert!(has_desc(&collapsed, "Expand"));
        assert!(!has_desc(&collapsed, "Encrypt"));
        assert!(!has_desc(&collapsed, "Edit"));
        let expanded = yaml(
            YamlHintFocus::Tree,
            Some(YamlSelectionKind::Container { expanded: true }),
            true,
        );
        assert!(has_desc(&expanded, "Collapse"));
    }

    #[test]
    fn yaml_scalar_shows_scalar_actions() {
        let h = yaml(
            YamlHintFocus::Tree,
            Some(YamlSelectionKind::ScalarPlain),
            true,
        );
        assert!(has_desc(&h, "Edit"));
        assert!(has_desc(&h, "Navigate"));
    }

    #[test]
    fn plain_and_encrypted_scalars_get_matching_crypto_hints() {
        let plain = yaml(
            YamlHintFocus::Tree,
            Some(YamlSelectionKind::ScalarPlain),
            true,
        );
        assert!(has_desc(&plain, "Encrypt"));
        assert!(!has_desc(&plain, "Decrypt"));
        let enc = yaml(
            YamlHintFocus::Tree,
            Some(YamlSelectionKind::ScalarEncrypted),
            true,
        );
        assert!(has_desc(&enc, "Decrypt"));
        assert!(!has_desc(&enc, "Encrypt"));
    }

    #[test]
    fn yaml_edit_mode_only_editing_actions() {
        let h = contextual_hints(&HintContext::YamlEditing);
        assert!(has_desc(&h, "Apply"));
        assert!(has_desc(&h, "Cancel"));
        assert!(!has_desc(&h, "Encrypt"));
        assert!(!has_desc(&h, "Save"));
        assert!(!has_desc(&h, "Navigate"));
    }

    #[test]
    fn file_browser_hints() {
        let dir = contextual_hints(&HintContext::FileBrowser {
            on_dir: true,
            on_yaml: false,
        });
        assert!(has_desc(&dir, "Open directory"));
        assert!(has_desc(&dir, "Navigate"));
        let file = contextual_hints(&HintContext::FileBrowser {
            on_dir: false,
            on_yaml: true,
        });
        assert!(has_desc(&file, "Open file"));
    }

    #[test]
    fn path_input_hints() {
        let h = contextual_hints(&HintContext::PathInput);
        assert!(has_desc(&h, "Open"));
        assert!(has_desc(&h, "Cancel"));
        assert!(!has_desc(&h, "Navigate"));
    }

    #[test]
    fn confirmation_shows_only_its_actions_and_all_required() {
        let quit = contextual_hints(&HintContext::Confirmation(ConfirmationKind::UnsavedQuit));
        assert_eq!(descs(&quit), vec!["Save", "Discard", "Cancel"]);
        for h in &quit {
            assert_eq!(h.priority, HintPriority::Critical);
        }
        let del = contextual_hints(&HintContext::Confirmation(ConfirmationKind::DeleteEnv));
        assert!(has_desc(&del, "Delete"));
        assert!(has_desc(&del, "Cancel"));
    }

    #[test]
    fn critical_hints_survive_when_width_is_tiny() {
        let quit = contextual_hints(&HintContext::Confirmation(ConfirmationKind::UnsavedQuit));
        let fitted = fit_to_width(quit.clone(), 4); // absurdly small
                                                    // All three are critical, so none are dropped.
        assert_eq!(fitted.len(), 3);
        assert!(has_desc(&fitted, "Save"));
        assert!(has_desc(&fitted, "Discard"));
        assert!(has_desc(&fitted, "Cancel"));
    }

    #[test]
    fn low_priority_dropped_before_critical() {
        let main = contextual_hints(&HintContext::Main);
        let fitted = fit_to_width(main, 20);
        // Quit is critical and must remain.
        assert!(has_desc(&fitted, "Quit"));
        // Some secondary hint must have been dropped.
        assert!(!has_desc(&fitted, "Copy"));
    }

    #[test]
    fn no_duplicate_hints() {
        for ctx in [
            HintContext::Main,
            HintContext::Playground,
            HintContext::Yaml(YamlHints {
                focus: YamlHintFocus::Tree,
                file_open: true,
                crypto_in_progress: false,
                dirty: true,
                env_selected: true,
                selection: Some(YamlSelectionKind::ScalarEncrypted),
            }),
        ] {
            let h = contextual_hints(&ctx);
            let mut seen = std::collections::HashSet::new();
            for k in keys(&h) {
                assert!(seen.insert(k.clone()), "duplicate key {k} in {ctx:?}");
            }
        }
    }

    #[test]
    fn ordering_is_stable() {
        let a = keys(&contextual_hints(&HintContext::Main));
        let b = keys(&contextual_hints(&HintContext::Main));
        assert_eq!(a, b);
    }

    #[test]
    fn render_never_exceeds_width_after_fit() {
        let main = contextual_hints(&HintContext::Main);
        for w in [1usize, 5, 10, 20, 40, 80, 200] {
            let fitted = fit_to_width(main.clone(), w);
            assert!(
                total_width(&fitted) <= w.saturating_sub(1)
                    || fitted.iter().all(|h| h.priority == HintPriority::Critical)
            );
        }
    }
}
