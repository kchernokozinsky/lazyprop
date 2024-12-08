pub mod app_state;
pub mod popup;

use std::time::Duration;

use anyhow::Result;
use app_state::AppState;
use crossterm::event::{self, Event, KeyCode};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};

use crate::{config::app::AppConfig, env::Environment};
use crate::{
    encryption::{decrypt, encrypt},
    tui::popup::{PopupField, PopupMode, ALGORITHMS, STATES},
};

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut AppState,
    config: &AppConfig,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, state))?;

        if crossterm::event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if state.popup.mode != PopupMode::None {
                    handle_popup_input(state, key.code, config)?;
                } else {
                    handle_normal_input(state, key.code, config)?
                }
            }
        }
    }
}

pub fn draw_ui<B: Backend>(f: &mut Frame<B>, state: &AppState) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(29),
            Constraint::Percentage(1),
        ])
        .split(size);

    let top_area = chunks[0];
    let middle_area = chunks[1];
    let hints_area = chunks[2];

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(top_area);

    let items: Vec<tui::widgets::ListItem> = state
        .envs
        .environments
        .iter()
        .enumerate()
        .map(|(i, env)| {
            let prefix = if i == state.selected_index {
                "> "
            } else {
                "  "
            };
            tui::widgets::ListItem::new(format!("{}{}", prefix, env.name))
        })
        .collect();

    let list = tui::widgets::List::new(items)
        .block(Block::default().title("Environments").borders(Borders::ALL));
    f.render_widget(list, top_chunks[0]);

    let details = if let Some(env) = state.envs.environments.get(state.selected_index) {
        format!(
            "Algorithm: {:?}\nState: {:?}\nUse Random IVs: {}\nKey: {}",
            env.algorithm, env.state, env.use_random_ivs, env.key
        )
    } else {
        "No environment selected".to_string()
    };

    let details_paragraph =
        Paragraph::new(details).block(Block::default().title("Details").borders(Borders::ALL));
    f.render_widget(details_paragraph, top_chunks[1]);

    let (title, content) = ("Status", &state.status_message);
    let paragraph =
        Paragraph::new(content.clone()).block(Block::default().title(title).borders(Borders::ALL));
    f.render_widget(paragraph, middle_area);

    if state.popup.mode != PopupMode::None {
        draw_popup(f, state);
    }

    let hints = "Hints: [a] Add Env | [e] Edit Env | [r] Remove Env | [Up/Down] Cycle Env | [s] Save | [q] Quit";
    f.render_widget(Clear, hints_area);
    let hints_par = Paragraph::new(hints).style(Style::default().fg(Color::Yellow));
    f.render_widget(hints_par, hints_area);
}

fn draw_popup<B: Backend>(f: &mut Frame<B>, state: &AppState) {
    match state.popup.mode {
        PopupMode::Add | PopupMode::Edit => {
            draw_add_edit_popup(f, state);
        }
        PopupMode::EncryptDecrypt => {
            draw_encrypt_decrypt_popup(f, state);
        }
        PopupMode::None => {}
    }
}

