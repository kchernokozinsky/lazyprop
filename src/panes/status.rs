use crate::{config::Config, state::State};
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane};

pub struct StatusPane {
    config: Config,
    focused: bool,
    focused_border_style: Style,
}

impl StatusPane {
    pub fn new(focused: bool, focused_border_style: Style, config: Config) -> Self {
        Self {
            config,
            focused,
            focused_border_style,
        }
    }

    fn border_style(&self) -> Style {
        match self.focused {
            true => self.focused_border_style,
            false => Style::default(),
        }
    }

    fn border_type(&self) -> BorderType {
        match self.focused {
            true => BorderType::Thick,
            false => BorderType::Plain,
        }
    }
}

impl Pane for StatusPane {
    fn height_constraint(&self) -> Constraint {
        match self.focused {
            true => Constraint::Fill(3),
            false => Constraint::Fill(3),
        }
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Focus => {
                self.focused = true;
            }
            Action::UnFocus => {
                self.focused = false;
            }
            Action::Submit => {}
            Action::Update => {}
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
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
                Constraint::Percentage(50), // For configuration paths
                Constraint::Percentage(50), // For status messages
            ])
            .split(area);

        // Prepare the configuration paths text
        let config_text = "";

        // Create a Paragraph for configuration paths
        let config_paragraph = Paragraph::new(config_text)
            .style(Style::default().fg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        // Render the configuration paths Paragraph
        frame.render_widget(config_paragraph, inner_area[0]);

        // Prepare the status message text
        let status_message = "TO DO".to_string();

        let status_text = vec![Span::raw(status_message)];
        // Add more status lines if needed;

        // Create a Paragraph for status messages
        let status_paragraph = Paragraph::new(Line::from(status_text))
            .style(Style::default().fg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        // Render the status messages Paragraph
        frame.render_widget(status_paragraph, inner_area[1]);

        Ok(())
    }
}
