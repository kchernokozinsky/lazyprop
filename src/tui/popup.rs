use smart_default::SmartDefault;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::env::{Algorithm, State};

use super::app_state::AppState;

pub const ALGORITHMS: &[Algorithm] = &[
    Algorithm::AES,
    Algorithm::Blowfish,
    Algorithm::DES,
    Algorithm::DESede,
    Algorithm::RC2,
    Algorithm::RCA,
];

pub const STATES: &[State] = &[State::CBC, State::CFB, State::ECB, State::OFB];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupMode {
    None,
    Add,
    Edit,
    EncryptDecrypt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupField {
    Name,
    Algorithm,
    State,
    IV,
    Key,
    TextInput,
    EncryptButton,
    DecryptButton,
}

#[derive(Debug, SmartDefault)]
pub struct PopupState {
    #[default(PopupMode::None)]
    pub mode: PopupMode,
    #[default(PopupField::Name)]
    pub focus: PopupField,
    #[default(String::new())]
    pub name: String,
    #[default = 0]
    pub selected_algorithm: usize,
    #[default = 0]
    pub selected_state: usize,
    #[default = false]
    pub use_random_ivs: bool,
    #[default(String::new())]
    pub key: String,
    #[default(String::new())]
    pub text_input: String,
}

impl PopupState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn draw_popup<B: Backend>(f: &mut Frame<B>, state: &AppState) {
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
        .constraints([Constraint::Length(3), Constraint::Length(1)])
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