fn draw_add_edit_popup<B: Backend>(f: &mut Frame<B>, state: &AppState) {
    let size = f.size();
    let popup_width = size.width / 2;
    let popup_height = 10;
    let popup_x = (size.width - popup_width) / 2;
    let popup_y = (size.height - popup_height) / 2;
    let area = tui::layout::Rect::new(popup_x, popup_y, popup_width, popup_height);

    f.render_widget(Clear, area);

    let title = match state.popup.mode {
        PopupMode::Add => "Add Environment",
        PopupMode::Edit => "Edit Environment",
        _ => "",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::White));
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);

    let fields_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Name
            Constraint::Length(1), // Algorithm
            Constraint::Length(1), // State
            Constraint::Length(1), // IV
            Constraint::Length(1), // Key
        ])
        .margin(1)
        .split(inner);

    let highlight = |field: PopupField| {
        if state.popup.focus == field {
            "> "
        } else {
            "  "
        }
    };

    let text_style = Style::default().bg(Color::White).fg(Color::Black);
    let hints_text_style = Style::default().bg(Color::White).fg(Color::Yellow);

    let name_par = Paragraph::new(format!(
        "{}Name: {}",
        highlight(PopupField::Name),
        state.popup.name
    ))
    .style(text_style);
    f.render_widget(name_par, fields_layout[0]);

    let algo_par = Paragraph::new(format!(
        "{}Algorithm: {:?}",
        highlight(PopupField::Algorithm),
        ALGORITHMS[state.popup.selected_algorithm]
    ))
    .style(text_style);
    f.render_widget(algo_par, fields_layout[1]);

    let state_par = Paragraph::new(format!(
        "{}State: {:?}",
        highlight(PopupField::State),
        STATES[state.popup.selected_state]
    ))
    .style(text_style);
    f.render_widget(state_par, fields_layout[2]);

    let iv_par = Paragraph::new(format!(
        "{}Use Random IV: {}",
        highlight(PopupField::IV),
        state.popup.use_random_ivs
    ))
    .style(text_style);
    f.render_widget(iv_par, fields_layout[3]);

    let key_par = Paragraph::new(format!(
        "{}Key: {}",
        highlight(PopupField::Key),
        state.popup.key
    ))
    .style(text_style);
    f.render_widget(key_par, fields_layout[4]);

    let hints = [
        "[Tab] Next Field",
        "[Shift+Tab] Prev Field",
        "[Up/Down] Cycle Dropdown",
        "[Space] Toggle IV",
        "[Enter] Save",
        "[Esc] Cancel",
    ];

    let hints_height = hints.len() as u16;
    let hints_area =
        tui::layout::Rect::new(popup_x, popup_y + popup_height, popup_width, hints_height);

    f.render_widget(Clear, hints_area);

    let constraints: Vec<Constraint> = hints.iter().map(|_| Constraint::Length(1)).collect();
    let hints_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&*constraints)
        .margin(0)
        .split(hints_area);

    for (i, hint) in hints.iter().enumerate() {
        let hint_par = Paragraph::new(*hint).style(hints_text_style);
        f.render_widget(hint_par, hints_layout[i]);
    }
}

