use crate::state::State;
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane};

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

    fn border_style(&self) -> Style {
        Style::default()
    }

    fn border_type(&self) -> BorderType {
        BorderType::Plain
    }
}

impl Default for StatusPane {
    fn default() -> Self {
        Self::new()
    }
}

impl Pane for StatusPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Fill(1)
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
        let status_block = Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .border_style(self.border_style())
            .border_type(self.border_type());

        // Render the block
        frame.render_widget(status_block, area);

        // Define the inner area within the block (with some padding)
        let inner_area = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .margin(1) // Padding inside the block
            .constraints([
                Constraint::Fill(1), // For configuration paths // For status messages
            ])
            .split(area);

        // Prepare the status message text
        let status_message = self.message.clone();

        let status_text = vec![Span::raw(status_message)];
        // Add more status lines if needed;

        // Create a Paragraph for status messages
        let status_paragraph = Paragraph::new(Line::from(status_text))
            .style(Style::default().fg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        // Render the status messages Paragraph
        frame.render_widget(status_paragraph, inner_area[0]);

        Ok(())
    }

    fn focusable(&self) -> bool {
        false
    }
}
