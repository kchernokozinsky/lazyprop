//! Key handling for the YAML editor screen, kept out of `app.rs`.

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::state::{Operation, State};
use crate::yaml_editor::state::{OpenMode, ToggleResult, YamlFocus};

fn ctrl(key: &KeyEvent, c: char) -> bool {
    key.code == KeyCode::Char(c) && key.modifiers.contains(KeyModifiers::CONTROL)
}

pub fn handle_key(key: KeyEvent, state: &mut State, tx: &UnboundedSender<Action>) -> Result<()> {
    // 1. Confirmation dialog takes precedence.
    if state.yaml.confirm.is_some() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => state.yaml.confirm_yes(),
            KeyCode::Char('n') | KeyCode::Esc => state.yaml.confirm_no(),
            _ => {}
        }
        return Ok(());
    }

    // 2. Open-file modal.
    if state.yaml.open_modal.is_some() {
        handle_modal(key, state);
        return Ok(());
    }

    // 3. Scalar edit mode.
    if state.yaml.editing.is_some() {
        match key.code {
            KeyCode::Esc => state.yaml.cancel_edit(),
            KeyCode::Enter => {
                if let Err(e) = state.yaml.apply_edit() {
                    state.yaml.report(e, true);
                }
            }
            KeyCode::Backspace => edit(state, |f| f.backspace()),
            KeyCode::Delete => edit(state, |f| f.delete()),
            KeyCode::Left => edit(state, |f| f.left()),
            KeyCode::Right => edit(state, |f| f.right()),
            KeyCode::Home => edit(state, |f| f.home()),
            KeyCode::End => edit(state, |f| f.end()),
            KeyCode::Char(c) => edit(state, |f| f.insert(c)),
            _ => {}
        }
        return Ok(());
    }

    // 4. Normal mode.
    // Screen switching (no text field is focused here).
    match key.code {
        KeyCode::Char('1') => return send(tx, Action::GoMain),
        KeyCode::Char('2') => return send(tx, Action::GoPlayground),
        KeyCode::Char('3') => return Ok(()), // already here
        KeyCode::Char('4') => return send(tx, Action::GoAbout),
        KeyCode::Char('h') => return send(tx, Action::PrevScreen),
        KeyCode::Char('l') => return send(tx, Action::NextScreen),
        _ => {}
    }
    if ctrl(&key, 'o') {
        state.yaml.open_dialog();
        return Ok(());
    }
    if ctrl(&key, 's') {
        state.yaml.request_save();
        return Ok(());
    }
    if ctrl(&key, 'r') {
        state.yaml.request_restore();
        return Ok(());
    }
    if ctrl(&key, 'c') || ctrl(&key, 'd') {
        return send(tx, Action::Quit);
    }

    match key.code {
        KeyCode::Char('q') => return send(tx, Action::Quit),
        KeyCode::Tab => {
            state.yaml.focus = match state.yaml.focus {
                YamlFocus::Tree => YamlFocus::Environments,
                YamlFocus::Environments => YamlFocus::Tree,
            };
        }
        KeyCode::Up | KeyCode::Char('w') => nav(state, -1),
        KeyCode::Down | KeyCode::Char('s') => nav(state, 1),
        KeyCode::Left => {
            if state.yaml.focus == YamlFocus::Tree {
                state.yaml.collapse_or_parent();
            }
        }
        KeyCode::Right => {
            if state.yaml.focus == YamlFocus::Tree {
                state.yaml.expand_selected();
            }
        }
        KeyCode::Enter => {
            if state.yaml.focus == YamlFocus::Tree {
                if let ToggleResult::EditScalar = state.yaml.toggle_or_edit() {
                    if let Err(e) = state.yaml.begin_edit() {
                        state.yaml.report(e, true);
                    }
                }
            }
        }
        KeyCode::Char('e') => state.yaml_begin_crypto(tx.clone(), Operation::Encrypt),
        KeyCode::Char('d') => state.yaml_begin_crypto(tx.clone(), Operation::Decrypt),
        KeyCode::Char('r') => state.yaml.reveal = !state.yaml.reveal,
        _ => {}
    }
    Ok(())
}

fn handle_modal(key: KeyEvent, state: &mut State) {
    let Some(modal) = state.yaml.open_modal.as_mut() else {
        return;
    };
    match key.code {
        KeyCode::Esc => state.yaml.close_dialog(),
        KeyCode::Tab => {
            modal.mode = match modal.mode {
                OpenMode::Browse => OpenMode::Path,
                OpenMode::Path => OpenMode::Browse,
            };
            modal.error = None;
        }
        KeyCode::Enter => {
            state.yaml.modal_activate();
        }
        _ => match modal.mode {
            OpenMode::Browse => match key.code {
                KeyCode::Up | KeyCode::Char('w') => modal.browser.move_selection(-1),
                KeyCode::Down | KeyCode::Char('s') => modal.browser.move_selection(1),
                _ => {}
            },
            OpenMode::Path => match key.code {
                KeyCode::Backspace => modal.path_input.backspace(),
                KeyCode::Delete => modal.path_input.delete(),
                KeyCode::Left => modal.path_input.left(),
                KeyCode::Right => modal.path_input.right(),
                KeyCode::Home => modal.path_input.home(),
                KeyCode::End => modal.path_input.end(),
                KeyCode::Char(c) => modal.path_input.insert(c),
                _ => {}
            },
        },
    }
}

fn nav(state: &mut State, delta: isize) {
    match state.yaml.focus {
        YamlFocus::Tree => state.yaml.move_selection(delta),
        YamlFocus::Environments => {
            if delta < 0 {
                state.prev();
            } else {
                state.next();
            }
        }
    }
}

fn edit(state: &mut State, f: impl FnOnce(&mut crate::text_field::TextField)) {
    if let Some(field) = state.yaml.editing.as_mut() {
        f(field);
    }
}

fn send(tx: &UnboundedSender<Action>, action: Action) -> Result<()> {
    let _ = tx.send(action);
    Ok(())
}
