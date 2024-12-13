use std::fmt::format;

use crate::{config::Config, state::State};
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane};

#[derive(Debug)]
pub struct SearchPane {
    config: Config,
    focused: bool,
    focused_border_style: Style,
    input: Option<String>,
}

impl SearchPane {
    pub fn new(focused: bool, focused_border_style: Style, config: Config) -> Self {
        Self {
            config,
            focused,
            focused_border_style,
            input: None,
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
    fn input_length(&self) -> usize {
        match &self.input {
            Some(input) => input.len(),
            None => 0,
        }
    }
}

impl Pane for SearchPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Max(3)
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Input(c) => {
                // Append character to input
                match self.input {
                    Some(ref mut s) => s.push(c),
                    None => self.input = Some(format!("{}", c)),
                }
            }
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
        let search_block = Block::default()
            .title(" Search ")
            .borders(Borders::ALL)
            .border_style(self.border_style())
            .border_type(self.border_type());

        // Render the block
        frame.render_widget(search_block, area);

        // Define the inner area within the block with padding
        let inner_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(1) // Padding inside the block
            .constraints([Constraint::Max(1000)])
            .split(area)[0];

        let input_text = if let Some(ref input) = self.input {
            Span::raw(input)
        } else {
            Span::styled(
                "Type to search...",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )
        };

        // Optionally, add a cursor when focused
        let display_input = if self.focused && self.input.is_some() {
            Line::from(vec![input_text, Span::raw("▌")]) // ▌ as cursor
        } else {
            Line::from(vec![input_text])
        };

        // Create a Paragraph for the search input
        let search_paragraph = Paragraph::new(display_input)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        // Render the search input Paragraph
        frame.render_widget(search_paragraph, inner_area);

        Ok(())
    }

    fn focusable(&self) -> bool {
        true
    }
}
