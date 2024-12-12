use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::{cli::VERSION_MESSAGE, panes::Pane, state::State};

#[derive(Default)]
pub struct HeaderPane {}

impl HeaderPane {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pane for HeaderPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Max(1)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, _state: &State) -> Result<()> {
        frame.render_widget(
            Line::from(vec![
                Span::styled(
                    format!("[ {} {} ", "MULE LAZYPROP", symbols::DOT),
                    Style::default().fg(Color::Blue),
                ),
                Span::styled(
                    format!("{} ", VERSION_MESSAGE),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled("]", Style::default().fg(Color::Blue)),
            ])
            .right_aligned(),
            area,
        );

        Ok(())
    }
}