fn draw_encrypt_decrypt_popup<B: Backend>(f: &mut Frame<B>, state: &AppState) {
    let size = f.size();
    let popup_width = size.width / 2;
    let popup_height = 10;
    let popup_x = (size.width - popup_width) / 2;
    let popup_y = (size.height - popup_height) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Encrypt/Decrypt")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::White));
    f.render_widget(block.clone(), popup_area);

    let inner = block.inner(popup_area);

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), 
            Constraint::Length(1),
        ])
        .margin(1)
        .split(inner);

    let text_style = Style::default().bg(Color::White).fg(Color::Black);

    let text_input_block = Block::default()
        .borders(Borders::ALL)
        .title("Paste or write text here:");
    let text_input_paragraph = Paragraph::new(state.popup.text_input.clone())
        .style(text_style)
        .block(text_input_block);
    f.render_widget(text_input_paragraph, vertical_layout[0]);

    let buttons_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_layout[1]);

    let encrypt_highlight = if state.popup.focus == PopupField::EncryptButton {
        "> "
    } else {
        "  "
    };
    let decrypt_highlight = if state.popup.focus == PopupField::DecryptButton {
        "> "
    } else {
        "  "
    };

    let encrypt_par = Paragraph::new(format!("{}Encrypt", encrypt_highlight))
        .style(text_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(encrypt_par, buttons_layout[0]);

    let decrypt_par = Paragraph::new(format!("{}Decrypt", decrypt_highlight))
        .style(text_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(decrypt_par, buttons_layout[1]);

    let hints_y = popup_y + popup_height;
    let hints_height = 1;
    let hints_area = Rect::new(popup_x, hints_y, popup_width, hints_height);

    f.render_widget(Clear, hints_area);

    let hints = "[Enter] Confirm | [Esc] Cancel | [Tab] Switch Focus";
    let hints_par =
        Paragraph::new(hints).style(Style::default().bg(Color::White).fg(Color::Yellow));
    f.render_widget(hints_par, hints_area);
}

fn handle_normal_input(state: &mut AppState, code: KeyCode, config: &AppConfig) -> Result<()> {
    match code {
        KeyCode::Char('s') => {
            state.envs.save(&config.envs_path.display().to_string())?;
            state.status_message = "Environments are saved".to_string();
        }
        KeyCode::Char('q') => {
            std::process::exit(0);
        }
        KeyCode::Char('a') => {
            state.popup.mode = PopupMode::Add;
            state.popup.focus = PopupField::Name;
            state.popup.name.clear();
            state.popup.key.clear();
            state.popup.selected_algorithm = 0;
            state.popup.selected_state = 0;
            state.popup.use_random_ivs = true;
        }
        KeyCode::Char('e') => {
            if state.selected_index < state.len_env() {
                let curr = state.curr_env()?.clone();
                state.popup.mode = PopupMode::Edit;
                state.popup.focus = PopupField::Name;
                state.popup.name = curr.name.clone();
                state.popup.selected_algorithm = ALGORITHMS
                    .iter()
                    .position(|&a| a == curr.algorithm)
                    .unwrap_or(0);
                state.popup.selected_state =
                    STATES.iter().position(|&s| s == curr.state).unwrap_or(0);
                state.popup.use_random_ivs = curr.use_random_ivs;
                state.popup.key = curr.key.clone();
            }
        }
        KeyCode::Char('r') => {
            if let Err(err) = state.remove_env() {
                state.status_message = format!("Error removing env: {}", err);
            } else {
                if state.selected_index >= state.len_env() && !state.is_empty_env() {
                    state.selected_index = state.len_env() - 1;
                }
                state.status_message = "Environment removed".to_string();
            }
        }
        KeyCode::Down => {
            if state.selected_index + 1 < state.len_env() {
                state.selected_index += 1;
            } else {
                state.selected_index = 0;
            }
        }
        KeyCode::Up => {
            if state.selected_index > 0 {
                state.selected_index -= 1;
            } else {
                state.selected_index = state.len_env() - 1;
            }
        }
        KeyCode::Enter => {
            if state.selected_index < state.len_env() {
                state.popup.mode = PopupMode::EncryptDecrypt;
                state.popup.focus = PopupField::TextInput;
                state.popup.text_input.clear();
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_popup_input(state: &mut AppState, code: KeyCode, config: &AppConfig) -> Result<()> {
    match state.popup.mode {
        PopupMode::Add | PopupMode::Edit => handle_add_edit_popup_input(state, code),
        PopupMode::EncryptDecrypt => handle_encrypt_decrypt_popup_input(state, code, config),
        PopupMode::None => Ok(()),
    }
}

fn handle_add_edit_popup_input(state: &mut AppState, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Tab => {
            state.popup.focus = next_focus(state.popup.focus);
        }
        KeyCode::BackTab => {
            state.popup.focus = prev_focus(state.popup.focus);
        }
        KeyCode::Up => {
            if state.popup.focus == PopupField::Algorithm && state.popup.selected_algorithm > 0 {
                state.popup.selected_algorithm -= 1;
            } else if state.popup.focus == PopupField::State && state.popup.selected_state > 0 {
                state.popup.selected_state -= 1;
            }
        }
        KeyCode::Down => {
            if state.popup.focus == PopupField::Algorithm
                && state.popup.selected_algorithm + 1 < ALGORITHMS.len()
            {
                state.popup.selected_algorithm += 1;
            } else if state.popup.focus == PopupField::State
                && state.popup.selected_state + 1 < STATES.len()
            {
                state.popup.selected_state += 1;
            }
        }
        KeyCode::Char(' ') => {
            if state.popup.focus == PopupField::IV {
                state.popup.use_random_ivs = !state.popup.use_random_ivs;
            } else if state.popup.focus == PopupField::Name {
                state.popup.name.push(' ');
            } else if state.popup.focus == PopupField::Key {
                state.popup.key.push(' ');
            }
        }
        KeyCode::Char(c) => {
            if state.popup.focus == PopupField::Name {
                state.popup.name.push(c);
            } else if state.popup.focus == PopupField::Key {
                state.popup.key.push(c);
            }
        }
        KeyCode::Backspace => {
            if state.popup.focus == PopupField::Name {
                state.popup.name.pop();
            } else if state.popup.focus == PopupField::Key {
                state.popup.key.pop();
            }
        }
        KeyCode::Enter => {
            let new_env = Environment {
                name: state.popup.name.clone(),
                algorithm: ALGORITHMS[state.popup.selected_algorithm],
                state: STATES[state.popup.selected_state],
                use_random_ivs: state.popup.use_random_ivs,
                key: state.popup.key.clone(),
            };
            let res = match state.popup.mode {
                PopupMode::Add => state.add_env(new_env),
                PopupMode::Edit => state.edit_env(new_env),
                PopupMode::None | PopupMode::EncryptDecrypt => Ok(()),
            };
            match res {
                Ok(_) => state.status_message = "Environment edited".to_string(),
                Err(e) => state.status_message = format!("Error: {}", e),
            }
            state.popup.mode = PopupMode::None;
        }
        KeyCode::Esc => {
            state.popup.mode = PopupMode::None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_encrypt_decrypt_popup_input(
    state: &mut AppState,
    code: KeyCode,
    config: &AppConfig,
) -> Result<()> {
    match code {
        KeyCode::Tab => {
            state.popup.focus = next_focus_encrypt_decrypt(state.popup.focus);
        }
        KeyCode::BackTab => {
            state.popup.focus = prev_focus_encrypt_decrypt(state.popup.focus);
        }
        KeyCode::Char(c) => {
            if state.popup.focus == PopupField::TextInput {
                state.popup.text_input.push(c);
            }
        }
        KeyCode::Backspace => {
            if state.popup.focus == PopupField::TextInput {
                state.popup.text_input.pop();
            }
        }
        KeyCode::Enter => {
            if state.popup.focus == PopupField::EncryptButton {
                let encrypted = encrypt(
                    &state.popup.text_input,
                    state.curr_env()?,
                    config.jar_path.clone(),
                );
                match encrypted {
                    Ok(encr) => {
                        state.status_message = format!("Encrypted: {}", encr);
                    }
                    Err(e) => {
                        state.status_message = format!("Error: {}", e);
                    }
                }
                state.popup.mode = PopupMode::None;
            } else if state.popup.focus == PopupField::DecryptButton {
                let decrypted = decrypt(
                    &state.popup.text_input,
                    state.curr_env()?,
                    config.jar_path.clone(),
                );
                match decrypted {
                    Ok(decr) => {
                        state.status_message = format!("Decrypted: {}", decr);
                    }
                    Err(e) => {
                        state.status_message = format!("Error: {}", e);
                    }
                }
                state.popup.mode = PopupMode::None;
            }
        }
        KeyCode::Esc => {
            state.popup.mode = PopupMode::None;
        }
        _ => {}
    }
    Ok(())
}

fn next_focus(current: PopupField) -> PopupField {
    match current {
        PopupField::Name => PopupField::Algorithm,
        PopupField::Algorithm => PopupField::State,
        PopupField::State => PopupField::IV,
        PopupField::IV => PopupField::Key,
        PopupField::Key => PopupField::Name,
        _ => current,
    }
}

fn prev_focus(current: PopupField) -> PopupField {
    match current {
        PopupField::Name => PopupField::Key,
        PopupField::Algorithm => PopupField::Name,
        PopupField::State => PopupField::Algorithm,
        PopupField::IV => PopupField::State,
        PopupField::Key => PopupField::IV,
        _ => current,
    }
}

fn next_focus_encrypt_decrypt(current: PopupField) -> PopupField {
    match current {
        PopupField::TextInput => PopupField::EncryptButton,
        PopupField::EncryptButton => PopupField::DecryptButton,
        PopupField::DecryptButton => PopupField::TextInput,
        _ => current,
    }
}

fn prev_focus_encrypt_decrypt(current: PopupField) -> PopupField {
    match current {
        PopupField::TextInput => PopupField::DecryptButton,
        PopupField::EncryptButton => PopupField::TextInput,
        PopupField::DecryptButton => PopupField::EncryptButton,
        _ => current,
    }
}
