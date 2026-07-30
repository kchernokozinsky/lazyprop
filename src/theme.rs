//! A small, theme-agnostic palette.
//!
//! The colours here are ANSI-named (not fixed RGB / indexed) so the terminal's
//! own light or dark theme maps them to appropriate values. Secondary text uses
//! the `DIM` modifier with no explicit colour instead of a grey, so it stays
//! legible on both light and dark backgrounds. No background colours are ever
//! set — the terminal background always shows through.

use ratatui::style::{Color, Modifier, Style};

/// Interactive accent: focus, selection, cursors, labels and key hints.
/// Cyan reads well on both light and dark backgrounds (unlike yellow on white).
pub const ACCENT: Color = Color::Cyan;

/// Successful / positive result.
pub const SUCCESS: Color = Color::Green;

/// Error / destructive.
pub const ERROR: Color = Color::Red;

/// De-emphasised secondary text (hints, placeholders).
pub fn hint() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// De-emphasised secondary text, italicised (empty states, placeholders).
pub fn hint_italic() -> Style {
    Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC)
}

/// Field / section labels.
pub fn label() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// A keybinding key (e.g. the `e` in "e encrypt").
pub fn key() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}
