use crate::state::State;
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane, theme};

pub struct StatusPane {
    message: String,
    is_error: bool,
}

impl StatusPane {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            is_error: false,
        }
    }
}

impl Default for StatusPane {
    fn default() -> Self {
        Self::new()
    }
}

impl Pane for StatusPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Length(3)
    }

    fn update(&mut self, action: Action, _state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Error(message) => {
                self.message = message;
                self.is_error = true;
            }
            Action::Message(message) => {
                self.message = message;
                self.is_error = false;
            }
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, _state: &State) -> Result<()> {
        let block = Block::default().title(" Status ").borders(Borders::ALL);

        let (text, style): (Span, Style) = if self.message.is_empty() {
            (
                Span::raw("Ready. Press 2 for the playground, 3 for help."),
                theme::hint_italic(),
            )
        } else if self.is_error {
            (
                Span::raw(self.message.clone()),
                Style::default().fg(theme::error()),
            )
        } else {
            (
                Span::raw(self.message.clone()),
                Style::default().fg(theme::success()),
            )
        };

        let paragraph = Paragraph::new(Line::from(text))
            .style(style)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
        Ok(())
    }

    fn focusable(&self) -> bool {
        false
    }
}
