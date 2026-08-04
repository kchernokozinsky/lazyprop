use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::{
    app::Mode,
    panes::Pane,
    state::{InputMode, State},
    theme,
};

/// A one-line footer showing the most useful keybindings for the active screen.
#[derive(Default)]
pub struct FooterPane {}

impl FooterPane {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pane for FooterPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Max(1)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        // On the main screen the hints depend on what currently has focus, so
        // only keys that actually do something are shown. Items are ordered
        // most-important first so the least useful drop off when narrow.
        let main_searching: &[(&str, &str)] =
            &[("type", "filter"), ("Esc", "done"), ("Tab", "focus")];
        let main_value: &[(&str, &str)] = &[("type", "value"), ("Esc", "done"), ("Tab", "focus")];
        let main_normal: &[(&str, &str)] = &[
            ("e", "encrypt"),
            ("d", "decrypt"),
            ("Ctrl-y", "copy"),
            ("/", "search"),
            ("a", "add"),
            ("Enter", "edit"),
            ("x", "delete"),
            ("r", "reveal"),
            ("w/s", "move"),
            ("4", "yaml"),
            ("q", "quit"),
        ];

        let hints: &[(&str, &str)] = match state.mode {
            Mode::Main => {
                if state.searching {
                    main_searching
                } else if state.input_mode == InputMode::Insert {
                    main_value
                } else {
                    main_normal
                }
            }
            Mode::Playground => &[
                ("Tab/↑↓", "move"),
                ("←/→", "change"),
                ("Enter", "generate"),
                ("Ctrl-y", "copy"),
                ("Esc", "back"),
                ("1/2/3", "screen"),
            ],
            Mode::About => &[
                ("1", "main"),
                ("2", "playground"),
                ("3", "about"),
                ("4", "yaml"),
                ("h/l", "switch"),
                ("w/s", "scroll"),
                ("q", "quit"),
            ],
            Mode::Yaml => {
                if state.yaml.editing.is_some() {
                    &[("type", "value"), ("Enter", "apply"), ("Esc", "cancel")]
                } else if state.yaml.open_modal.is_some() {
                    &[
                        ("↑↓", "move"),
                        ("Enter", "open"),
                        ("Tab", "mode"),
                        ("Esc", "cancel"),
                    ]
                } else {
                    &[
                        ("Ctrl-o", "open"),
                        ("w/s", "move"),
                        ("←/→", "fold"),
                        ("Enter", "edit"),
                        ("e", "encrypt"),
                        ("d", "decrypt"),
                        ("Ctrl-s", "save"),
                        ("Ctrl-r", "restore"),
                        ("r", "reveal"),
                        ("Tab", "focus"),
                    ]
                }
            }
        };

        frame.render_widget(Line::from(fit_hints(hints, area.width as usize)), area);
        Ok(())
    }

    fn focusable(&self) -> bool {
        false
    }
}

/// Build footer spans, keeping only as many hints as fit in `width` cells.
/// Items are consumed in order, so callers list the most useful first.
fn fit_hints(hints: &[(&str, &str)], width: usize) -> Vec<Span<'static>> {
    let sep = || Span::styled(" · ", theme::hint());
    let mut spans = vec![Span::raw(" ")];
    let mut used = 1usize; // leading space
    let mut first = true;

    for (k, d) in hints {
        // "k d" plus a separator when not the first item.
        let item_w = k.chars().count() + 1 + d.chars().count() + if first { 0 } else { 3 };
        if used + item_w + 1 > width {
            break;
        }
        if !first {
            spans.push(sep());
        }
        spans.push(Span::styled(k.to_string(), theme::key()));
        spans.push(Span::raw(" "));
        spans.push(Span::raw(d.to_string()));
        used += item_w;
        first = false;
    }
    spans
}
