//! A small, theme-agnostic palette.
//!
//! The colours here are ANSI-named (not fixed RGB / indexed) by default so the
//! terminal's own light or dark theme maps them to appropriate values.
//! Secondary text uses the `DIM` modifier with no explicit colour instead of a
//! grey, so it stays legible on both light and dark backgrounds. No background
//! colours are ever set — the terminal background always shows through.
//!
//! The three role colours (accent, success, error) can be overridden from the
//! config file's `theme` section (e.g. `theme: { accent: "magenta" }`); call
//! [`init`] once at start-up with the parsed map. Unset roles keep the
//! defaults below.

use std::{collections::HashMap, sync::OnceLock};

use ratatui::style::{Color, Modifier, Style};

/// Runtime-resolved role colours. Initialised once from config at start-up.
#[derive(Clone, Copy, Debug)]
struct Palette {
    accent: Color,
    success: Color,
    error: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            // Cyan reads well on both light and dark backgrounds.
            accent: Color::Cyan,
            success: Color::Green,
            error: Color::Red,
        }
    }
}

static PALETTE: OnceLock<Palette> = OnceLock::new();

/// Apply theme overrides from the config `theme` section. Recognised keys:
/// `accent`, `success`, `error`; each value is a colour name understood by the
/// config style parser (e.g. `magenta`, `bright blue`, `color12`). Unknown
/// keys and unparseable colours are ignored, keeping the default. Safe to call
/// at most once; later calls are no-ops.
pub fn init(overrides: &HashMap<String, String>) {
    let mut p = Palette::default();
    if let Some(c) = overrides
        .get("accent")
        .and_then(|s| crate::config::parse_color(s))
    {
        p.accent = c;
    }
    if let Some(c) = overrides
        .get("success")
        .and_then(|s| crate::config::parse_color(s))
    {
        p.success = c;
    }
    if let Some(c) = overrides
        .get("error")
        .and_then(|s| crate::config::parse_color(s))
    {
        p.error = c;
    }
    let _ = PALETTE.set(p);
}

fn palette() -> Palette {
    *PALETTE.get_or_init(Palette::default)
}

/// Interactive accent: focus, selection, cursors, labels and key hints.
pub fn accent() -> Color {
    palette().accent
}

/// Successful / positive result.
pub fn success() -> Color {
    palette().success
}

/// Error / destructive.
pub fn error() -> Color {
    palette().error
}

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
    Style::default().fg(accent()).add_modifier(Modifier::BOLD)
}

/// A keybinding key (e.g. the `e` in "e encrypt").
pub fn key() -> Style {
    Style::default().fg(accent()).add_modifier(Modifier::BOLD)
}
