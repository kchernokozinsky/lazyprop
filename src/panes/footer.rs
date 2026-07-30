use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::{app::Mode, panes::Pane, state::State, theme};

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
        let sep = Span::styled(" · ", theme::hint());
        let key = |k: &str| Span::styled(k.to_string(), theme::key());
        let desc = |d: &str| Span::raw(d.to_string());

        let hints: &[(&str, &str)] = match state.mode {
            Mode::Main => &[
                ("w/s", "move"),
                ("/", "search"),
                ("e", "encrypt"),
                ("d", "decrypt"),
                ("a", "add"),
                ("Enter", "edit"),
                ("x", "delete"),
                ("2", "about"),
                ("q", "quit"),
            ],
            Mode::About => &[("w/s", "scroll"), ("1", "back"), ("q", "quit")],
        };

        let mut spans = vec![Span::raw(" ")];
        for (k, d) in hints {
            spans.push(key(k));
            spans.push(Span::raw(" "));
            spans.push(desc(d));
            spans.push(sep.clone());
        }
        spans.pop(); // trailing separator

        frame.render_widget(Line::from(spans), area);
        Ok(())
    }

    fn focusable(&self) -> bool {
        false
    }
}
