use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{block::*, *},
};

use crate::{panes::Pane, state::State};

pub struct DetailsPane {}

impl DetailsPane {
    pub fn new() -> Self {
        Self {}
    }

    fn border_style(&self) -> Style {
        Style::default()
    }

    fn border_type(&self) -> BorderType {
        BorderType::Plain
    }
}

impl Default for DetailsPane {
    fn default() -> Self {
        Self::new()
    }
}

impl Pane for DetailsPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Fill(1)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        let status_block = Block::default()
            .title(format!(
                " {} Details ",
                state.envs.get(state.cur())?.name.clone()
            ))
            .borders(Borders::ALL)
            .border_style(self.border_style())
            .border_type(self.border_type());

        frame.render_widget(status_block, area);

        let inner_area = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let status_message = state.envs.get(state.cur())?.name.clone();

        let status_text = vec![Span::raw(status_message)];

        let status_paragraph = Paragraph::new(Line::from(status_text))
            .style(Style::default().fg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(status_paragraph, inner_area[1]);

        Ok(())
    }

    fn focusable(&self) -> bool {
        false
    }
}
