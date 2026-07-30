use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use crate::{panes::Pane, state::State, theme};

/// Shows the configuration of the currently selected environment.
#[derive(Default)]
pub struct DetailsPane {}

impl DetailsPane {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pane for DetailsPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Length(8)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        let block = Block::default()
            .title(" Environment ")
            .borders(Borders::ALL);

        let Some(env) = state.selected_env() else {
            let empty = Paragraph::new("No environment selected.")
                .style(theme::hint_italic())
                .block(block);
            frame.render_widget(empty, area);
            return Ok(());
        };

        let label = theme::label();
        let key_display = if state.reveal_key {
            env.key.clone()
        } else {
            "•".repeat(env.key.chars().count())
        };

        let rows = [
            ("Name", env.name.clone()),
            ("Algorithm", format!("{:?}", env.algorithm)),
            ("Mode", format!("{:?}", env.state)),
            (
                "Random IV",
                if env.use_random_ivs { "yes" } else { "no" }.to_string(),
            ),
            (
                "Key",
                format!(
                    "{}  {}",
                    key_display,
                    if state.reveal_key {
                        "(r to hide)"
                    } else {
                        "(r to reveal)"
                    }
                ),
            ),
        ];

        let lines: Vec<Line> = rows
            .iter()
            .map(|(k, v)| {
                Line::from(vec![
                    Span::styled(format!("{k:>10}: "), label),
                    Span::raw(v.clone()),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        Ok(())
    }

    fn focusable(&self) -> bool {
        false
    }
}
