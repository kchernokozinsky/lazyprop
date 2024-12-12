use crate::{config::Config, state::State};
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane};

pub struct DetailsPane {
    config: Config,
    focused: bool,
    focused_border_style: Style,
}

impl DetailsPane {
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

impl Pane for DetailsPane {
    fn height_constraint(&self) -> Constraint {
        match self.focused {
            true => Constraint::Fill(3),
            false => Constraint::Fill(3),
        }
    }

    fn update(&mut self, action: Action, _state: &mut State) -> Result<Option<Action>> {
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

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, _state: &State) -> Result<()> {
        let status_block = Block::default()
            .title(" Details ")
            .borders(Borders::ALL)
            .border_style(self.border_style())
            .border_type(self.border_type());

        frame.render_widget(status_block, area);

        let inner_area = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let config_text = "";

        let config_paragraph = Paragraph::new(config_text)
            .style(Style::default().fg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(config_paragraph, inner_area[0]);

        let status_message = "TO DO".to_string();

        let status_text = vec![Span::raw(status_message)];

        let status_paragraph = Paragraph::new(Line::from(status_text))
            .style(Style::default().fg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(status_paragraph, inner_area[1]);

        Ok(())
    }
}
