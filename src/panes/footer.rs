use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::{
    app::Mode,
    hints::{
        contextual_hints, render_footer_line, ConfirmationKind, HintContext, YamlHintFocus,
        YamlHints,
    },
    panes::Pane,
    state::{InputMode, PlaygroundField, State},
    yaml_editor::state::{Confirm, Guard, OpenMode, YamlFocus},
};

/// A one-line footer showing the actions valid for the current screen, focus and
/// mode. All hint text comes from the shared [`crate::hints`] renderer.
#[derive(Default)]
pub struct FooterPane {}

impl FooterPane {
    pub fn new() -> Self {
        Self {}
    }
}

/// Derive the active hint context from application state. Open modals and
/// confirmation dialogs take precedence over the screen behind them, so
/// background hints never leak into a popup.
pub fn hint_context(state: &State) -> HintContext {
    // Modals that can appear over any screen.
    if state.form.is_some() {
        return HintContext::EnvForm;
    }
    if state.pending_delete.is_some() {
        return HintContext::Confirmation(ConfirmationKind::DeleteEnv);
    }

    match state.mode {
        Mode::Main => {
            if state.searching {
                HintContext::MainSearching
            } else if state.input_mode == InputMode::Insert {
                HintContext::MainEditing
            } else {
                HintContext::Main
            }
        }
        Mode::Playground => match state.playground.field {
            PlaygroundField::Key | PlaygroundField::Value => HintContext::PlaygroundEditing,
            _ => HintContext::Playground,
        },
        Mode::About => HintContext::About,
        Mode::Yaml => yaml_context(state),
    }
}

fn yaml_context(state: &State) -> HintContext {
    let y = &state.yaml;
    // Unsaved-changes guard wins over everything.
    if let Some(guard) = y.guard() {
        return HintContext::Confirmation(match guard {
            Guard::Quit => ConfirmationKind::UnsavedQuit,
            Guard::Open(_) => ConfirmationKind::UnsavedOpen,
        });
    }
    if let Some(confirm) = y.confirm {
        return HintContext::Confirmation(match confirm {
            Confirm::Restore => ConfirmationKind::Restore,
            Confirm::OverwriteExternal => ConfirmationKind::OverwriteExternal,
        });
    }
    if let Some(modal) = &y.open_modal {
        return match modal.mode {
            OpenMode::Browse => {
                let on_dir = modal
                    .browser
                    .selected_entry()
                    .map(|e| e.is_dir)
                    .unwrap_or(false);
                HintContext::FileBrowser {
                    on_dir,
                    on_yaml: !on_dir,
                }
            }
            OpenMode::Path => HintContext::PathInput,
        };
    }
    if y.editing.is_some() {
        return HintContext::YamlEditing;
    }
    HintContext::Yaml(YamlHints {
        focus: match y.focus {
            YamlFocus::Environments => YamlHintFocus::Environments,
            YamlFocus::Tree => YamlHintFocus::Tree,
        },
        file_open: y.is_open(),
        crypto_in_progress: y.crypto_in_progress,
        dirty: y.dirty(),
        env_selected: state.selected_env().is_some(),
        selection: y.selection_kind(),
    })
}

impl Pane for FooterPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Max(1)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        let hints = contextual_hints(&hint_context(state));
        frame.render_widget(render_footer_line(hints, area.width as usize), area);
        Ok(())
    }

    fn focusable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_form_overrides_screen() {
        let mut state = State::for_test();
        state.mode = Mode::Main;
        state.open_add_form();
        assert_eq!(hint_context(&state), HintContext::EnvForm);
    }

    #[test]
    fn about_context_shows_screen_shortcuts() {
        let mut state = State::for_test();
        state.mode = Mode::About;
        let hints = contextual_hints(&hint_context(&state));
        assert!(hints
            .iter()
            .any(|h| h.key.chars().any(|c| ('1'..='4').contains(&c))));
    }
}
